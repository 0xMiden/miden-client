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
//! use miden_protocol::account::AccountId;
//! use miden_protocol::asset::FungibleAsset;
//! use miden_protocol::note::NoteType;
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

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_protocol::account::{Account, AccountCode, AccountId};
use miden_protocol::asset::{AssetCallbackFlag, AssetVaultKey, NonFungibleAsset};
use miden_protocol::block::BlockNumber;
use miden_protocol::errors::AssetError;
use miden_protocol::note::{
    Note,
    NoteAttachments,
    NoteDetails,
    NoteId,
    NoteRecipient,
    NoteScript,
    NoteTag,
};
use miden_protocol::transaction::AccountInputs;
use miden_protocol::vm::MIN_STACK_DEPTH;
use miden_protocol::{EMPTY_WORD, Felt, MIN_PROOF_SECURITY_LEVEL, Word};
use miden_standards::account::interface::AccountInterfaceExt;
use miden_tx::{DataStore, NoteConsumptionChecker, TransactionExecutor, TransactionVerifier};
use tracing::info;

use super::Client;
use crate::ClientError;
use crate::note::{NoteScreenerError, NoteUpdateTracker};
use crate::rpc::domain::account::AccountStorageRequirements;
use crate::rpc::{AccountStateAt, GrpcError, NodeRpcClient, RpcError};
use crate::store::data_store::ClientDataStore;
use crate::store::input_note_states::ExpectedNoteState;
use crate::store::{
    AccountRecord,
    InputNoteRecord,
    InputNoteState,
    NoteFilter,
    NoteRecordError,
    OutputNoteRecord,
    Store,
    StoreError,
    TransactionFilter,
};
use crate::sync::NoteTagRecord;
use crate::transaction::batch::InMemoryBatchDataStore;

pub mod batch;
pub use batch::{BatchBuilder, BatchBuilderError};

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
    PswapTransactionData,
    SwapTransactionData,
    TransactionRequest,
    TransactionRequestBuilder,
    TransactionRequestError,
    TransactionScriptTemplate,
};

mod result;
// RE-EXPORTS
// ================================================================================================
pub use miden_protocol::transaction::{
    ExecutedTransaction,
    InputNote,
    InputNotes,
    OutputNote,
    OutputNotes,
    ProvenTransaction,
    PublicOutputNote,
    RawOutputNote,
    RawOutputNotes,
    TransactionArgs,
    TransactionId,
    TransactionInputs,
    TransactionKernel,
    TransactionScript,
    TransactionSummary,
};
pub use miden_protocol::vm::{AdviceInputs, AdviceMap};
pub use miden_standards::account::interface::{AccountComponentInterface, AccountInterface};
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

    // TRANSACTION BATCH
    // --------------------------------------------------------------------------------------------

    /// Open a new [`BatchBuilder`] for accumulating transactions across one or more local
    /// accounts.
    ///
    /// See [`crate::transaction::batch`] for usage and constraints.
    pub fn new_transaction_batch(&self) -> BatchBuilder<'_, AUTH> {
        let inner_data_store = ClientDataStore::new(self.store.clone(), self.rpc_api.clone());
        BatchBuilder {
            client: self,
            data_store: InMemoryBatchDataStore::new(inner_data_store),
            pushed_txs: Vec::new(),
            consumed_input_notes: BTreeSet::new(),
        }
    }

    // TRANSACTION
    // --------------------------------------------------------------------------------------------

    /// Executes a transaction specified by the request against the specified account,
    /// proves it, submits it to the network, and updates the local database.
    ///
    /// Uses the client's default prover (configured via
    /// [`crate::builder::ClientBuilder::prover`]).
    pub async fn submit_new_transaction(
        &mut self,
        account_id: AccountId,
        transaction_request: TransactionRequest,
    ) -> Result<TransactionId, ClientError> {
        let prover = self.tx_prover.clone();
        self.submit_new_transaction_with_prover(account_id, transaction_request, prover)
            .await
    }

    /// Executes a transaction specified by the request against the specified account,
    /// proves it with the provided prover, submits it to the network, and updates the local
    /// database.
    ///
    /// This is useful for falling back to a different prover (e.g., local) when the default
    /// prover (e.g., remote) fails with a [`ClientError::TransactionProvingError`].
    pub async fn submit_new_transaction_with_prover(
        &mut self,
        account_id: AccountId,
        transaction_request: TransactionRequest,
        tx_prover: Arc<dyn TransactionProver>,
    ) -> Result<TransactionId, ClientError> {
        // Register any missing NTX scripts before the main transaction.
        // The registration path contains its own full execute -> prove -> submit pipeline.
        if !transaction_request.expected_ntx_scripts().is_empty() {
            Box::pin(self.ensure_ntx_scripts_registered(
                account_id,
                transaction_request.expected_ntx_scripts(),
                tx_prover.clone(),
            ))
            .await?;
        }

        let tx_result = self.execute_transaction(account_id, transaction_request).await?;
        let tx_id = tx_result.executed_transaction().id();

        let proven_transaction = self.prove_transaction_with(&tx_result, tx_prover).await?;
        let submission_height =
            self.submit_proven_transaction(proven_transaction, &tx_result).await?;

        self.apply_transaction(&tx_result, submission_height).await?;

        Ok(tx_id)
    }

    /// Creates and executes a transaction specified by the request against the specified account,
    /// but doesn't change the local database.
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
        let account: Account = self.get_native_account_record(account_id).await?.try_into()?;

        let prep = self.prepare_transaction(&account, transaction_request).await?;

        let data_store = ClientDataStore::new(self.store.clone(), self.rpc_api.clone());
        data_store.register_foreign_account_inputs(prep.foreign_account_inputs.iter().cloned());
        for fpi_account in &prep.foreign_account_inputs {
            data_store.mast_store().load_account_code(fpi_account.code());
        }

        data_store.mast_store().load_account_code(account.code());

        let mut notes = prep.notes;
        if prep.ignore_invalid_notes {
            notes = self.get_valid_input_notes(&account, notes, prep.tx_args.clone()).await?;
        }

        let executed_transaction = self
            .build_executor(&data_store)?
            .execute_transaction(account_id, prep.block_num, notes, prep.tx_args)
            .await?;

        validate_executed_transaction(&executed_transaction, &prep.output_recipients)?;
        TransactionResult::new(executed_transaction, prep.future_notes)
    }

    /// Performs the data-store-independent setup shared by `execute_transaction` and
    /// `execute_transaction_for_batch`: validates the request against the supplied
    /// `account`, loads/filters input notes, builds the transaction script and args,
    /// retrieves foreign-account inputs, upserts output note scripts, and computes the
    /// reference block number.
    ///
    /// `account` is the state validation runs against — for a single transaction this is
    /// the persisted account; inside [`crate::transaction::BatchBuilder::push`] it is the
    /// in-batch (stacked) state, so balances reflect prior pushes.
    pub(crate) async fn prepare_transaction(
        &self,
        account: &Account,
        transaction_request: TransactionRequest,
    ) -> Result<PreparedTransaction, ClientError> {
        let account_id = account.id();
        self.validate_recency().await?;
        validate_account_request(&transaction_request, account)?;

        // Retrieve all input notes from the store.
        let mut stored_note_records = self
            .store
            .get_input_notes(NoteFilter::List(transaction_request.input_note_ids().collect()))
            .await?;

        // Verify that none of the authenticated input notes are already consumed.
        for note in &stored_note_records {
            if note.is_consumed() {
                let id = note.id().expect(
                    "stored note records reaching this check carry metadata so id() is Some",
                );
                return Err(ClientError::TransactionRequestError(
                    TransactionRequestError::InputNoteAlreadyConsumed(id),
                ));
            }
        }

        // Only keep authenticated input notes from the store.
        stored_note_records.retain(InputNoteRecord::is_authenticated);

        let authenticated_note_ids =
            stored_note_records.iter().filter_map(InputNoteRecord::id).collect::<Vec<_>>();

        // Upsert request notes missing from the store so they can be tracked and updated.
        // NOTE: Unauthenticated notes may be stored locally in an unverified/invalid state at
        // this point. The upsert will replace the state to an InputNoteState::Expected (with
        // metadata included).
        let unauthenticated_input_notes = transaction_request
            .input_notes()
            .iter()
            .filter(|n| !authenticated_note_ids.contains(&n.id()))
            .cloned()
            .map(Into::into)
            .collect::<Vec<_>>();

        self.store.upsert_input_notes(&unauthenticated_input_notes).await?;

        let notes = transaction_request.build_input_notes(stored_note_records)?;

        let output_recipients =
            transaction_request.expected_output_recipients().cloned().collect::<Vec<_>>();

        let future_notes: Vec<(NoteDetails, NoteTag)> =
            transaction_request.expected_future_notes().cloned().collect();

        let tx_script = transaction_request.build_transaction_script(
            &self.get_account_interface(account_id).await?,
            self.source_manager.clone(),
        )?;

        let foreign_accounts = transaction_request.foreign_accounts().clone();

        let (fpi_block_num, foreign_account_inputs) =
            self.retrieve_foreign_account_inputs(foreign_accounts).await?;

        let ignore_invalid_notes = transaction_request.ignore_invalid_input_notes();

        let output_note_scripts: Vec<NoteScript> =
            output_recipients.iter().map(|r| r.script().clone()).collect();
        self.store.upsert_note_scripts(&output_note_scripts).await?;

        let block_num = if let Some(block_num) = fpi_block_num {
            block_num
        } else {
            self.store.get_sync_height().await?
        };

        let tx_args = transaction_request.into_transaction_args(tx_script);

        Ok(PreparedTransaction {
            notes,
            output_recipients,
            future_notes,
            tx_args,
            foreign_account_inputs,
            block_num,
            ignore_invalid_notes,
        })
    }

    /// Proves the specified transaction using the prover configured for this client.
    pub async fn prove_transaction(
        &self,
        tx_result: &TransactionResult,
    ) -> Result<ProvenTransaction, ClientError> {
        self.prove_transaction_with(tx_result, self.tx_prover.clone()).await
    }

    /// Proves the specified transaction using the provided prover.
    pub async fn prove_transaction_with(
        &self,
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
        info!("Verifying transaction proof...");
        TransactionVerifier::new(MIN_PROOF_SECURITY_LEVEL)
            .verify(&proven_transaction)
            .map_err(|source| ClientError::TransactionVerificationError {
                transaction_id: proven_transaction.id(),
                source,
            })?;
        info!("Transaction proof verified.");

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
    ) -> Result<TransactionStoreUpdate, TransactionStoreUpdateError> {
        let note_updates = self.get_note_updates(submission_height, tx_result).await?;

        let mut new_tags: Vec<NoteTagRecord> = note_updates
            .updated_input_notes()
            .filter_map(|note| {
                let note = note.inner();

                if let InputNoteState::Expected(ExpectedNoteState { tag: Some(tag), .. }) =
                    note.state()
                {
                    Some(NoteTagRecord::with_note_source(*tag, note.details_commitment()))
                } else {
                    None
                }
            })
            .collect();

        new_tags.extend(note_updates.updated_output_notes().map(|note| {
            let note = note.inner();
            NoteTagRecord::with_note_source(note.metadata().tag(), note.details_commitment())
        }));

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

        if self.account_reader(account_id).status().await?.is_locked() {
            return Err(ClientError::AccountLocked(account_id));
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
        foreign_accounts: BTreeMap<AccountId, ForeignAccount>,
    ) -> Result<[Felt; MIN_STACK_DEPTH], ClientError> {
        let (data_store, block_ref) =
            self.prepare_program_execution(account_id, foreign_accounts).await?;

        Ok(self
            .build_executor(&data_store)?
            .execute_tx_view_script(account_id, block_ref, tx_script, advice_inputs)
            .await?)
    }

    /// Executes the provided transaction script with a DAP debug adapter listening for
    /// connections, allowing interactive debugging via any DAP-compatible client.
    #[cfg(feature = "dap")]
    pub async fn execute_program_with_dap(
        &mut self,
        account_id: AccountId,
        tx_script: TransactionScript,
        advice_inputs: AdviceInputs,
        foreign_accounts: BTreeMap<AccountId, ForeignAccount>,
    ) -> Result<[Felt; MIN_STACK_DEPTH], ClientError> {
        let (data_store, block_ref) =
            self.prepare_program_execution(account_id, foreign_accounts).await?;

        Ok(self
            .build_dap_executor(&data_store)?
            .execute_tx_view_script(account_id, block_ref, tx_script, advice_inputs)
            .await?)
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

    /// Validates that the specified transaction request can be executed by the specified account.
    ///
    /// This does't guarantee that the transaction will succeed, but it's useful to avoid submitting
    /// transactions that are guaranteed to fail. Some of the validations include:
    /// - That the account has enough balance to cover the outgoing assets.
    /// - That the client is not too far behind the chain tip.
    pub async fn validate_request(
        &self,
        account_id: AccountId,
        transaction_request: &TransactionRequest,
    ) -> Result<(), ClientError> {
        self.validate_recency().await?;
        let account = self.try_get_account(account_id).await?;
        validate_account_request(transaction_request, &account)
    }

    async fn validate_recency(&self) -> Result<(), ClientError> {
        if let Some(max_block_number_delta) = self.max_block_number_delta {
            let current_chain_tip =
                self.rpc_api.get_block_header_by_number(None, false).await?.0.block_num();

            if current_chain_tip > self.store.get_sync_height().await? + max_block_number_delta {
                return Err(ClientError::RecencyConditionError(
                    "The client is too far behind the chain tip to execute the transaction",
                ));
            }
        }
        Ok(())
    }

    /// Checks whether the node's `note_scripts` registry already has each of the expected NTX
    /// scripts. For any script that is missing, creates and submits a registration transaction
    /// that produces a public note carrying that script.
    ///
    /// `account_id` is the account that will execute the registration transaction.
    ///
    /// This method is called automatically by [`Self::submit_new_transaction_with_prover`] when the
    /// [`TransactionRequest`] contains expected NTX scripts. It can also be called directly if
    /// you want to register scripts ahead of time.
    pub async fn ensure_ntx_scripts_registered(
        &mut self,
        account_id: AccountId,
        scripts: &[NoteScript],
        tx_prover: Arc<dyn TransactionProver>,
    ) -> Result<(), ClientError> {
        let mut missing_scripts = Vec::new();

        for script in scripts {
            let script_root = script.root();

            // Check if the node already has this script registered.
            match self.rpc_api.get_note_script_by_root(script_root.into()).await {
                Ok(_) => {},
                Err(RpcError::RequestError { error_kind: GrpcError::NotFound, .. }) => {
                    missing_scripts.push(script.clone());
                },
                Err(other) => {
                    return Err(ClientError::NtxScriptRegistrationFailed {
                        script_root: script_root.into(),
                        source: other,
                    });
                },
            }
        }

        if missing_scripts.is_empty() {
            return Ok(());
        }

        let registration_request = TransactionRequestBuilder::new().build_register_note_scripts(
            account_id,
            missing_scripts,
            self.rng(),
        )?;

        let tx_result = self.execute_transaction(account_id, registration_request).await?;
        let proven = self.prove_transaction_with(&tx_result, tx_prover).await?;
        let submission_height = self.submit_proven_transaction(proven, &tx_result).await?;
        self.apply_transaction(&tx_result, submission_height).await?;

        Ok(())
    }

    pub(crate) async fn get_valid_input_notes(
        &self,
        account: &Account,
        mut input_notes: InputNotes<InputNote>,
        tx_args: TransactionArgs,
    ) -> Result<InputNotes<InputNote>, ClientError> {
        loop {
            let data_store = ClientDataStore::new(self.store.clone(), self.rpc_api.clone());

            data_store.mast_store().load_account_code(account.code());
            let execution = NoteConsumptionChecker::new(&self.build_executor(&data_store)?)
                .check_notes_consumability(
                    account.id(),
                    self.store.get_sync_height().await?,
                    input_notes.iter().map(|n| n.clone().into_note()).collect(),
                    tx_args.clone(),
                )
                .await?;

            if execution.failed().is_empty() {
                break;
            }

            let failed_note_ids: BTreeSet<NoteId> =
                execution.failed().iter().map(|n| n.note().id()).collect();
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

    /// Returns foreign account inputs for the required foreign accounts specified by the
    /// transaction request.
    ///
    /// For any [`ForeignAccount::Public`] in `foreign_accounts`, these pieces of data are retrieved
    /// from the network. For any [`ForeignAccount::Private`] account, inner data is used and only
    /// a proof of the account's existence on the network is fetched.
    async fn retrieve_foreign_account_inputs(
        &self,
        foreign_accounts: BTreeMap<AccountId, ForeignAccount>,
    ) -> Result<(Option<BlockNumber>, Vec<AccountInputs>), ClientError> {
        if foreign_accounts.is_empty() {
            return Ok((None, Vec::new()));
        }

        let block_num = self.store.get_sync_height().await?;
        let mut return_foreign_account_inputs = Vec::with_capacity(foreign_accounts.len());

        for foreign_account in foreign_accounts.into_values() {
            let foreign_account_inputs = match foreign_account {
                ForeignAccount::Public(account_id, storage_requirements) => {
                    fetch_public_account_inputs(
                        &self.store,
                        &self.rpc_api,
                        account_id,
                        storage_requirements,
                        AccountStateAt::Block(block_num),
                    )
                    .await?
                },
                ForeignAccount::Private(partial_account) => {
                    let account_id = partial_account.id();
                    let (_, account_proof) = self
                        .rpc_api
                        .get_account_proof(
                            account_id,
                            AccountStorageRequirements::default(),
                            AccountStateAt::Block(block_num),
                            None,
                            None,
                        )
                        .await?;
                    let (witness, _) = account_proof.into_parts();
                    AccountInputs::new(partial_account, witness)
                },
            };

            return_foreign_account_inputs.push(foreign_account_inputs);
        }

        Ok((Some(block_num), return_foreign_account_inputs))
    }

    /// Prepares the data store and block reference for program execution.
    ///
    /// This is shared setup for both `execute_program` and `execute_program_with_dap`.
    async fn prepare_program_execution(
        &mut self,
        account_id: AccountId,
        foreign_accounts: BTreeMap<AccountId, ForeignAccount>,
    ) -> Result<(ClientDataStore, BlockNumber), ClientError> {
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

        let data_store = ClientDataStore::new(self.store.clone(), self.rpc_api.clone());

        data_store.register_foreign_account_inputs(foreign_account_inputs.iter().cloned());

        // Ensure code is loaded on MAST store
        data_store.mast_store().load_account_code(account.code());

        for fpi_account in &foreign_account_inputs {
            data_store.mast_store().load_account_code(fpi_account.code());
        }

        Ok((data_store, block_ref))
    }

    /// Creates a transaction executor configured with the client's runtime options,
    /// authenticator, and source manager.
    pub(crate) fn build_executor<'store, 'auth, STORE: DataStore + Sync>(
        &'auth self,
        data_store: &'store STORE,
    ) -> Result<TransactionExecutor<'store, 'auth, STORE, AUTH>, TransactionExecutorError> {
        let mut executor = TransactionExecutor::new(data_store)
            .with_options(self.exec_options)?
            .with_source_manager(self.source_manager.clone());
        if let Some(authenticator) = self.authenticator.as_deref() {
            executor = executor.with_authenticator(authenticator);
        }
        Ok(executor)
    }

    /// Loads an [`AccountRecord`] for an account that must be usable as a transaction's native
    /// account. Errors out if the account is not tracked or if it is watched.
    async fn get_native_account_record(
        &self,
        account_id: AccountId,
    ) -> Result<AccountRecord, ClientError> {
        let account_record = self
            .store
            .get_account(account_id)
            .await?
            .ok_or(ClientError::AccountDataNotFound(account_id))?;
        if account_record.is_watched() {
            return Err(ClientError::AccountIsWatched(account_id));
        }
        Ok(account_record)
    }

    /// Creates a transaction executor configured for DAP (Debug Adapter Protocol) debugging.
    #[cfg(feature = "dap")]
    pub(crate) fn build_dap_executor<'store, 'auth, STORE: DataStore + Sync>(
        &'auth self,
        data_store: &'store STORE,
    ) -> Result<
        TransactionExecutor<'store, 'auth, STORE, AUTH, DapProgramExecutor>,
        TransactionExecutorError,
    > {
        Ok(self.build_executor(data_store)?.with_program_executor::<DapProgramExecutor>())
    }

    /// Loads the account and constructs an [`AccountInterface`] from it.
    pub(crate) async fn get_account_interface(
        &self,
        account_id: AccountId,
    ) -> Result<AccountInterface, ClientError> {
        let account = self.try_get_account(account_id).await?;
        Ok(AccountInterface::from_account(&account))
    }

    /// Returns [`NoteUpdateTracker`] containing the note updates generated by an executed
    /// transaction.
    async fn get_note_updates(
        &self,
        submission_height: BlockNumber,
        tx_result: &TransactionResult,
    ) -> Result<NoteUpdateTracker, TransactionStoreUpdateError> {
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
        let output_notes: Vec<Note> =
            notes_from_output(executed_tx.output_notes()).cloned().collect();
        let note_screener = self.note_screener().clone();
        let output_note_relevances = note_screener.can_consume_batch(&output_notes).await?;

        for note in output_notes {
            if output_note_relevances.contains_key(&note.id()) {
                let metadata = *note.metadata();
                let tag = metadata.tag();
                let attachments = note.attachments().clone();

                new_input_notes.push(InputNoteRecord::new(
                    note.into(),
                    attachments,
                    current_timestamp,
                    ExpectedNoteState {
                        metadata: Some(metadata),
                        after_block_num: submission_height,
                        tag: Some(tag),
                    }
                    .into(),
                ));
            }
        }

        // Track future input notes described in the transaction result.
        new_input_notes.extend(tx_result.future_notes().iter().map(|(note_details, tag)| {
            InputNoteRecord::new(
                note_details.clone(),
                NoteAttachments::empty(),
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

        let consumed_notes =
            self.store.get_input_notes(NoteFilter::List(consumed_note_ids)).await?;

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
}

/// Adapts [`miden_debug::DapExecutor`] (which exposes `new` + `execute_async`) to
/// [`miden_tx::ProgramExecutor`]. `miden-debug` does not depend on `miden-tx`, so the impl
/// must live here.
#[cfg(feature = "dap")]
pub(crate) struct DapProgramExecutor(miden_debug::DapExecutor);

#[cfg(feature = "dap")]
impl miden_tx::ProgramExecutor for DapProgramExecutor {
    fn new(
        stack_inputs: miden_processor::StackInputs,
        advice_inputs: miden_processor::advice::AdviceInputs,
        options: miden_processor::ExecutionOptions,
    ) -> Self {
        Self(miden_debug::DapExecutor::new(stack_inputs, advice_inputs, options))
    }

    fn execute<H: miden_processor::Host + Send>(
        self,
        program: &miden_processor::Program,
        host: &mut H,
    ) -> impl miden_processor::FutureMaybeSend<
        Result<miden_processor::ExecutionOutput, miden_processor::ExecutionError>,
    > {
        self.0.execute_async(program, host)
    }
}

// TRANSACTION STORE UPDATE ERROR
// ================================================================================================

/// Error returned by [`Client::get_transaction_store_update`] when building the store update
/// for a submitted transaction fails.
#[derive(Debug, thiserror::Error)]
pub enum TransactionStoreUpdateError {
    #[error("store error")]
    Store(#[from] StoreError),
    #[error("note screener error")]
    NoteScreener(#[from] NoteScreenerError),
    #[error("note record error")]
    NoteRecord(#[from] NoteRecordError),
}

// HELPERS
// ================================================================================================

/// Data-store-independent state produced during transaction preparation.
pub(crate) struct PreparedTransaction {
    pub(crate) notes: InputNotes<InputNote>,
    pub(crate) output_recipients: Vec<NoteRecipient>,
    pub(crate) future_notes: Vec<(NoteDetails, NoteTag)>,
    pub(crate) tx_args: TransactionArgs,
    pub(crate) foreign_account_inputs: Vec<AccountInputs>,
    pub(crate) block_num: BlockNumber,
    pub(crate) ignore_invalid_notes: bool,
}

/// Helper to get the account outgoing assets.
///
/// Any outgoing assets resulting from executing note scripts but not present in expected output
/// notes wouldn't be included.
fn get_outgoing_assets(
    transaction_request: &TransactionRequest,
) -> (BTreeMap<AccountId, u64>, Vec<NonFungibleAsset>) {
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

    request::collect_assets(outgoing_assets)
}

/// Validates a transaction request against the supplied `account`. Faucets are currently
/// skipped; for non-faucets, defers to [`validate_basic_account_request`] for asset-balance
/// checks.
pub(super) fn validate_account_request(
    transaction_request: &TransactionRequest,
    account: &Account,
) -> Result<(), ClientError> {
    let account_interface = AccountInterface::from_account(account);
    if account_interface
        .components()
        .contains(&AccountComponentInterface::FungibleFaucet)
    {
        // TODO(SantiagoPittella): Add faucet validations.
        Ok(())
    } else {
        validate_basic_account_request(transaction_request, account)
    }
}

/// Ensures a transaction request is compatible with the current account state,
/// primarily by checking asset balances against the requested transfers.
fn validate_basic_account_request(
    transaction_request: &TransactionRequest,
    account: &Account,
) -> Result<(), ClientError> {
    // Get outgoing assets
    let (fungible_balance_map, non_fungible_set) = get_outgoing_assets(transaction_request);

    // Get incoming assets
    let (incoming_fungible_balance_map, incoming_non_fungible_balance_set) =
        transaction_request.incoming_assets();

    // Check if the account balance plus incoming assets is greater than or equal to the
    // outgoing fungible assets
    for (faucet_id, amount) in fungible_balance_map {
        let account_asset_amount = get_fungible_balance_by_faucet(account, faucet_id)?;
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
    for non_fungible in &non_fungible_set {
        match account.vault().has_non_fungible_asset(*non_fungible) {
            Ok(true) => (),
            Ok(false) => {
                // Check if the non fungible asset is in the incoming assets
                if !incoming_non_fungible_balance_set.contains(non_fungible) {
                    return Err(ClientError::TransactionRequestError(
                        TransactionRequestError::MissingNonFungibleAsset(non_fungible.faucet_id()),
                    ));
                }
            },
            _ => {
                return Err(ClientError::TransactionRequestError(
                    TransactionRequestError::MissingNonFungibleAsset(non_fungible.faucet_id()),
                ));
            },
        }
    }

    Ok(())
}

fn get_fungible_balance_by_faucet(
    account: &Account,
    faucet_id: AccountId,
) -> Result<u64, ClientError> {
    let mut total = 0u64;

    for callback_flag in [AssetCallbackFlag::Disabled, AssetCallbackFlag::Enabled] {
        let vault_key = AssetVaultKey::new_fungible(faucet_id, callback_flag);
        let amount = account.vault().get_balance(vault_key).map_err(ClientError::AssetError)?;
        total = total.saturating_add(u64::from(amount));
    }

    Ok(total)
}

/// Fetches a foreign account's proof and details from the network, converts them into
/// [`AccountInputs`], and caches the returned code in the store for future requests.
///
/// # Errors
/// Fails if the account is private: the RPC does not return account details for them, causing
/// [`TransactionRequestError::ForeignAccountDataMissing`].
pub(crate) async fn fetch_public_account_inputs(
    store: &Arc<dyn Store>,
    rpc_api: &Arc<dyn NodeRpcClient>,
    account_id: AccountId,
    storage_requirements: AccountStorageRequirements,
    account_state_at: AccountStateAt,
) -> Result<AccountInputs, ClientError> {
    let known_account_code: Option<AccountCode> =
        store.get_foreign_account_code(vec![account_id]).await?.into_values().next();

    // Get vault assets only if known commitment doesn't match the current one.
    let known_vault_commitment = store
        .get_account_header(account_id)
        .await?
        .map_or(EMPTY_WORD, |(header, _)| header.vault_root());

    let (_, account_proof) = rpc_api
        .get_account_proof(
            account_id,
            storage_requirements.clone(),
            account_state_at,
            known_account_code,
            Some(known_vault_commitment),
        )
        .await?;

    let account_inputs = request::account_proof_into_inputs(account_proof, &storage_requirements)?;

    let _ = store
        .upsert_foreign_account_code(account_id, account_inputs.code().clone())
        .await
        .inspect_err(|err| {
            tracing::warn!(
                %account_id,
                %err,
                "Failed to persist foreign account code to store"
            );
        });

    Ok(account_inputs)
}

/// Extracts notes from [`RawOutputNotes`].
/// Used for:
/// - Checking the relevance of notes to save them as input notes.
/// - Validate hashes versus expected output notes after a transaction is executed.
pub fn notes_from_output(output_notes: &RawOutputNotes) -> impl Iterator<Item = &Note> {
    output_notes.iter().filter_map(|n| match n {
        RawOutputNote::Full(n) => Some(n),
        RawOutputNote::Partial(_) => None,
    })
}

/// Validates that the executed transaction's output recipients match what was expected in the
/// transaction request.
pub(crate) fn validate_executed_transaction(
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
