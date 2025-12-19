//! Provides APIs for creating, executing, proving, and submitting transactions to the Miden
//! network.
//!
//! ## Overview
//!
//! This module enables clients to:
//!
//! - Build transaction requests using the [`TransactionRequestBuilder`].
//!   - [`TransactionRequestBuilder`] contains simple builders for standard transaction types, such
//!     as `p2id` (pay-to-id)
//! - Execute transactions via the local transaction executor and generate a [`TransactionResult`]
//!   that includes execution details and relevant notes for state tracking.
//! - Prove transactions (locally or remotely) using a [`TransactionProver`] and submit the proven
//!   transactions to the network.
//! - Track and update the state of transactions, including their status (e.g., `Pending`,
//!   `Committed`, or `Discarded`).
//!
//! ## Example
//!
//! The following example demonstrates how to create and submit a transaction:
//!
//! ```rust
//! use miden_client::Client;
//! use miden_client::auth::TransactionAuthenticator;
//! use miden_client::crypto::FeltRng;
//! use miden_client::transaction::{PaymentNoteDescription, TransactionRequestBuilder};
//! use miden_objects::account::AccountId;
//! use miden_objects::asset::FungibleAsset;
//! use miden_objects::note::NoteType;
//! # use std::error::Error;
//!
//! /// Executes, proves and submits a P2ID transaction.
//! ///
//! /// This transaction is executed by `sender_id`, and creates an output note
//! /// containing 100 tokens of `faucet_id`'s fungible asset.
//! async fn create_and_submit_transaction<
//!     R: rand::Rng,
//!     AUTH: TransactionAuthenticator + Sync + 'static,
//! >(
//!     client: &mut Client<AUTH>,
//!     sender_id: AccountId,
//!     target_id: AccountId,
//!     faucet_id: AccountId,
//! ) -> Result<(), Box<dyn Error>> {
//!     // Create an asset representing the amount to be transferred.
//!     let asset = FungibleAsset::new(faucet_id, 100)?;
//!
//!     // Build a transaction request for a pay-to-id transaction.
//!     let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
//!         PaymentNoteDescription::new(vec![asset.into()], sender_id, target_id),
//!         NoteType::Private,
//!         client.rng(),
//!     )?;
//!
//!     // Execute, prove, and submit the transaction in a single call.
//!     let _tx_id = client.submit_new_transaction(sender_id, tx_request).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! For more detailed information about each function and error type, refer to the specific API
//! documentation.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_objects::account::{Account, AccountId};
use miden_objects::asset::{Asset, NonFungibleAsset};
use miden_objects::block::BlockNumber;
use miden_objects::note::{Note, NoteDetails, NoteId, NoteRecipient, NoteScript, NoteTag};
use miden_objects::transaction::AccountInputs;
use miden_objects::{AssetError, Felt, Word};
use miden_tx::{DataStore, NoteConsumptionChecker, TransactionExecutor};
use tracing::info;

use super::Client;
use crate::ClientError;
use crate::note::{NoteScreener, NoteUpdateTracker};
use crate::rpc::AccountStateAt;
use crate::store::data_store::ClientDataStore;
use crate::store::input_note_states::ExpectedNoteState;
use crate::store::{
    InputNoteRecord,
    InputNoteState,
    NoteFilter,
    OutputNoteRecord,
    StoreError,
    TransactionFilter,
};
use crate::sync::NoteTagRecord;

mod prover;
pub use prover::TransactionProver;

mod record;
pub use record::{
    DiscardCause,
    TransactionDetails,
    TransactionRecord,
    TransactionStatus,
    TransactionStatusVariant,
};

mod store_update;
pub use store_update::TransactionStoreUpdate;

mod request;
pub use request::{
    ForeignAccount,
    NoteArgs,
    PaymentNoteDescription,
    SwapTransactionData,
    TransactionRequest,
    TransactionRequestBuilder,
    TransactionRequestError,
    TransactionScriptTemplate,
};

mod result;
// RE-EXPORTS
// ================================================================================================
pub use miden_lib::account::interface::{AccountComponentInterface, AccountInterface};
pub use miden_lib::transaction::TransactionKernel;
pub use miden_objects::transaction::{
    ExecutedTransaction,
    InputNote,
    InputNotes,
    OutputNote,
    OutputNotes,
    ProvenTransaction,
    TransactionArgs,
    TransactionId,
    TransactionInputs,
    TransactionScript,
    TransactionSummary,
};
pub use miden_objects::vm::{AdviceInputs, AdviceMap};
pub use miden_tx::auth::TransactionAuthenticator;
pub use miden_tx::{
    DataStoreError,
    LocalTransactionProver,
    ProvingOptions,
    TransactionExecutorError,
    TransactionProverError,
};
pub use result::TransactionResult;

/// Transaction management methods
impl<AUTH> Client<AUTH>
where
    AUTH: TransactionAuthenticator + Sync + 'static,
{
    // TRANSACTION DATA RETRIEVAL
    // --------------------------------------------------------------------------------------------

    /// Retrieves tracked transactions, filtered by [`TransactionFilter`].
    pub async fn get_transactions(
        &self,
        filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, ClientError> {
        self.store.get_transactions(filter).await.map_err(Into::into)
    }

    // TRANSACTION
    // --------------------------------------------------------------------------------------------

    /// Executes a transaction specified by the request against the specified account,
    /// proves it, submits it to the network, and updates the local database.
    ///
    /// If the transaction utilizes foreign account data, there is a chance that the client
    /// doesn't have the required block header in the local database. In these scenarios, a sync to
    /// the chain tip is performed, and the required block header is retrieved.
    pub async fn submit_new_transaction(
        &mut self,
        account_id: AccountId,
        transaction_request: TransactionRequest,
    ) -> Result<TransactionId, ClientError> {
        let tx_result = self.execute_transaction(account_id, transaction_request).await?;
        let tx_id = tx_result.executed_transaction().id();

        let proven_transaction = self.prove_transaction(&tx_result).await?;
        let submission_height =
            self.submit_proven_transaction(proven_transaction, &tx_result).await?;

        self.apply_transaction(&tx_result, submission_height).await?;

        Ok(tx_id)
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
        account_id: AccountId,
        transaction_request: TransactionRequest,
    ) -> Result<TransactionResult, ClientError> {
        // Validates the transaction request before executing
        self.validate_request(account_id, &transaction_request).await?;

        // Retrieve all input notes from the store.
        // But only mark as authenticated if they are committed or consumed.
        let mut authenticated_note_records = self
            .store
            .get_input_notes(NoteFilter::List(transaction_request.get_input_note_ids()))
            .await?;
        authenticated_note_records.retain(|note| {
            matches!(
                note.state(),
                InputNoteState::Committed(_)
                    | InputNoteState::ConsumedAuthenticatedLocal(_)
                    | InputNoteState::ConsumedExternal(_)
            )
        });

        let authenticated_note_ids =
            authenticated_note_records.iter().map(InputNoteRecord::id).collect::<Vec<_>>();

        // If tx request contains unauthenticated_input_notes we should insert them
        let unauthenticated_input_notes = transaction_request
            .input_notes()
            .iter()
            .filter(|n| !authenticated_note_ids.contains(&n.id()))
            .cloned()
            .map(Into::into)
            .collect::<Vec<_>>();

        self.store.upsert_input_notes(&unauthenticated_input_notes).await?;

        let mut notes = transaction_request.build_input_notes(authenticated_note_records)?;

        let output_recipients =
            transaction_request.expected_output_recipients().cloned().collect::<Vec<_>>();

        let future_notes: Vec<(NoteDetails, NoteTag)> =
            transaction_request.expected_future_notes().cloned().collect();

        let tx_script = transaction_request.build_transaction_script(
            &self.get_account_interface(account_id).await?,
            self.in_debug_mode().into(),
        )?;

        let foreign_accounts = transaction_request.foreign_accounts().clone();

        // Inject state and code of foreign accounts
        let (fpi_block_num, foreign_account_inputs) =
            self.retrieve_foreign_account_inputs(foreign_accounts).await?;

        let ignore_invalid_notes = transaction_request.ignore_invalid_input_notes();

        let data_store = ClientDataStore::new(self.store.clone());
        data_store.register_foreign_account_inputs(foreign_account_inputs.iter().cloned());
        for fpi_account in &foreign_account_inputs {
            data_store.mast_store().load_account_code(fpi_account.code());
        }

        // Upsert note scripts for later retrieval from the client's DataStore
        let output_note_scripts: Vec<NoteScript> = transaction_request
            .expected_output_recipients()
            .map(|n| n.script().clone())
            .collect();
        self.store.upsert_note_scripts(&output_note_scripts).await?;

        let block_num = if let Some(block_num) = fpi_block_num {
            block_num
        } else {
            self.store.get_sync_height().await?
        };

        // Load account code into MAST forest store
        // TODO: Refactor this to get account code only?
        let account_record = self
            .store
            .get_account(account_id)
            .await?
            .ok_or(ClientError::AccountDataNotFound(account_id))?;
        let account: Account = account_record.try_into()?;
        data_store.mast_store().load_account_code(account.code());

        // Get transaction args
        let tx_args = transaction_request.into_transaction_args(tx_script);

        if ignore_invalid_notes {
            // Remove invalid notes
            notes = self.get_valid_input_notes(account, notes, tx_args.clone()).await?;
        }

        // Execute the transaction and get the witness
        let executed_transaction = self
            .build_executor(&data_store)?
            .execute_transaction(account_id, block_num, notes, tx_args)
            .await?;

        validate_executed_transaction(&executed_transaction, &output_recipients)?;
        TransactionResult::new(executed_transaction, future_notes)
    }

    /// Proves the specified transaction using the prover configured for this client.
    pub async fn prove_transaction(
        &mut self,
        tx_result: &TransactionResult,
    ) -> Result<ProvenTransaction, ClientError> {
        self.prove_transaction_with(tx_result, self.tx_prover.clone()).await
    }

    /// Proves the specified transaction using the provided prover.
    pub async fn prove_transaction_with(
        &mut self,
        tx_result: &TransactionResult,
        tx_prover: Arc<dyn TransactionProver>,
    ) -> Result<ProvenTransaction, ClientError> {
        info!("Proving transaction...");

        let proven_transaction =
            tx_prover.prove(tx_result.executed_transaction().clone().into()).await?;

        info!("Transaction proven.");

        Ok(proven_transaction)
    }

    /// Submits a previously proven transaction to the RPC endpoint and returns the node’s chain tip
    /// upon mempool admission.
    pub async fn submit_proven_transaction(
        &mut self,
        proven_transaction: ProvenTransaction,
        transaction_inputs: impl Into<TransactionInputs>,
    ) -> Result<BlockNumber, ClientError> {
        info!("Submitting transaction to the network...");
        let block_num = self
            .rpc_api
            .submit_proven_transaction(proven_transaction, transaction_inputs.into())
            .await?;
        info!("Transaction submitted.");

        Ok(block_num)
    }

    /// Builds a [`TransactionStoreUpdate`] for the provided transaction result at the specified
    /// submission height.
    pub async fn get_transaction_store_update(
        &self,
        tx_result: &TransactionResult,
        submission_height: BlockNumber,
    ) -> Result<TransactionStoreUpdate, ClientError> {
        let note_updates = self.get_note_updates(submission_height, tx_result).await?;

        let new_tags = note_updates
            .updated_input_notes()
            .filter_map(|note| {
                let note = note.inner();

                if let InputNoteState::Expected(ExpectedNoteState { tag: Some(tag), .. }) =
                    note.state()
                {
                    Some(NoteTagRecord::with_note_source(*tag, note.id()))
                } else {
                    None
                }
            })
            .collect();

        Ok(TransactionStoreUpdate::new(
            tx_result.executed_transaction().clone(),
            submission_height,
            note_updates,
            tx_result.future_notes().to_vec(),
            new_tags,
        ))
    }

    /// Persists the effects of a submitted transaction into the local store,
    /// updating account data, note metadata, and future note tracking.
    pub async fn apply_transaction(
        &self,
        tx_result: &TransactionResult,
        submission_height: BlockNumber,
    ) -> Result<(), ClientError> {
        let tx_update = self.get_transaction_store_update(tx_result, submission_height).await?;

        self.apply_transaction_update(tx_update).await
    }

    pub async fn apply_transaction_update(
        &self,
        tx_update: TransactionStoreUpdate,
    ) -> Result<(), ClientError> {
        // Transaction was proven and submitted to the node correctly, persist note details and
        // update account
        info!("Applying transaction to the local store...");

        let executed_transaction = tx_update.executed_transaction();
        let account_id = executed_transaction.account_id();
        let account_record = self.try_get_account(account_id).await?;

        if account_record.is_locked() {
            return Err(ClientError::AccountLocked(account_id));
        }

        let final_commitment = executed_transaction.final_account().commitment();
        if self.store.get_account_header_by_commitment(final_commitment).await?.is_some() {
            return Err(ClientError::StoreError(StoreError::AccountCommitmentAlreadyExists(
                final_commitment,
            )));
        }

        self.store.apply_transaction(tx_update).await?;
        info!("Transaction stored.");
        Ok(())
    }

    /// Executes the provided transaction script against the specified account, and returns the
    /// resulting stack. Advice inputs and foreign accounts can be provided for the execution.
    ///
    /// The transaction will use the current sync height as the block reference.
    pub async fn execute_program(
        &mut self,
        account_id: AccountId,
        tx_script: TransactionScript,
        advice_inputs: AdviceInputs,
        foreign_accounts: BTreeSet<ForeignAccount>,
    ) -> Result<[Felt; 16], ClientError> {
        let (fpi_block_number, foreign_account_inputs) =
            self.retrieve_foreign_account_inputs(foreign_accounts).await?;

        let block_ref = if let Some(block_number) = fpi_block_number {
            block_number
        } else {
            self.get_sync_height().await?
        };

        let account_record = self
            .store
            .get_account(account_id)
            .await?
            .ok_or(ClientError::AccountDataNotFound(account_id))?;

        let account: Account = account_record.try_into()?;

        let data_store = ClientDataStore::new(self.store.clone());

        data_store.register_foreign_account_inputs(foreign_account_inputs.iter().cloned());

        // Ensure code is loaded on MAST store
        data_store.mast_store().load_account_code(account.code());

        for fpi_account in &foreign_account_inputs {
            data_store.mast_store().load_account_code(fpi_account.code());
        }

        Ok(self
            .build_executor(&data_store)?
            .execute_tx_view_script(account_id, block_ref, tx_script, advice_inputs)
            .await?)
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

    /// Compiles the note updates needed to be applied to the store after executing a
    /// transaction.
    ///
    /// These updates include:
    /// - New output notes.
    /// - New input notes (only if they are relevant to the client).
    /// - Input notes that could be created as outputs of future transactions (e.g., a SWAP payback
    ///   note).
    /// - Updated input notes that were consumed locally.
    async fn get_note_updates(
        &self,
        submission_height: BlockNumber,
        tx_result: &TransactionResult,
    ) -> Result<NoteUpdateTracker, ClientError> {
        let executed_tx = tx_result.executed_transaction();
        let current_timestamp = self.store.get_current_timestamp();
        let current_block_num = self.store.get_sync_height().await?;

        // New output notes
        let new_output_notes = executed_tx
            .output_notes()
            .iter()
            .cloned()
            .filter_map(|output_note| {
                OutputNoteRecord::try_from_output_note(output_note, submission_height).ok()
            })
            .collect::<Vec<_>>();

        // New relevant input notes
        let mut new_input_notes = vec![];
        let note_screener = NoteScreener::new(self.store.clone(), self.authenticator.clone());

        for note in notes_from_output(executed_tx.output_notes()) {
            // TODO: check_relevance() should have the option to take multiple notes
            let account_relevance = note_screener.check_relevance(note).await?;
            if !account_relevance.is_empty() {
                let metadata = *note.metadata();

                new_input_notes.push(InputNoteRecord::new(
                    note.into(),
                    current_timestamp,
                    ExpectedNoteState {
                        metadata: Some(metadata),
                        after_block_num: submission_height,
                        tag: Some(metadata.tag()),
                    }
                    .into(),
                ));
            }
        }

        // Track future input notes described in the transaction result.
        new_input_notes.extend(tx_result.future_notes().iter().map(|(note_details, tag)| {
            InputNoteRecord::new(
                note_details.clone(),
                None,
                ExpectedNoteState {
                    metadata: None,
                    after_block_num: current_block_num,
                    tag: Some(*tag),
                }
                .into(),
            )
        }));

        // Locally consumed notes
        let consumed_note_ids =
            executed_tx.tx_inputs().input_notes().iter().map(InputNote::id).collect();

        let consumed_notes = self.get_input_notes(NoteFilter::List(consumed_note_ids)).await?;

        let mut updated_input_notes = vec![];

        for mut input_note_record in consumed_notes {
            if input_note_record.consumed_locally(
                executed_tx.account_id(),
                executed_tx.id(),
                self.store.get_current_timestamp(),
            )? {
                updated_input_notes.push(input_note_record);
            }
        }

        Ok(NoteUpdateTracker::for_transaction_updates(
            new_input_notes,
            updated_input_notes,
            new_output_notes,
        ))
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
        let outgoing_assets =
            output_notes_assets.values().flat_map(|note_assets| note_assets.iter());

        collect_assets(outgoing_assets)
    }

    /// Helper to get the account incoming assets.
    async fn get_incoming_assets(
        &self,
        transaction_request: &TransactionRequest,
    ) -> Result<(BTreeMap<AccountId, u64>, BTreeSet<NonFungibleAsset>), TransactionRequestError>
    {
        // Get incoming asset notes excluding unauthenticated ones
        let incoming_notes_ids: Vec<_> = transaction_request
            .input_notes()
            .iter()
            .filter_map(|note| {
                if transaction_request.input_notes().iter().any(|n| n.id() == note.id()) {
                    None
                } else {
                    Some(note.id())
                }
            })
            .collect();

        let store_input_notes = self
            .get_input_notes(NoteFilter::List(incoming_notes_ids))
            .await
            .map_err(|err| TransactionRequestError::NoteNotFound(err.to_string()))?;

        let all_incoming_assets = store_input_notes
            .iter()
            .flat_map(|note| note.assets().iter())
            .chain(transaction_request.input_notes().iter().flat_map(|note| note.assets().iter()));

        Ok(collect_assets(all_incoming_assets))
    }

    /// Ensures a transaction request is compatible with the current account state,
    /// primarily by checking asset balances against the requested transfers.
    async fn validate_basic_account_request(
        &self,
        transaction_request: &TransactionRequest,
        account: &Account,
    ) -> Result<(), ClientError> {
        // Get outgoing assets
        let (fungible_balance_map, non_fungible_set) =
            Client::<AUTH>::get_outgoing_assets(transaction_request);

        // Get incoming assets
        let (incoming_fungible_balance_map, incoming_non_fungible_balance_set) =
            self.get_incoming_assets(transaction_request).await?;

        // Check if the account balance plus incoming assets is greater than or equal to the
        // outgoing fungible assets
        for (faucet_id, amount) in fungible_balance_map {
            let account_asset_amount = account.vault().get_balance(faucet_id).unwrap_or(0);
            let incoming_balance = incoming_fungible_balance_map.get(&faucet_id).unwrap_or(&0);
            if account_asset_amount + incoming_balance < amount {
                return Err(ClientError::AssetError(
                    AssetError::FungibleAssetAmountNotSufficient {
                        minuend: account_asset_amount,
                        subtrahend: amount,
                    },
                ));
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
                    return Err(ClientError::AssetError(
                        AssetError::NonFungibleFaucetIdTypeMismatch(
                            non_fungible.faucet_id_prefix(),
                        ),
                    ));
                },
            }
        }

        Ok(())
    }

    /// Validates that the specified transaction request can be executed by the specified account.
    ///
    /// This does't guarantee that the transaction will succeed, but it's useful to avoid submitting
    /// transactions that are guaranteed to fail. Some of the validations include:
    /// - That the account has enough balance to cover the outgoing assets.
    /// - That the client is not too far behind the chain tip.
    pub async fn validate_request(
        &mut self,
        account_id: AccountId,
        transaction_request: &TransactionRequest,
    ) -> Result<(), ClientError> {
        if let Some(max_block_number_delta) = self.max_block_number_delta {
            let current_chain_tip =
                self.rpc_api.get_block_header_by_number(None, false).await?.0.block_num();

            if current_chain_tip > self.store.get_sync_height().await? + max_block_number_delta {
                return Err(ClientError::RecencyConditionError(
                    "The client is too far behind the chain tip to execute the transaction",
                ));
            }
        }

        let account: Account = self.try_get_account(account_id).await?.try_into()?;

        if account.is_faucet() {
            // TODO(SantiagoPittella): Add faucet validations.
            Ok(())
        } else {
            self.validate_basic_account_request(transaction_request, &account).await
        }
    }

    /// Filters out invalid or non-consumable input notes by simulating
    /// note consumption and removing any that fail validation.
    async fn get_valid_input_notes(
        &self,
        account: Account,
        mut input_notes: InputNotes<InputNote>,
        tx_args: TransactionArgs,
    ) -> Result<InputNotes<InputNote>, ClientError> {
        loop {
            let data_store = ClientDataStore::new(self.store.clone());

            data_store.mast_store().load_account_code(account.code());
            let execution = NoteConsumptionChecker::new(&self.build_executor(&data_store)?)
                .check_notes_consumability(
                    account.id(),
                    self.store.get_sync_height().await?,
                    input_notes.iter().map(|n| n.clone().into_note()).collect(),
                    tx_args.clone(),
                )
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

    /// Retrieves the account interface for the specified account.
    pub(crate) async fn get_account_interface(
        &self,
        account_id: AccountId,
    ) -> Result<AccountInterface, ClientError> {
        let account: Account = self.try_get_account(account_id).await?.try_into()?;

        Ok(AccountInterface::from(&account))
    }

    /// Returns foreign account inputs for the required foreign accounts specified by the
    /// transaction request.
    ///
    /// For any [`ForeignAccount::Public`] in `foreign_accounts`, these pieces of data are retrieved
    /// from the network. For any [`ForeignAccount::Private`] account, inner data is used and only
    /// a proof of the account's existence on the network is fetched.
    ///
    /// Account data is retrieved for the node's current chain tip, so we need to check whether we
    /// currently have the corresponding block header data. Otherwise, we additionally need to
    /// retrieve it, this implies a state sync call which may update the client in other ways.
    async fn retrieve_foreign_account_inputs(
        &mut self,
        foreign_accounts: BTreeSet<ForeignAccount>,
    ) -> Result<(Option<BlockNumber>, Vec<AccountInputs>), ClientError> {
        if foreign_accounts.is_empty() {
            return Ok((None, Vec::new()));
        }

        let block_num = self.get_sync_height().await?;
        let mut return_foreign_account_inputs = Vec::with_capacity(foreign_accounts.len());

        for foreign_account in foreign_accounts {
            let account_id = foreign_account.account_id();
            let known_account_code = self
                .store
                .get_foreign_account_code(vec![account_id])
                .await?
                .pop_first()
                .map(|(_, code)| code);

            let (_, account_proof) = self
                .rpc_api
                .get_account_proof(
                    foreign_account.clone(),
                    AccountStateAt::Block(block_num),
                    known_account_code,
                )
                .await?;
            let foreign_account_inputs = match foreign_account {
                ForeignAccount::Public(account_id, ..) => {
                    let foreign_account_inputs: AccountInputs = account_proof.try_into()?;

                    // Update our foreign account code cache
                    self.store
                        .upsert_foreign_account_code(
                            account_id,
                            foreign_account_inputs.code().clone(),
                        )
                        .await?;

                    foreign_account_inputs
                },
                ForeignAccount::Private(partial_account) => {
                    let (witness, _) = account_proof.into_parts();

                    AccountInputs::new(partial_account.clone(), witness)
                },
            };

            return_foreign_account_inputs.push(foreign_account_inputs);
        }

        Ok((Some(block_num), return_foreign_account_inputs))
    }

    /// Creates a transaction executor configured with the client's runtime options,
    /// authenticator, and source manager.
    pub(crate) fn build_executor<'store, 'auth, STORE: DataStore + Sync>(
        &'auth self,
        data_store: &'store STORE,
    ) -> Result<TransactionExecutor<'store, 'auth, STORE, AUTH>, TransactionExecutorError> {
        let mut executor = TransactionExecutor::new(data_store).with_options(self.exec_options)?;
        if let Some(authenticator) = self.authenticator.as_deref() {
            executor = executor.with_authenticator(authenticator);
        }
        executor = executor.with_source_manager(self.source_manager.clone());

        Ok(executor)
    }
}

// HELPERS
// ================================================================================================

/// Accumulates fungible totals and collectable non-fungible assets from an iterator of assets.
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

/// Extracts notes from [`OutputNotes`].
/// Used for:
/// - Checking the relevance of notes to save them as input notes.
/// - Validate hashes versus expected output notes after a transaction is executed.
pub fn notes_from_output(output_notes: &OutputNotes) -> impl Iterator<Item = &Note> {
    output_notes
        .iter()
        .filter(|n| matches!(n, OutputNote::Full(_)))
        .map(|n| match n {
            OutputNote::Full(n) => n,
            // The following todo!() applies until we have a way to support flows where we have
            // partial details of the note
            OutputNote::Header(_) | OutputNote::Partial(_) => {
                todo!("For now, all details should be held in OutputNote::Fulls")
            },
        })
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
