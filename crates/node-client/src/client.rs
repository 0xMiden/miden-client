use std::mem::ManuallyDrop;
use std::path::PathBuf;
use std::sync::Arc;

use miden_client::asset::FungibleAsset;
use miden_client::auth::{
    AuthEcdsaK256Keccak,
    AuthFalcon512Rpo,
    AuthSchemeId as NativeAuthSchemeId,
    AuthSecretKey,
};
use miden_client::crypto::RpoRandomCoin;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::{BlockNumber, NoteTag};
use miden_client::rpc::{Endpoint, GrpcClient};
use miden_client::store::NoteFilter as NativeNoteFilter;
use miden_client::transaction::{
    PaymentNoteDescription,
    SwapTransactionData,
    TransactionRequestBuilder as NativeTransactionRequestBuilder,
};
use miden_client::{
    Client,
    ExecutionOptions,
    Felt,
    MAX_TX_EXECUTION_CYCLES,
    MIN_TX_EXECUTION_CYCLES,
};
use miden_client_sqlite_store::SqliteStore;
use napi::bindgen_prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};
use tokio::runtime::Runtime;

use crate::error::to_napi_err;
use crate::models::account::Account;
use crate::models::account_header::AccountHeader;
use crate::models::account_id::AccountId;
use crate::models::account_storage_mode::AccountStorageMode;
use crate::models::address::Address;
use crate::models::auth::{AuthScheme, AuthSecretKey as NodeAuthSecretKey};
use crate::models::input_note_record::InputNoteRecord;
use crate::models::note_filter::NoteFilterType;
use crate::models::note_id::NoteId;
use crate::models::output_note_record::OutputNoteRecord;
use crate::models::sync_summary::SyncSummary;
use crate::models::transaction_id::TransactionId;
use crate::models::word::Word;

// NODE CLIENT
// ================================================================================================

#[napi]
pub struct NodeClient {
    inner: ManuallyDrop<Client<FilesystemKeyStore>>,
    keystore: ManuallyDrop<FilesystemKeyStore>,
    rt: Runtime,
}

impl Drop for NodeClient {
    fn drop(&mut self) {
        // Enter the tokio runtime context so that deadpool connections in SqliteStore
        // can be cleaned up properly (they require an active tokio reactor).
        let _guard = self.rt.enter();
        unsafe {
            ManuallyDrop::drop(&mut self.inner);
            ManuallyDrop::drop(&mut self.keystore);
        }
    }
}

#[napi]
impl NodeClient {
    /// Creates a new NodeClient connected to a Miden node.
    ///
    /// # Arguments
    /// * `rpc_url` - URL of the Miden node RPC endpoint (e.g. "https://rpc.testnet.miden.io:443").
    ///   If not provided, defaults to the testnet endpoint.
    /// * `db_path` - Path to the SQLite database file for client state.
    /// * `keys_dir` - Path to the directory where keys are stored.
    /// * `seed` - Optional 32-byte seed for deterministic key generation.
    #[napi(factory)]
    pub fn create_client(
        rpc_url: Option<String>,
        db_path: String,
        keys_dir: String,
        seed: Option<Buffer>,
    ) -> Result<NodeClient> {
        let rt = Runtime::new().map_err(|e| {
            napi::Error::from_reason(format!("Failed to create tokio runtime: {e}"))
        })?;

        let endpoint = match rpc_url {
            Some(url) => Endpoint::try_from(url.as_str())
                .map_err(|e| napi::Error::from_reason(format!("Invalid node URL: {e}")))?,
            None => Endpoint::testnet(),
        };

        let rpc_client = Arc::new(GrpcClient::new(&endpoint, 10_000));

        let mut rng = match seed {
            Some(seed_bytes) => {
                if seed_bytes.len() != 32 {
                    return Err(napi::Error::from_reason("Seed must be exactly 32 bytes"));
                }
                let mut seed_array = [0u8; 32];
                seed_array.copy_from_slice(&seed_bytes);
                StdRng::from_seed(seed_array)
            },
            None => StdRng::from_os_rng(),
        };

        let coin_seed: [u64; 4] = rng.random();
        let rpo_rng = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

        let keystore = FilesystemKeyStore::new(PathBuf::from(&keys_dir))
            .map_err(|e| napi::Error::from_reason(format!("Failed to create keystore: {e}")))?;

        let store = rt.block_on(async {
            SqliteStore::new(PathBuf::from(&db_path))
                .await
                .map_err(|e| napi::Error::from_reason(format!("Failed to create store: {e}")))
        })?;

        let store = Arc::new(store);

        let exec_options = ExecutionOptions::new(
            Some(MAX_TX_EXECUTION_CYCLES),
            MIN_TX_EXECUTION_CYCLES,
            false,
            false,
        )
        .expect("Default executor's options should always be valid");

        let client = rt.block_on(async {
            Client::new(
                rpc_client,
                Box::new(rpo_rng),
                store,
                Some(Arc::new(keystore.clone())),
                exec_options,
                None,
                None,
                None,
                None,
            )
            .await
            .map_err(|e| to_napi_err(e, "Failed to create client"))
        })?;

        Ok(NodeClient {
            inner: ManuallyDrop::new(client),
            keystore: ManuallyDrop::new(keystore),
            rt,
        })
    }

    // ACCOUNT METHODS
    // --------------------------------------------------------------------------------------------

    /// Returns all account headers tracked by this client.
    #[napi(js_name = "getAccounts")]
    pub fn get_accounts(&mut self) -> Result<Vec<AccountHeader>> {
        self.rt.block_on(async {
            let result = self
                .inner
                .get_account_headers()
                .await
                .map_err(|e| to_napi_err(e, "failed to get accounts"))?;
            Ok(result.into_iter().map(|(header, _)| header.into()).collect())
        })
    }

    /// Returns an account by its ID, or None if not found.
    #[napi(js_name = "getAccount")]
    pub fn get_account(&mut self, account_id: &AccountId) -> Result<Option<Account>> {
        self.rt.block_on(async {
            let result = self
                .inner
                .get_account(account_id.0)
                .await
                .map_err(|e| to_napi_err(e, "failed to get account"))?;

            match result {
                Some(record) => {
                    let native_account: miden_client::account::Account =
                        record.try_into().map_err(|_| {
                            napi::Error::from_reason("retrieval of partial account unsupported")
                        })?;
                    Ok(Some(native_account.into()))
                },
                None => Ok(None),
            }
        })
    }

    /// Returns all public key commitments associated with the given account ID.
    #[napi(js_name = "getPublicKeyCommitmentsOfAccount")]
    pub fn get_public_key_commitments_of(&mut self, account_id: &AccountId) -> Result<Vec<Word>> {
        self.rt.block_on(async {
            let commitments = self
                .inner
                .get_account_public_key_commitments(&account_id.0)
                .await
                .map_err(|e| to_napi_err(e, "failed to get public key commitments"))?;
            Ok(commitments.into_iter().map(miden_client::Word::from).map(Into::into).collect())
        })
    }

    /// Retrieves an auth secret key from the keystore given a public key commitment.
    #[napi(js_name = "getAccountAuthByPubKeyCommitment")]
    pub fn get_account_auth_by_pub_key_commitment(
        &self,
        pub_key_commitment: &Word,
    ) -> Result<NodeAuthSecretKey> {
        let key = self
            .keystore
            .get_key(pub_key_commitment.0.into())
            .map_err(|e| napi::Error::from_reason(format!("failed to get auth key: {e}")))?
            .ok_or_else(|| napi::Error::from_reason("Auth not found for account"))?;
        Ok(key.into())
    }

    /// Creates a new wallet account.
    #[napi(js_name = "newWallet")]
    pub fn new_wallet(
        &mut self,
        storage_mode: AccountStorageMode,
        mutable: bool,
        auth_scheme: AuthScheme,
        init_seed: Option<Buffer>,
    ) -> Result<Account> {
        let (new_account, key_pair) =
            generate_wallet(&storage_mode, mutable, init_seed, &auth_scheme)?;

        self.rt.block_on(async {
            self.inner
                .add_account(&new_account, false)
                .await
                .map_err(|e| to_napi_err(e, "failed to insert new wallet"))?;

            self.keystore
                .add_key(&key_pair)
                .map_err(|e| napi::Error::from_reason(format!("failed to store key: {e}")))?;

            self.inner
                .register_account_public_key_commitments(
                    &new_account.id(),
                    &[key_pair.public_key()],
                )
                .await
                .map_err(|e| to_napi_err(e, "failed to map account to public keys"))?;

            Ok(new_account.into())
        })
    }

    /// Creates a new fungible faucet account.
    #[napi(js_name = "newFaucet")]
    pub fn new_faucet(
        &mut self,
        storage_mode: AccountStorageMode,
        non_fungible: bool,
        token_symbol: String,
        decimals: u8,
        max_supply: BigInt,
        auth_scheme: AuthScheme,
    ) -> Result<Account> {
        use miden_client::account::component::BasicFungibleFaucet;
        use miden_client::account::{AccountBuilder, AccountType};
        use miden_client::asset::TokenSymbol;

        if non_fungible {
            return Err(napi::Error::from_reason("Non-fungible faucets are not supported yet"));
        }

        let mut seed = [0u8; 32];
        self.inner.rng().fill_bytes(&mut seed);
        let mut faucet_rng = StdRng::from_seed(seed);

        let native_scheme: NativeAuthSchemeId = auth_scheme.into();
        let (key_pair, auth_component) = match native_scheme {
            NativeAuthSchemeId::Falcon512Rpo => {
                let kp = AuthSecretKey::new_falcon512_rpo_with_rng(&mut faucet_rng);
                let comp: miden_client::account::AccountComponent =
                    AuthFalcon512Rpo::new(kp.public_key().to_commitment()).into();
                (kp, comp)
            },
            NativeAuthSchemeId::EcdsaK256Keccak => {
                let kp = AuthSecretKey::new_ecdsa_k256_keccak_with_rng(&mut faucet_rng);
                let comp: miden_client::account::AccountComponent =
                    AuthEcdsaK256Keccak::new(kp.public_key().to_commitment()).into();
                (kp, comp)
            },
            _ => {
                return Err(napi::Error::from_reason(format!(
                    "unsupported auth scheme: {native_scheme:?}"
                )));
            },
        };

        let symbol = TokenSymbol::new(&token_symbol)
            .map_err(|e| napi::Error::from_reason(format!("Invalid token symbol: {e}")))?;
        let max_supply_u64 = bigint_to_u64(&max_supply, "max_supply")?;
        let max_supply_felt = Felt::try_from(max_supply_u64.to_le_bytes().as_slice())
            .map_err(|e| napi::Error::from_reason(format!("Invalid max_supply value: {e}")))?;

        let mut init_seed = [0u8; 32];
        faucet_rng.fill_bytes(&mut init_seed);

        let native_storage_mode: miden_client::account::AccountStorageMode = storage_mode.into();

        let new_account = AccountBuilder::new(init_seed)
            .account_type(AccountType::FungibleFaucet)
            .storage_mode(native_storage_mode)
            .with_auth_component(auth_component)
            .with_component(
                BasicFungibleFaucet::new(symbol, decimals, max_supply_felt)
                    .map_err(|e| to_napi_err(e, "failed to create new faucet"))?,
            )
            .build()
            .map_err(|e| napi::Error::from_reason(format!("Failed to create new faucet: {e:?}")))?;

        self.keystore
            .add_key(&key_pair)
            .map_err(|e| napi::Error::from_reason(format!("failed to store key: {e}")))?;

        self.rt.block_on(async {
            self.inner
                .register_account_public_key_commitments(
                    &new_account.id(),
                    &[key_pair.public_key()],
                )
                .await
                .map_err(|e| to_napi_err(e, "failed to map account to public keys"))?;

            self.inner
                .add_account(&new_account, false)
                .await
                .map_err(|e| to_napi_err(e, "Failed to insert new faucet"))?;

            Ok(new_account.into())
        })
    }

    /// Adds a previously-created account to the client.
    #[napi(js_name = "newAccount")]
    pub fn new_account(&mut self, account: &Account, overwrite: bool) -> Result<()> {
        let native_account: miden_client::account::Account = account.into();
        self.rt.block_on(async {
            self.inner
                .add_account(&native_account, overwrite)
                .await
                .map_err(|e| to_napi_err(e, "failed to insert new account"))
        })
    }

    /// Stores a secret key in the keystore for a given account.
    #[napi(js_name = "addAccountSecretKey")]
    pub fn add_account_secret_key(
        &mut self,
        account_id: &AccountId,
        secret_key: &NodeAuthSecretKey,
    ) -> Result<()> {
        let native_key: AuthSecretKey = secret_key.into();
        self.keystore
            .add_key(&native_key)
            .map_err(|e| napi::Error::from_reason(format!("failed to store key: {e}")))?;

        self.rt.block_on(async {
            self.inner
                .register_account_public_key_commitments(&account_id.0, &[native_key.public_key()])
                .await
                .map_err(|e| to_napi_err(e, "failed to map account to public keys"))
        })
    }

    /// Inserts an address for a given account.
    #[napi(js_name = "insertAccountAddress")]
    pub fn insert_account_address(
        &mut self,
        account_id: &AccountId,
        address: &Address,
    ) -> Result<()> {
        self.rt.block_on(async {
            self.inner
                .add_address(address.0.clone(), account_id.0)
                .await
                .map_err(|e| to_napi_err(e, "failed to add address to account"))
        })
    }

    /// Removes an address for a given account.
    #[napi(js_name = "removeAccountAddress")]
    pub fn remove_account_address(
        &mut self,
        account_id: &AccountId,
        address: &Address,
    ) -> Result<()> {
        self.rt.block_on(async {
            self.inner
                .remove_address(address.0.clone(), account_id.0)
                .await
                .map_err(|e| to_napi_err(e, "failed to remove address from account"))
        })
    }

    // SYNC METHODS
    // --------------------------------------------------------------------------------------------

    /// Synchronizes the client state with the Miden network.
    #[napi(js_name = "syncState")]
    pub fn sync_state(&mut self) -> Result<SyncSummary> {
        self.rt.block_on(async {
            let summary = self
                .inner
                .sync_state()
                .await
                .map_err(|e| to_napi_err(e, "failed to sync state"))?;
            Ok(summary.into())
        })
    }

    /// Returns the current sync height.
    #[napi(js_name = "getSyncHeight")]
    pub fn get_sync_height(&mut self) -> Result<u32> {
        self.rt.block_on(async {
            let height = self
                .inner
                .get_sync_height()
                .await
                .map_err(|e| to_napi_err(e, "failed to get sync height"))?;
            Ok(height.as_u32())
        })
    }

    // TAG METHODS
    // --------------------------------------------------------------------------------------------

    /// Adds a note tag to track.
    #[napi(js_name = "addTag")]
    pub fn add_tag(&mut self, tag: String) -> Result<()> {
        let note_tag_as_u32: u32 = tag
            .parse()
            .map_err(|e| napi::Error::from_reason(format!("failed to parse note tag: {e}")))?;
        let note_tag: NoteTag = note_tag_as_u32.into();
        self.rt.block_on(async {
            self.inner
                .add_note_tag(note_tag)
                .await
                .map_err(|e| to_napi_err(e, "failed to add note tag"))?;
            Ok(())
        })
    }

    /// Removes a note tag.
    #[napi(js_name = "removeTag")]
    pub fn remove_tag(&mut self, tag: String) -> Result<()> {
        let note_tag_as_u32: u32 = tag
            .parse()
            .map_err(|e| napi::Error::from_reason(format!("failed to parse note tag: {e}")))?;
        let note_tag: NoteTag = note_tag_as_u32.into();
        self.rt.block_on(async {
            self.inner
                .remove_note_tag(note_tag)
                .await
                .map_err(|e| to_napi_err(e, "failed to remove note tag"))?;
            Ok(())
        })
    }

    /// Lists all tracked note tags.
    #[napi(js_name = "listTags")]
    pub fn list_tags(&mut self) -> Result<Vec<String>> {
        self.rt.block_on(async {
            let tags = self
                .inner
                .get_note_tags()
                .await
                .map_err(|e| to_napi_err(e, "failed to get note tags"))?;
            Ok(tags.into_iter().map(|t| t.tag.to_string()).collect())
        })
    }

    // SETTINGS METHODS
    // --------------------------------------------------------------------------------------------

    /// Gets a setting value by key.
    #[napi(js_name = "getSetting")]
    pub fn get_setting(&mut self, key: String) -> Result<Option<Buffer>> {
        self.rt.block_on(async {
            let result = self
                .inner
                .get_setting(key)
                .await
                .map_err(|e| to_napi_err(e, "failed to get setting"))?;
            Ok(result.map(|bytes: Vec<u8>| bytes.into()))
        })
    }

    /// Sets a setting key-value pair.
    #[napi(js_name = "setSetting")]
    pub fn set_setting(&mut self, key: String, value: Buffer) -> Result<()> {
        self.rt.block_on(async {
            self.inner
                .set_setting(key, value.to_vec())
                .await
                .map_err(|e| to_napi_err(e, "failed to set setting"))
        })
    }

    /// Removes a setting by key.
    #[napi(js_name = "removeSetting")]
    pub fn remove_setting(&mut self, key: String) -> Result<()> {
        self.rt.block_on(async {
            self.inner
                .remove_setting(key)
                .await
                .map_err(|e| to_napi_err(e, "failed to remove setting"))
        })
    }

    /// Lists all setting keys.
    #[napi(js_name = "listSettingKeys")]
    pub fn list_setting_keys(&mut self) -> Result<Vec<String>> {
        self.rt.block_on(async {
            self.inner
                .list_setting_keys()
                .await
                .map_err(|e| to_napi_err(e, "failed to list setting keys"))
        })
    }

    // NOTE METHODS
    // --------------------------------------------------------------------------------------------

    /// Returns input notes matching the given filter.
    #[napi(js_name = "getInputNotes")]
    pub fn get_input_notes(&mut self, filter: NoteFilterType) -> Result<Vec<InputNoteRecord>> {
        let native_filter = match filter {
            NoteFilterType::All => NativeNoteFilter::All,
            NoteFilterType::Consumed => NativeNoteFilter::Consumed,
            NoteFilterType::Committed => NativeNoteFilter::Committed,
            NoteFilterType::Expected => NativeNoteFilter::Expected,
            NoteFilterType::Processing => NativeNoteFilter::Processing,
            NoteFilterType::Unverified => NativeNoteFilter::Unverified,
        };
        self.rt.block_on(async {
            let notes = self
                .inner
                .get_input_notes(native_filter)
                .await
                .map_err(|e| to_napi_err(e, "failed to get input notes"))?;
            Ok(notes.into_iter().map(Into::into).collect())
        })
    }

    /// Returns output notes matching the given filter.
    #[napi(js_name = "getOutputNotes")]
    pub fn get_output_notes(&mut self, filter: NoteFilterType) -> Result<Vec<OutputNoteRecord>> {
        let native_filter = match filter {
            NoteFilterType::All => NativeNoteFilter::All,
            NoteFilterType::Consumed => NativeNoteFilter::Consumed,
            NoteFilterType::Committed => NativeNoteFilter::Committed,
            NoteFilterType::Expected => NativeNoteFilter::Expected,
            NoteFilterType::Processing => NativeNoteFilter::Processing,
            NoteFilterType::Unverified => NativeNoteFilter::Unverified,
        };
        self.rt.block_on(async {
            let notes = self
                .inner
                .get_output_notes(native_filter)
                .await
                .map_err(|e| to_napi_err(e, "failed to get output notes"))?;
            Ok(notes.into_iter().map(Into::into).collect())
        })
    }

    /// Returns consumable notes, optionally filtered by account.
    #[napi(js_name = "getConsumableNotes")]
    pub fn get_consumable_notes(
        &mut self,
        account_id: Option<&AccountId>,
    ) -> Result<Vec<InputNoteRecord>> {
        let native_id = account_id.map(|id| id.0);
        self.rt.block_on(async {
            let notes = self
                .inner
                .get_consumable_notes(native_id)
                .await
                .map_err(|e| to_napi_err(e, "failed to get consumable notes"))?;
            Ok(notes.into_iter().map(|(note, _)| note.into()).collect())
        })
    }

    // TRANSACTION METHODS
    // --------------------------------------------------------------------------------------------

    /// Creates a mint transaction request for a fungible asset.
    #[napi(js_name = "newMintTransactionRequest")]
    pub fn new_mint_transaction_request(
        &mut self,
        target_account_id: &AccountId,
        faucet_id: &AccountId,
        note_type: String,
        amount: BigInt,
    ) -> Result<Buffer> {
        let amount_u64 = bigint_to_u64(&amount, "amount")?;
        let fungible_asset = FungibleAsset::new(faucet_id.0, amount_u64)
            .map_err(|e| to_napi_err(e, "failed to create fungible asset"))?;

        let native_note_type = parse_note_type(&note_type)?;

        let tx_request = NativeTransactionRequestBuilder::new()
            .build_mint_fungible_asset(
                fungible_asset,
                target_account_id.0,
                native_note_type,
                self.inner.rng(),
            )
            .map_err(|e| to_napi_err(e, "failed to create mint transaction request"))?;

        let bytes = miden_client::utils::Serializable::to_bytes(&tx_request);
        Ok(bytes.into())
    }

    /// Creates a send (pay-to-id) transaction request.
    #[napi(js_name = "newSendTransactionRequest")]
    pub fn new_send_transaction_request(
        &mut self,
        sender_account_id: &AccountId,
        target_account_id: &AccountId,
        faucet_id: &AccountId,
        note_type: String,
        amount: BigInt,
        recall_height: Option<u32>,
    ) -> Result<Buffer> {
        let amount_u64 = bigint_to_u64(&amount, "amount")?;
        let fungible_asset = FungibleAsset::new(faucet_id.0, amount_u64)
            .map_err(|e| to_napi_err(e, "failed to create fungible asset"))?;

        let native_note_type = parse_note_type(&note_type)?;

        let mut payment_description = PaymentNoteDescription::new(
            vec![fungible_asset.into()],
            sender_account_id.0,
            target_account_id.0,
        );

        if let Some(recall_height) = recall_height {
            payment_description =
                payment_description.with_reclaim_height(BlockNumber::from(recall_height));
        }

        let tx_request = NativeTransactionRequestBuilder::new()
            .build_pay_to_id(payment_description, native_note_type, self.inner.rng())
            .map_err(|e| to_napi_err(e, "failed to create send transaction request"))?;

        let bytes = miden_client::utils::Serializable::to_bytes(&tx_request);
        Ok(bytes.into())
    }

    /// Creates a swap transaction request.
    #[napi(js_name = "newSwapTransactionRequest")]
    pub fn new_swap_transaction_request(
        &mut self,
        sender_account_id: &AccountId,
        offered_asset_faucet_id: &AccountId,
        offered_asset_amount: BigInt,
        requested_asset_faucet_id: &AccountId,
        requested_asset_amount: BigInt,
        note_type: String,
        payback_note_type: String,
    ) -> Result<Buffer> {
        let offered_u64 = bigint_to_u64(&offered_asset_amount, "offered_asset_amount")?;
        let requested_u64 = bigint_to_u64(&requested_asset_amount, "requested_asset_amount")?;

        let offered = FungibleAsset::new(offered_asset_faucet_id.0, offered_u64)
            .map_err(|e| to_napi_err(e, "failed to create offered fungible asset"))?
            .into();

        let requested = FungibleAsset::new(requested_asset_faucet_id.0, requested_u64)
            .map_err(|e| to_napi_err(e, "failed to create requested fungible asset"))?
            .into();

        let swap_data = SwapTransactionData::new(sender_account_id.0, offered, requested);

        let native_note_type = parse_note_type(&note_type)?;
        let native_payback_type = parse_note_type(&payback_note_type)?;

        let tx_request = NativeTransactionRequestBuilder::new()
            .build_swap(&swap_data, native_note_type, native_payback_type, self.inner.rng())
            .map_err(|e| to_napi_err(e, "failed to create swap transaction request"))?;

        let bytes = miden_client::utils::Serializable::to_bytes(&tx_request);
        Ok(bytes.into())
    }

    /// Executes, proves, submits, and applies a transaction in one step.
    #[napi(js_name = "submitNewTransaction")]
    pub fn submit_new_transaction(
        &mut self,
        account_id: &AccountId,
        transaction_request_bytes: Buffer,
    ) -> Result<TransactionId> {
        let tx_request = miden_client::transaction::TransactionRequest::read_from_bytes(
            &transaction_request_bytes,
        )
        .map_err(|e| {
            napi::Error::from_reason(format!("Failed to deserialize transaction request: {e}"))
        })?;

        self.rt.block_on(async {
            // Execute
            let tx_result = self
                .inner
                .execute_transaction(account_id.0, tx_request)
                .await
                .map_err(|e| to_napi_err(e, "failed to execute transaction"))?;

            let tx_id = tx_result.executed_transaction().id();

            // Prove
            let prover = self.inner.prover();
            let proven_tx = self
                .inner
                .prove_transaction_with(&tx_result, prover)
                .await
                .map_err(|e| to_napi_err(e, "failed to prove transaction"))?;

            // Submit
            let submission_height = self
                .inner
                .submit_proven_transaction(proven_tx, &tx_result)
                .await
                .map_err(|e| to_napi_err(e, "failed to submit proven transaction"))?;

            // Apply
            let update = self
                .inner
                .get_transaction_store_update(&tx_result, submission_height)
                .await
                .map_err(|e| to_napi_err(e, "failed to build transaction update"))?;

            self.inner
                .apply_transaction_update(update)
                .await
                .map_err(|e| to_napi_err(e, "failed to apply transaction result"))?;

            Ok(tx_id.into())
        })
    }

    // IMPORT/EXPORT METHODS
    // --------------------------------------------------------------------------------------------

    /// Imports an account file (account + keys).
    #[napi(js_name = "importAccountFile")]
    pub fn import_account_file(
        &mut self,
        account_bytes: Buffer,
        key_bytes: Vec<Buffer>,
    ) -> Result<String> {
        use miden_client::account::Account as NativeAccount;
        use miden_client::utils::Deserializable;

        let account = NativeAccount::read_from_bytes(&account_bytes)
            .map_err(|e| napi::Error::from_reason(format!("Failed to deserialize account: {e}")))?;

        let account_id = account.id().to_string();

        let mut keys = Vec::new();
        for kb in &key_bytes {
            let key = AuthSecretKey::read_from_bytes(kb)
                .map_err(|e| napi::Error::from_reason(format!("Failed to deserialize key: {e}")))?;
            keys.push(key);
        }

        self.rt.block_on(async {
            self.inner
                .add_account(&account, false)
                .await
                .map_err(|e| to_napi_err(e, "failed to import account"))?;

            for key in &keys {
                self.keystore
                    .add_key(key)
                    .map_err(|e| napi::Error::from_reason(format!("failed to store key: {e}")))?;
            }

            let pub_keys: Vec<_> = keys.iter().map(AuthSecretKey::public_key).collect();
            self.inner
                .register_account_public_key_commitments(&account.id(), &pub_keys)
                .await
                .map_err(|e| to_napi_err(e, "failed to map account to public keys"))?;

            Ok(format!("Imported account with ID: {account_id}"))
        })
    }

    /// Imports a note file.
    #[napi(js_name = "importNoteFile")]
    pub fn import_note_file(&mut self, note_file_bytes: Buffer) -> Result<NoteId> {
        use miden_client::notes::NoteFile;
        use miden_client::utils::Deserializable;

        let note_file = NoteFile::read_from_bytes(&note_file_bytes).map_err(|e| {
            napi::Error::from_reason(format!("Failed to deserialize note file: {e}"))
        })?;

        self.rt.block_on(async {
            let note_ids = self
                .inner
                .import_notes(&[note_file])
                .await
                .map_err(|e| to_napi_err(e, "failed to import note"))?;

            note_ids
                .first()
                .copied()
                .ok_or_else(|| napi::Error::from_reason("Note import did not return a note ID"))
                .map(Into::into)
        })
    }

    /// Imports a public account by ID from the network.
    #[napi(js_name = "importAccountById")]
    pub fn import_account_by_id(&mut self, account_id: &AccountId) -> Result<()> {
        self.rt.block_on(async {
            self.inner
                .import_account_by_id(account_id.0)
                .await
                .map_err(|e| to_napi_err(e, "failed to import public account"))
        })
    }

    // TRANSACTION HISTORY
    // --------------------------------------------------------------------------------------------

    /// Returns transactions matching the given filter.
    #[napi(js_name = "getTransactions")]
    pub fn get_transactions(&mut self, filter: String) -> Result<Vec<TransactionId>> {
        use miden_client::store::TransactionFilter;

        let native_filter = match filter.as_str() {
            "All" => TransactionFilter::All,
            "Uncommitted" => TransactionFilter::Uncommitted,
            _ => {
                return Err(napi::Error::from_reason(format!(
                    "Invalid transaction filter: {filter}. Expected 'All' or 'Uncommitted'"
                )));
            },
        };

        self.rt.block_on(async {
            let records = self
                .inner
                .get_transactions(native_filter)
                .await
                .map_err(|e| to_napi_err(e, "failed to get transactions"))?;
            Ok(records.into_iter().map(|r| r.id.into()).collect())
        })
    }

    /// Ensures the genesis block has been fetched and stored.
    #[napi(js_name = "ensureGenesisInPlace")]
    pub fn ensure_genesis_in_place(&mut self) -> Result<()> {
        self.rt.block_on(async {
            self.inner
                .ensure_genesis_in_place()
                .await
                .map_err(|e| to_napi_err(e, "failed to ensure genesis in place"))?;
            Ok(())
        })
    }
}

// HELPERS
// ================================================================================================

use miden_client::Deserializable;

fn generate_wallet(
    storage_mode: &AccountStorageMode,
    mutable: bool,
    seed: Option<Buffer>,
    auth_scheme: &AuthScheme,
) -> Result<(miden_client::account::Account, AuthSecretKey)> {
    use miden_client::account::component::{AccountComponent, BasicWallet};
    use miden_client::account::{AccountBuilder, AccountType};

    let mut rng = match seed {
        Some(seed_bytes) => {
            let seed_array: [u8; 32] = seed_bytes[..]
                .try_into()
                .map_err(|_| napi::Error::from_reason("Seed must be exactly 32 bytes"))?;
            StdRng::from_seed(seed_array)
        },
        None => StdRng::from_os_rng(),
    };

    let native_scheme: NativeAuthSchemeId = auth_scheme.into();
    let (key_pair, auth_component) = match native_scheme {
        NativeAuthSchemeId::Falcon512Rpo => {
            let kp = AuthSecretKey::new_falcon512_rpo_with_rng(&mut rng);
            let comp: AccountComponent =
                AuthFalcon512Rpo::new(kp.public_key().to_commitment()).into();
            (kp, comp)
        },
        NativeAuthSchemeId::EcdsaK256Keccak => {
            let kp = AuthSecretKey::new_ecdsa_k256_keccak_with_rng(&mut rng);
            let comp: AccountComponent =
                AuthEcdsaK256Keccak::new(kp.public_key().to_commitment()).into();
            (kp, comp)
        },
        _ => {
            return Err(napi::Error::from_reason(format!(
                "unsupported auth scheme: {native_scheme:?}"
            )));
        },
    };

    let account_type = if mutable {
        AccountType::RegularAccountUpdatableCode
    } else {
        AccountType::RegularAccountImmutableCode
    };

    let mut init_seed = [0u8; 32];
    rng.fill_bytes(&mut init_seed);

    let native_storage_mode: miden_client::account::AccountStorageMode = storage_mode.into();

    let new_account = AccountBuilder::new(init_seed)
        .account_type(account_type)
        .storage_mode(native_storage_mode)
        .with_auth_component(auth_component)
        .with_component(BasicWallet)
        .build()
        .map_err(|e| napi::Error::from_reason(format!("failed to create new wallet: {e:?}")))?;

    Ok((new_account, key_pair))
}

fn parse_note_type(s: &str) -> Result<miden_client::note::NoteType> {
    match s {
        "Public" | "public" => Ok(miden_client::note::NoteType::Public),
        "Private" | "private" => Ok(miden_client::note::NoteType::Private),
        "Encrypted" | "encrypted" => Ok(miden_client::note::NoteType::Encrypted),
        other => Err(napi::Error::from_reason(format!(
            "Invalid note type: {other}. Expected 'Public', 'Private', or 'Encrypted'"
        ))),
    }
}

fn bigint_to_u64(value: &BigInt, name: &str) -> Result<u64> {
    let (sign_bit, val, lossless) = value.get_u64();
    if !lossless || sign_bit {
        return Err(napi::Error::from_reason(format!(
            "{name} must be a non-negative integer that fits in u64"
        )));
    }
    Ok(val)
}
