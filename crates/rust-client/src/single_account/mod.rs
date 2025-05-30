use core::time::Duration;
use std::{boxed::Box, path::PathBuf, sync::Arc, vec::Vec};

use miden_lib::account::{auth::RpoFalcon512, faucets::BasicFungibleFaucet, wallets::BasicWallet};
use miden_objects::{
    Digest, Felt, Word,
    account::{
        Account, AccountBuilder, AccountId, AccountIdAnchor, AccountStorageMode, AccountType,
        AuthSecretKey,
    },
    asset::{Asset, FungibleAsset, TokenSymbol},
    block::BlockNumber,
    crypto::{dsa::rpo_falcon512::SecretKey, rand::RpoRandomCoin},
    note::{NoteFile, NoteType},
    transaction::OutputNote,
};
use rand::{Rng, RngCore};

use crate::{
    Client, ClientError, ClientRng,
    builder::ClientBuilder,
    keystore::FilesystemKeyStore,
    rpc::{Endpoint, TonicRpcClient},
    store::{
        AccountRecord, InputNoteRecord, NoteExportType, NoteFilter, TransactionFilter,
        sqlite_store::SqliteStore,
    },
    sync::SyncSummary,
    transaction::{
        PaymentTransactionData, TransactionRequest, TransactionRequestBuilder, TransactionStatus,
    },
};

pub struct SingleAccountClient {
    client: Client,
    account_id: AccountId,
}

impl SingleAccountClient {
    pub async fn new(
        account: Account,
        seed: Option<Word>,
        key_pair: SecretKey,
        directory: PathBuf,
        node_endpoint: Endpoint,
    ) -> Result<Self, ClientError> {
        let store_filepath = directory.join("store.sqlite3");
        let store = {
            let sqlite_store = SqliteStore::new(store_filepath).await.unwrap();
            Arc::new(sqlite_store)
        };

        let mut rng = rand::rng();
        let coin_seed: [u64; 4] = rng.random();

        let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));

        let keystore = FilesystemKeyStore::new(directory.clone()).unwrap();

        let mut client = ClientBuilder::new()
            .with_rpc(Arc::new(TonicRpcClient::new(&node_endpoint, 10_000)))
            .with_rng(Box::new(rng))
            .with_store(store)
            .with_filesystem_keystore(directory.to_str().unwrap())
            .in_debug_mode(true)
            .with_tx_graceful_blocks(None)
            .build()
            .await
            .unwrap();

        client.sync_state().await?;
        client.add_account(&account, seed, false).await.unwrap();
        keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair)).unwrap();

        Ok(Self { client, account_id: account.id() })
    }

    pub async fn sync_state(&mut self) -> Result<SyncSummary, ClientError> {
        self.client.sync_state().await
    }

    pub async fn get_account(&self) -> Result<AccountRecord, ClientError> {
        let account = self.client.get_account(self.account_id).await?;

        Ok(account.expect("Account should be present"))
    }

    pub async fn get_input_notes(&self) -> Result<Vec<InputNoteRecord>, ClientError> {
        self.client.get_input_notes(NoteFilter::All).await
    }

    pub async fn import_notes(&mut self, note_files: Vec<NoteFile>) -> Result<(), ClientError> {
        for note_file in note_files {
            self.client.import_note(note_file).await?;
        }

        Ok(())
    }

    pub async fn execute_tx_and_sync(
        &mut self,
        transaction_request: TransactionRequest,
    ) -> Result<Vec<NoteFile>, ClientError> {
        // Ensure the client is synced before executing the transaction
        self.client.sync_state().await?;

        let tx_result = self.client.new_transaction(self.account_id, transaction_request).await?;
        let transaction_id = tx_result.executed_transaction().id();
        let output_notes = tx_result
            .executed_transaction()
            .output_notes()
            .iter()
            .map(OutputNote::id)
            .collect::<Vec<_>>();

        self.client.submit_transaction(tx_result).await?;

        // TODO: Use wait_for_tx
        loop {
            self.client.sync_state().await.unwrap();

            // Check if executed transaction got committed by the node
            let tracked_transaction = self
                .client
                .get_transactions(TransactionFilter::Ids(vec![transaction_id]))
                .await
                .unwrap()
                .pop()
                .unwrap();

            match tracked_transaction.status {
                TransactionStatus::Committed(_) => {
                    break;
                },
                TransactionStatus::Pending => {
                    std::thread::sleep(Duration::from_secs(1));
                },
                TransactionStatus::Discarded(cause) => {
                    panic!("Transaction was discarded with cause: {:?}", cause);
                },
            }
        }

        let note_files = self
            .client
            .get_output_notes(NoteFilter::List(output_notes))
            .await?
            .into_iter()
            .map(|note| note.into_note_file(&NoteExportType::NoteWithProof))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(note_files)
    }

    pub fn rng(&mut self) -> &mut ClientRng {
        self.client.rng()
    }
}

pub struct BasicWalletClient {
    client: SingleAccountClient,
}

impl BasicWalletClient {
    pub async fn new(
        anchor_id: AccountIdAnchor,
        storage_mode: AccountStorageMode,
        directory: PathBuf,
        node_endpoint: Endpoint,
    ) -> Result<Self, ClientError> {
        let mut rng = rand::rng();

        let key_pair = SecretKey::with_rng(&mut rng);
        let pub_key = key_pair.public_key();

        let mut init_seed = [0u8; 32];
        rng.fill_bytes(&mut init_seed);

        let (account, seed) = AccountBuilder::new(init_seed)
            .anchor(anchor_id)
            .account_type(AccountType::RegularAccountImmutableCode)
            .storage_mode(storage_mode)
            .with_component(RpoFalcon512::new(pub_key))
            .with_component(BasicWallet)
            .build()
            .unwrap();

        let client =
            SingleAccountClient::new(account, Some(seed), key_pair, directory, node_endpoint)
                .await?;
        Ok(Self { client })
    }

    pub fn account_id(&self) -> AccountId {
        self.client.account_id
    }

    pub fn inner_client(&self) -> &SingleAccountClient {
        &self.client
    }

    pub async fn send_asset(
        &mut self,
        target: AccountId,
        asset: FungibleAsset,
        recall_height: Option<BlockNumber>,
        note_type: NoteType,
    ) -> Result<Vec<NoteFile>, ClientError> {
        let transaction_request = TransactionRequestBuilder::new().build_pay_to_id(
            PaymentTransactionData::new(
                vec![Asset::Fungible(asset)],
                self.client.account_id,
                target,
            ),
            recall_height,
            note_type,
            self.client.rng(),
        )?;

        self.client.execute_tx_and_sync(transaction_request).await
    }

    pub async fn receive_assets(&mut self, notes: Vec<NoteFile>) -> Result<(), ClientError> {
        let note_ids = notes
            .iter()
            .map(|note| match note {
                // TODO: Implement this on miden-base
                NoteFile::NoteId(note_id) => note_id.clone(),
                NoteFile::NoteDetails { details, .. } => details.id(),
                NoteFile::NoteWithProof(note, _) => note.id(),
            })
            .collect::<Vec<_>>();

        self.client.import_notes(notes).await?;

        let transaction_request = TransactionRequestBuilder::new().build_consume_notes(note_ids)?;

        self.client.execute_tx_and_sync(transaction_request).await?;

        Ok(())
    }
}

pub struct FungibleFaucetClient {
    client: SingleAccountClient,
}

impl FungibleFaucetClient {
    pub async fn new(
        anchor_id: AccountIdAnchor,
        storage_mode: AccountStorageMode,
        directory: PathBuf,
        node_endpoint: Endpoint,
    ) -> Result<Self, ClientError> {
        let mut rng = rand::rng();

        let key_pair = SecretKey::with_rng(&mut rng);
        let pub_key = key_pair.public_key();

        let mut init_seed = [0u8; 32];
        rng.fill_bytes(&mut init_seed);

        let symbol = TokenSymbol::new("TEST").unwrap();
        let max_supply = Felt::try_from(9_999_999_u64.to_le_bytes().as_slice())
            .expect("u64 can be safely converted to a field element");

        let (account, seed) = AccountBuilder::new(init_seed)
            .anchor(anchor_id)
            .account_type(AccountType::FungibleFaucet)
            .storage_mode(storage_mode)
            .with_component(RpoFalcon512::new(pub_key))
            .with_component(BasicFungibleFaucet::new(symbol, 10, max_supply).unwrap())
            .build()
            .unwrap();

        let client =
            SingleAccountClient::new(account, Some(seed), key_pair, directory, node_endpoint)
                .await?;
        Ok(Self { client })
    }

    pub fn account_id(&self) -> AccountId {
        self.client.account_id
    }

    pub fn inner_client(&self) -> &SingleAccountClient {
        &self.client
    }

    pub async fn mint_assets(
        &mut self,
        target: AccountId,
        amount: u64,
        note_type: NoteType,
    ) -> Result<Vec<NoteFile>, ClientError> {
        let transaction_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
            FungibleAsset::new(self.client.account_id, amount).unwrap(),
            target,
            note_type,
            self.client.rng(),
        )?;

        self.client.execute_tx_and_sync(transaction_request).await
    }
}
