use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_objects::account::{Account, AccountId};
use miden_objects::asset::{Asset, NonFungibleAsset};
use miden_objects::block::BlockNumber;
use miden_objects::note::{NoteDetails, NoteId, NoteRecipient, NoteTag};
use miden_objects::transaction::{
    AccountInputs, ExecutedTransaction, InputNote, InputNotes, ProvenTransaction, TransactionArgs,
};
use miden_objects::{AssetError, Word};
use miden_tx::auth::TransactionAuthenticator;
use miden_tx::{NoteConsumptionChecker, TransactionExecutor};

use crate::rpc::NodeRpcClient;
use crate::store::data_store::ClientDataStore;
use crate::transaction::{
    TransactionProver, TransactionRequest, TransactionScriptTemplate, TransactionStoreUpdate,
};
use crate::{ClientError, DebugMode};

#[derive(Clone)]
pub struct TransactionPipeline {
    /// The RPC client used to communicate with the node.
    rpc_api: Arc<dyn NodeRpcClient + Send>,
    /// Maximum number of blocks the client can be behind the network for transactions and account
    /// proofs to be considered valid.
    max_block_number_delta: Option<u32>,
    /// Indicates whether scripts should be assembled in debug mode or not.
    debug_mode: DebugMode,
    /// Future notes that are expected to be created as a result of the transaction.
    future_notes: Vec<(NoteDetails, NoteTag)>,
}

impl TransactionPipeline {
    pub fn new(
        rpc_api: Arc<dyn NodeRpcClient + Send>,
        max_block_number_delta: Option<u32>,
        debug_mode: DebugMode,
    ) -> Self {
        Self {
            rpc_api,
            max_block_number_delta,
            debug_mode,
            future_notes: Vec::new(),
        }
    }

    /// Creates and executes a transaction specified by the request against the specified account,
    /// but doesn't change the local database.
    ///
    /// If the transaction utilizes foreign account data, there is a chance that the client doesn't
    /// have the required block header in the local database. In these scenarios, a sync to
    /// the chain tip is performed, and the required block header is retrieved.
    ///
    /// # Errors
    ///
    /// - Returns [`ClientError::MissingOutputRecipients`] if the [`TransactionRequest`] output
    ///   notes are not a subset of executor's output notes.
    /// - Returns a [`ClientError::TransactionExecutorError`] if the execution fails.
    /// - Returns a [`ClientError::TransactionRequestError`] if the request is invalid.
    pub async fn execute_transaction(
        &mut self,
        account: Account,
        transaction_request: TransactionRequest,
        foreign_account_inputs: Vec<AccountInputs>,
        mut input_notes: InputNotes<InputNote>,
        data_store: &ClientDataStore,
        executor: &TransactionExecutor<
            '_,
            '_,
            ClientDataStore,
            impl TransactionAuthenticator + Sync,
        >,
        block_ref: BlockNumber,
    ) -> Result<ExecutedTransaction, ClientError> {
        // Validates the transaction request before executing
        self.validate_request(&account, &transaction_request, block_ref, &input_notes)
            .await?;

        let output_recipients =
            transaction_request.expected_output_recipients().cloned().collect::<Vec<_>>();

        let future_notes: Vec<(NoteDetails, NoteTag)> =
            transaction_request.expected_future_notes().cloned().collect();

        let tx_script =
            transaction_request.build_transaction_script(&(&account).into(), self.debug_mode)?;

        let ignore_invalid_notes = transaction_request.ignore_invalid_input_notes();

        for fpi_account in &foreign_account_inputs {
            data_store.mast_store().load_account_code(fpi_account.code());
        }

        let tx_args = transaction_request.into_transaction_args(tx_script, foreign_account_inputs);

        data_store.mast_store().load_account_code(account.code());

        if ignore_invalid_notes {
            // Remove invalid notes
            input_notes = get_valid_input_notes(
                account.id(),
                input_notes,
                tx_args.clone(),
                executor,
                &block_ref,
            )
            .await?;
        }

        // Execute the transaction and get the witness
        let executed_transaction = executor
            .execute_transaction(account.id(), block_ref, input_notes, tx_args)
            .await?;

        validate_executed_transaction(&executed_transaction, &output_recipients)?;

        self.future_notes = future_notes;

        Ok(executed_transaction)
    }

    /// Validates that the specified transaction request can be executed by the specified account.
    ///
    /// This does't guarantee that the transaction will succeed, but it's useful to avoid submitting
    /// transactions that are guaranteed to fail. Some of the validations include:
    /// - That the account has enough balance to cover the outgoing assets.
    /// - That the reference block is not too far behind the chain tip.
    pub async fn validate_request(
        &mut self,
        account: &Account,
        transaction_request: &TransactionRequest,
        block_ref: BlockNumber,
        input_notes: &InputNotes<InputNote>,
    ) -> Result<(), ClientError> {
        if let Some(max_block_number_delta) = self.max_block_number_delta {
            let current_chain_tip =
                self.rpc_api.get_block_header_by_number(None, false).await?.0.block_num();

            if current_chain_tip > block_ref + max_block_number_delta {
                return Err(ClientError::RecencyConditionError(
                    "The reference block is too far behind the chain tip to execute the transaction"
                        .to_string(),
                ));
            }
        }

        if account.is_faucet() {
            // TODO(SantiagoPittella): Add faucet validations.
            Ok(())
        } else {
            validate_basic_account_request(transaction_request, account, input_notes)
        }
    }

    pub async fn prove_transaction(
        &self,
        executed_transaction: ExecutedTransaction,
        prover: Arc<dyn TransactionProver + Send + Sync>,
    ) -> Result<ProvenTransaction, ClientError> {
        Ok(prover.prove(executed_transaction.into()).await?)
    }

    pub async fn submit_proven_transaction(
        &self,
        proven_transaction: ProvenTransaction,
    ) -> Result<BlockNumber, ClientError> {
        Ok(self.rpc_api.submit_proven_transaction(proven_transaction).await?)
    }

    pub fn get_transaction_update(
        self,
        submission_height: BlockNumber,
        executed_transaction: ExecutedTransaction,
    ) -> TransactionStoreUpdate {
        TransactionStoreUpdate {
            submission_height,
            executed_transaction,
            future_notes: self.future_notes,
        }
    }
}
// HELPERS
// ================================================================================================

fn validate_basic_account_request(
    transaction_request: &TransactionRequest,
    account: &Account,
    input_notes: &InputNotes<InputNote>,
) -> Result<(), ClientError> {
    // Get outgoing assets
    let (fungible_balance_map, non_fungible_set) = get_outgoing_assets(transaction_request);

    // Get incoming assets
    let (incoming_fungible_balance_map, incoming_non_fungible_balance_set) =
        collect_assets(input_notes.iter().flat_map(|note| note.note().assets().iter()));

    // Check if the account balance plus incoming assets is greater than or equal to the
    // outgoing fungible assets
    for (faucet_id, amount) in fungible_balance_map {
        let account_asset_amount = account.vault().get_balance(faucet_id).unwrap_or(0);
        let incoming_balance = incoming_fungible_balance_map.get(&faucet_id).unwrap_or(&0);
        if account_asset_amount + incoming_balance < amount {
            return Err(ClientError::AssetError(AssetError::FungibleAssetAmountNotSufficient {
                minuend: account_asset_amount,
                subtrahend: amount,
            }));
        }
    }

    // Check if the account balance plus incoming assets is greater than or equal to the
    // outgoing non fungible assets
    for non_fungible in non_fungible_set {
        match account.vault().has_non_fungible_asset(non_fungible) {
            Ok(true) => (),
            Ok(false) => {
                // Check if the non fungible asset is in the incoming assets
                if !incoming_non_fungible_balance_set.contains(&non_fungible) {
                    return Err(ClientError::AssetError(
                        AssetError::NonFungibleFaucetIdTypeMismatch(
                            non_fungible.faucet_id_prefix(),
                        ),
                    ));
                }
            },
            _ => {
                return Err(ClientError::AssetError(AssetError::NonFungibleFaucetIdTypeMismatch(
                    non_fungible.faucet_id_prefix(),
                )));
            },
        }
    }

    Ok(())
}

async fn get_valid_input_notes(
    account_id: AccountId,
    mut input_notes: InputNotes<InputNote>,
    tx_args: TransactionArgs,
    executor: &TransactionExecutor<'_, '_, ClientDataStore, impl TransactionAuthenticator + Sync>,
    block_ref: &BlockNumber,
) -> Result<InputNotes<InputNote>, ClientError> {
    loop {
        let execution = NoteConsumptionChecker::new(executor)
            .check_notes_consumability(account_id, *block_ref, input_notes.clone(), tx_args.clone())
            .await?;

        if execution.failed.is_empty() {
            break;
        }

        let failed_note_ids: BTreeSet<NoteId> =
            execution.failed.iter().map(|n| n.note.id()).collect();
        let filtered_input_notes = InputNotes::new(
            input_notes
                .into_iter()
                .filter(|note| !failed_note_ids.contains(&note.id()))
                .collect(),
        )
        .expect("Created from a valid input notes list");

        input_notes = filtered_input_notes;
    }

    Ok(input_notes)
}

/// Validates that the executed transaction's output recipients match what was expected in the
/// transaction request.
fn validate_executed_transaction(
    executed_transaction: &ExecutedTransaction,
    expected_output_recipients: &[NoteRecipient],
) -> Result<(), ClientError> {
    let tx_output_recipient_digests = executed_transaction
        .output_notes()
        .iter()
        .filter_map(|n| n.recipient().map(NoteRecipient::digest))
        .collect::<Vec<_>>();

    let missing_recipient_digest: Vec<Word> = expected_output_recipients
        .iter()
        .filter_map(|recipient| {
            (!tx_output_recipient_digests.contains(&recipient.digest()))
                .then_some(recipient.digest())
        })
        .collect();

    if !missing_recipient_digest.is_empty() {
        return Err(ClientError::MissingOutputRecipients(missing_recipient_digest));
    }

    Ok(())
}

/// Helper to get the account outgoing assets.
///
/// Any outgoing assets resulting from executing note scripts but not present in expected output
/// notes wouldn't be included.
fn get_outgoing_assets(
    transaction_request: &TransactionRequest,
) -> (BTreeMap<AccountId, u64>, BTreeSet<NonFungibleAsset>) {
    // Get own notes assets
    let mut own_notes_assets = match transaction_request.script_template() {
        Some(TransactionScriptTemplate::SendNotes(notes)) => notes
            .iter()
            .map(|note| (note.id(), note.assets().clone()))
            .collect::<BTreeMap<_, _>>(),
        _ => BTreeMap::default(),
    };
    // Get transaction output notes assets
    let mut output_notes_assets = transaction_request
        .expected_output_own_notes()
        .into_iter()
        .map(|note| (note.id(), note.assets().clone()))
        .collect::<BTreeMap<_, _>>();

    // Merge with own notes assets and delete duplicates
    output_notes_assets.append(&mut own_notes_assets);

    // Create a map of the fungible and non-fungible assets in the output notes
    let outgoing_assets = output_notes_assets.values().flat_map(|note_assets| note_assets.iter());

    collect_assets(outgoing_assets)
}

fn collect_assets<'a>(
    assets: impl Iterator<Item = &'a Asset>,
) -> (BTreeMap<AccountId, u64>, BTreeSet<NonFungibleAsset>) {
    let mut fungible_balance_map = BTreeMap::new();
    let mut non_fungible_set = BTreeSet::new();

    assets.for_each(|asset| match asset {
        Asset::Fungible(fungible) => {
            fungible_balance_map
                .entry(fungible.faucet_id())
                .and_modify(|balance| *balance += fungible.amount())
                .or_insert(fungible.amount());
        },
        Asset::NonFungible(non_fungible) => {
            non_fungible_set.insert(*non_fungible);
        },
    });

    (fungible_balance_map, non_fungible_set)
}
