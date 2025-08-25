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
//! use miden_client::transaction::{
//!     PaymentNoteDescription,
//!     TransactionRequestBuilder,
//!     TransactionResult,
//! };
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
//!     // Execute the transaction. This returns a TransactionResult.
//!     let tx_result: TransactionResult = client.new_transaction(sender_id, tx_request).await?;
//!
//!     // Prove and submit the transaction, persisting its details to the local store.
//!     client.submit_transaction(tx_result).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! For more detailed information about each function and error type, refer to the specific API
//! documentation.

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::{self};

use miden_objects::account::{Account, AccountCode, AccountDelta, AccountId};
use miden_objects::assembly::DefaultSourceManager;
use miden_objects::asset::{Asset, NonFungibleAsset};
use miden_objects::block::BlockNumber;
use miden_objects::note::{Note, NoteDetails, NoteId, NoteRecipient, NoteTag};
use miden_objects::transaction::{AccountInputs, TransactionArgs};
use miden_objects::{AssetError, Felt, Word};
use miden_remote_prover::client::RemoteTransactionProver;
use miden_tx::utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable};
use miden_tx::{DataStore, NoteConsumptionChecker, TransactionExecutor};
use tracing::info;

use super::Client;
use crate::ClientError;
use crate::note::{NoteScreener, NoteUpdateTracker};
use crate::rpc::domain::account::AccountProof;
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

mod request;

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
    TransactionId,
    TransactionScript,
    TransactionWitness,
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

// TRANSACTION RESULT
// ================================================================================================

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait TransactionProver {
    async fn prove(
        &self,
        tx_result: TransactionWitness,
    ) -> Result<ProvenTransaction, TransactionProverError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl TransactionProver for LocalTransactionProver {
    async fn prove(
        &self,
        witness: TransactionWitness,
    ) -> Result<ProvenTransaction, TransactionProverError> {
        LocalTransactionProver::prove(self, witness)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl TransactionProver for RemoteTransactionProver {
    async fn prove(
        &self,
        witness: TransactionWitness,
    ) -> Result<ProvenTransaction, TransactionProverError> {
        let fut = RemoteTransactionProver::prove(self, witness);
        fut.await
    }
}

/// Represents the result of executing a transaction by the client.
///
/// It contains an [`ExecutedTransaction`], and a list of `future_notes` that we expect to receive
/// in the future (you can check at swap notes for an example of this).
#[derive(Clone, Debug, PartialEq)]
pub struct TransactionResult {
    transaction: ExecutedTransaction,
    future_notes: Vec<(NoteDetails, NoteTag)>,
}

impl TransactionResult {
    /// Screens the output notes to store and track the relevant ones, and instantiates a
    /// [`TransactionResult`].
    pub fn new(
        transaction: ExecutedTransaction,
        future_notes: Vec<(NoteDetails, NoteTag)>,
    ) -> Result<Self, ClientError> {
        Ok(Self { transaction, future_notes })
    }

    /// Returns the [`ExecutedTransaction`].
    pub fn executed_transaction(&self) -> &ExecutedTransaction {
        &self.transaction
    }

    /// Returns the output notes that were generated as a result of the transaction execution.
    pub fn created_notes(&self) -> &OutputNotes {
        self.transaction.output_notes()
    }

    /// Returns the list of notes that might be created in the future as a result of the
    /// transaction execution.
    pub fn future_notes(&self) -> &[(NoteDetails, NoteTag)] {
        &self.future_notes
    }

    /// Returns the block against which the transaction was executed.
    pub fn block_num(&self) -> BlockNumber {
        self.transaction.block_header().block_num()
    }

    /// Returns transaction's [`TransactionArgs`].
    pub fn transaction_arguments(&self) -> &TransactionArgs {
        self.transaction.tx_args()
    }

    /// Returns the [`AccountDelta`] that describes the change of state for the executing [Account].
    pub fn account_delta(&self) -> &AccountDelta {
        self.transaction.account_delta()
    }

    /// Returns input notes that were consumed as part of the transaction.
    pub fn consumed_notes(&self) -> &InputNotes<InputNote> {
        self.transaction.tx_inputs().input_notes()
    }
}

impl From<TransactionResult> for ExecutedTransaction {
    fn from(tx_result: TransactionResult) -> ExecutedTransaction {
        tx_result.transaction
    }
}

impl Serializable for TransactionResult {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.transaction.write_into(target);
        self.future_notes.write_into(target);
    }
}

impl Deserializable for TransactionResult {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let transaction = ExecutedTransaction::read_from(source)?;
        let future_notes = Vec::<(NoteDetails, NoteTag)>::read_from(source)?;

        Ok(Self { transaction, future_notes })
    }
}

// TRANSACTION RECORD
// ================================================================================================

/// Describes a transaction that has been executed and is being tracked on the Client.
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    /// Unique identifier for the transaction.
    pub id: TransactionId,
    /// Details associated with the transaction.
    pub details: TransactionDetails,
    /// Script associated with the transaction, if no script is provided, only note scripts are
    /// executed.
    pub script: Option<TransactionScript>,
    /// Current status of the transaction.
    pub status: TransactionStatus,
}

impl TransactionRecord {
    /// Creates a new [`TransactionRecord`] instance.
    pub fn new(
        id: TransactionId,
        details: TransactionDetails,
        script: Option<TransactionScript>,
        status: TransactionStatus,
    ) -> TransactionRecord {
        TransactionRecord { id, details, script, status }
    }

    /// Updates (if necessary) the transaction status to signify that the transaction was
    /// committed. Will return true if the record was modified, false otherwise.
    pub fn commit_transaction(&mut self, commit_height: BlockNumber) -> bool {
        match self.status {
            TransactionStatus::Pending => {
                self.status = TransactionStatus::Committed(commit_height);
                true
            },
            TransactionStatus::Discarded(_) | TransactionStatus::Committed(_) => false,
        }
    }

    /// Updates (if necessary) the transaction status to signify that the transaction was
    /// discarded. Will return true if the record was modified, false otherwise.
    pub fn discard_transaction(&mut self, cause: DiscardCause) -> bool {
        match self.status {
            TransactionStatus::Pending => {
                self.status = TransactionStatus::Discarded(cause);
                true
            },
            TransactionStatus::Discarded(_) | TransactionStatus::Committed(_) => false,
        }
    }
}

/// Describes the details associated with a transaction.
#[derive(Debug, Clone)]
pub struct TransactionDetails {
    /// ID of the account that executed the transaction.
    pub account_id: AccountId,
    /// Initial state of the account before the transaction was executed.
    pub init_account_state: Word,
    /// Final state of the account after the transaction was executed.
    pub final_account_state: Word,
    /// Nullifiers of the input notes consumed in the transaction.
    pub input_note_nullifiers: Vec<Word>,
    /// Output notes generated as a result of the transaction.
    pub output_notes: OutputNotes,
    /// Block number for the block against which the transaction was executed.
    pub block_num: BlockNumber,
    /// Block number at which the transaction was submitted.
    pub submission_height: BlockNumber,
    /// Block number at which the transaction is set to expire.
    pub expiration_block_num: BlockNumber,
}

impl Serializable for TransactionDetails {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.account_id.write_into(target);
        self.init_account_state.write_into(target);
        self.final_account_state.write_into(target);
        self.input_note_nullifiers.write_into(target);
        self.output_notes.write_into(target);
        self.block_num.write_into(target);
        self.submission_height.write_into(target);
        self.expiration_block_num.write_into(target);
    }
}

impl Deserializable for TransactionDetails {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let account_id = AccountId::read_from(source)?;
        let init_account_state = Word::read_from(source)?;
        let final_account_state = Word::read_from(source)?;
        let input_note_nullifiers = Vec::<Word>::read_from(source)?;
        let output_notes = OutputNotes::read_from(source)?;
        let block_num = BlockNumber::read_from(source)?;
        let submission_height = BlockNumber::read_from(source)?;
        let expiration_block_num = BlockNumber::read_from(source)?;

        Ok(Self {
            account_id,
            init_account_state,
            final_account_state,
            input_note_nullifiers,
            output_notes,
            block_num,
            submission_height,
            expiration_block_num,
        })
    }
}

/// Represents the cause of the discarded transaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiscardCause {
    Expired,
    InputConsumed,
    DiscardedInitialState,
    Stale,
}

impl DiscardCause {
    pub fn from_string(cause: &str) -> Result<Self, DeserializationError> {
        match cause {
            "Expired" => Ok(DiscardCause::Expired),
            "InputConsumed" => Ok(DiscardCause::InputConsumed),
            "DiscardedInitialState" => Ok(DiscardCause::DiscardedInitialState),
            "Stale" => Ok(DiscardCause::Stale),
            _ => Err(DeserializationError::InvalidValue(format!("Invalid discard cause: {cause}"))),
        }
    }
}

impl fmt::Display for DiscardCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscardCause::Expired => write!(f, "Expired"),
            DiscardCause::InputConsumed => write!(f, "InputConsumed"),
            DiscardCause::DiscardedInitialState => write!(f, "DiscardedInitialState"),
            DiscardCause::Stale => write!(f, "Stale"),
        }
    }
}

impl Serializable for DiscardCause {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        match self {
            DiscardCause::Expired => target.write_u8(0),
            DiscardCause::InputConsumed => target.write_u8(1),
            DiscardCause::DiscardedInitialState => target.write_u8(2),
            DiscardCause::Stale => target.write_u8(3),
        }
    }
}

impl Deserializable for DiscardCause {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        match source.read_u8()? {
            0 => Ok(DiscardCause::Expired),
            1 => Ok(DiscardCause::InputConsumed),
            2 => Ok(DiscardCause::DiscardedInitialState),
            3 => Ok(DiscardCause::Stale),
            _ => Err(DeserializationError::InvalidValue("Invalid discard cause".to_string())),
        }
    }
}

/// Represents the status of a transaction.
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    /// Transaction has been submitted but not yet committed.
    Pending,
    /// Transaction has been committed and included at the specified block number.
    Committed(BlockNumber),
    /// Transaction has been discarded and isn't included in the node.
    Discarded(DiscardCause),
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "Pending"),
            TransactionStatus::Committed(block_number) => {
                write!(f, "Committed (Block: {block_number})")
            },
            TransactionStatus::Discarded(cause) => write!(f, "Discarded ({cause})",),
        }
    }
}

// TRANSACTION STORE UPDATE
// ================================================================================================

/// Represents the changes that need to be applied to the client store as a result of a
/// transaction execution.
pub struct TransactionStoreUpdate {
    /// Details of the executed transaction to be inserted.
    executed_transaction: ExecutedTransaction,
    /// Block number at which the transaction was submitted.
    submission_height: BlockNumber,
    /// Updated account state after the [`AccountDelta`] has been applied.
    updated_account: Account,
    /// Information about note changes after the transaction execution.
    note_updates: NoteUpdateTracker,
    /// New note tags to be tracked.
    new_tags: Vec<NoteTagRecord>,
}

impl TransactionStoreUpdate {
    /// Creates a new [`TransactionStoreUpdate`] instance.
    ///
    /// # Arguments
    /// - `executed_transaction`: The executed transaction details.
    /// - `submission_height`: The block number at which the transaction was submitted.
    /// - `updated_account`: The updated account state after applying the transaction.
    /// - `note_updates`: The note updates that need to be applied to the store after the
    ///   transaction execution.
    /// - `new_tags`: New note tags that were need to be tracked because of created notes.
    pub fn new(
        executed_transaction: ExecutedTransaction,
        submission_height: BlockNumber,
        updated_account: Account,
        note_updates: NoteUpdateTracker,
        new_tags: Vec<NoteTagRecord>,
    ) -> Self {
        Self {
            executed_transaction,
            submission_height,
            updated_account,
            note_updates,
            new_tags,
        }
    }

    /// Returns the executed transaction.
    pub fn executed_transaction(&self) -> &ExecutedTransaction {
        &self.executed_transaction
    }

    /// Returns the block number at which the transaction was submitted.
    pub fn submission_height(&self) -> BlockNumber {
        self.submission_height
    }

    /// Returns the updated account.
    pub fn updated_account(&self) -> &Account {
        &self.updated_account
    }

    /// Returns the note updates that need to be applied after the transaction execution.
    pub fn note_updates(&self) -> &NoteUpdateTracker {
        &self.note_updates
    }

    /// Returns the new tags that were created as part of the transaction.
    pub fn new_tags(&self) -> &[NoteTagRecord] {
        &self.new_tags
    }
}

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
    pub async fn new_transaction(
        &mut self,
        account_id: AccountId,
        transaction_request: TransactionRequest,
    ) -> Result<TransactionResult, ClientError> {
        // Validates the transaction request before executing
        self.validate_request(account_id, &transaction_request).await?;

        // Ensure authenticated notes have their inclusion proofs (a.k.a they're in a committed
        // state)
        let authenticated_input_note_ids: Vec<NoteId> =
            transaction_request.authenticated_input_note_ids().collect::<Vec<_>>();

        let authenticated_note_records = self
            .store
            .get_input_notes(NoteFilter::List(authenticated_input_note_ids))
            .await?;

        // If tx request contains unauthenticated_input_notes we should insert them
        let unauthenticated_input_notes = transaction_request
            .unauthenticated_input_notes()
            .iter()
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
        for fpi_account in &foreign_account_inputs {
            data_store.mast_store().load_account_code(fpi_account.code());
        }

        let tx_args = transaction_request.into_transaction_args(tx_script, foreign_account_inputs);

        let block_num = if let Some(block_num) = fpi_block_num {
            block_num
        } else {
            self.store.get_sync_height().await?
        };

        // TODO: Refactor this to get account code only?
        let account_record = self
            .store
            .get_account(account_id)
            .await?
            .ok_or(ClientError::AccountDataNotFound(account_id))?;
        let account: Account = account_record.into();
        data_store.mast_store().load_account_code(account.code());

        if ignore_invalid_notes {
            // Remove invalid notes
            notes = self.get_valid_input_notes(account, notes, tx_args.clone()).await?;
        }

        // Execute the transaction and get the witness
        let executed_transaction = self
            .build_executor(&data_store)?
            .execute_transaction(
                account_id,
                block_num,
                notes,
                tx_args,
                Arc::new(DefaultSourceManager::default()), // TODO: Use the correct source manager
            )
            .await?;

        validate_executed_transaction(&executed_transaction, &output_recipients)?;

        TransactionResult::new(executed_transaction, future_notes)
    }

    /// Proves the specified transaction using a local prover, submits it to the network, and saves
    /// the transaction into the local database for tracking.
    pub async fn submit_transaction(
        &mut self,
        tx_result: TransactionResult,
    ) -> Result<(), ClientError> {
        self.submit_transaction_with_prover(tx_result, self.tx_prover.clone()).await
    }

    /// Proves the specified transaction using the provided prover, submits it to the network, and
    /// saves the transaction into the local database for tracking.
    pub async fn submit_transaction_with_prover(
        &mut self,
        tx_result: TransactionResult,
        tx_prover: Arc<dyn TransactionProver>,
    ) -> Result<(), ClientError> {
        let proven_transaction = self.prove_transaction(&tx_result, tx_prover).await?;
        let block_num = self.submit_proven_transaction(proven_transaction).await?;
        self.apply_transaction(block_num, tx_result).await
    }

    /// Proves the specified transaction result using the provided prover.
    async fn prove_transaction(
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

    async fn submit_proven_transaction(
        &mut self,
        proven_transaction: ProvenTransaction,
    ) -> Result<BlockNumber, ClientError> {
        info!("Submitting transaction to the network...");
        let block_num = self.rpc_api.submit_proven_transaction(proven_transaction).await?;
        info!("Transaction submitted.");

        Ok(block_num)
    }

    async fn apply_transaction(
        &self,
        submission_height: BlockNumber,
        tx_result: TransactionResult,
    ) -> Result<(), ClientError> {
        // Transaction was proven and submitted to the node correctly, persist note details and
        // update account
        info!("Applying transaction to the local store...");

        let account_id = tx_result.executed_transaction().account_id();
        let account_delta = tx_result.account_delta();
        let account_record = self.try_get_account(account_id).await?;

        if account_record.is_locked() {
            return Err(ClientError::AccountLocked(account_id));
        }

        let mut account: Account = account_record.into();
        account.apply_delta(account_delta)?;

        if self
            .store
            .get_account_header_by_commitment(account.commitment())
            .await?
            .is_some()
        {
            return Err(ClientError::StoreError(StoreError::AccountCommitmentAlreadyExists(
                account.commitment(),
            )));
        }

        let note_updates = self.get_note_updates(submission_height, &tx_result).await?;

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

        let tx_update = TransactionStoreUpdate::new(
            tx_result.into(),
            submission_height,
            account,
            note_updates,
            new_tags,
        );

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

        let account: Account = account_record.into();

        let data_store = ClientDataStore::new(self.store.clone());

        // Ensure code is loaded on MAST store
        data_store.mast_store().load_account_code(account.code());

        for fpi_account in &foreign_account_inputs {
            data_store.mast_store().load_account_code(fpi_account.code());
        }

        Ok(self
            .build_executor(&data_store)?
            .execute_tx_view_script(
                account_id,
                block_ref,
                tx_script,
                advice_inputs,
                foreign_account_inputs,
                Arc::new(DefaultSourceManager::default()), // TODO: Use the correct source manager
            )
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
            .filter_map(|(note_id, _)| {
                if transaction_request
                    .unauthenticated_input_notes()
                    .iter()
                    .any(|note| note.id() == *note_id)
                {
                    None
                } else {
                    Some(*note_id)
                }
            })
            .collect();

        let store_input_notes = self
            .get_input_notes(NoteFilter::List(incoming_notes_ids))
            .await
            .map_err(|err| TransactionRequestError::NoteNotFound(err.to_string()))?;

        let all_incoming_assets =
            store_input_notes.iter().flat_map(|note| note.assets().iter()).chain(
                transaction_request
                    .unauthenticated_input_notes()
                    .iter()
                    .flat_map(|note| note.assets().iter()),
            );

        Ok(collect_assets(all_incoming_assets))
    }

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
                    "The client is too far behind the chain tip to execute the transaction"
                        .to_string(),
                ));
            }
        }

        let account: Account = self.try_get_account(account_id).await?.into();

        if account.is_faucet() {
            // TODO(SantiagoPittella): Add faucet validations.
            Ok(())
        } else {
            self.validate_basic_account_request(transaction_request, &account).await
        }
    }

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
                    input_notes.clone(),
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
        let account: Account = self.try_get_account(account_id).await?.into();

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

        let mut return_foreign_account_inputs = Vec::with_capacity(foreign_accounts.len());

        let account_ids = foreign_accounts.iter().map(ForeignAccount::account_id);
        let known_account_codes =
            self.store.get_foreign_account_code(account_ids.collect()).await?;

        let known_account_codes: Vec<AccountCode> = known_account_codes.into_values().collect();

        // Fetch account proofs
        let (block_num, account_proofs) =
            self.rpc_api.get_account_proofs(&foreign_accounts, known_account_codes).await?;

        let mut account_proofs: BTreeMap<AccountId, AccountProof> =
            account_proofs.into_iter().map(|proof| (proof.account_id(), proof)).collect();

        for foreign_account in &foreign_accounts {
            let foreign_account_inputs = match foreign_account {
                ForeignAccount::Public(account_id, ..) => {
                    let account_proof = account_proofs
                        .remove(account_id)
                        .expect("proof was requested and received");

                    let foreign_account_inputs: AccountInputs = account_proof.try_into()?;

                    // Update  our foreign account code cache
                    self.store
                        .upsert_foreign_account_code(
                            *account_id,
                            foreign_account_inputs.code().clone(),
                        )
                        .await?;

                    foreign_account_inputs
                },
                ForeignAccount::Private(partial_account) => {
                    let account_id = partial_account.id();
                    let (witness, _) = account_proofs
                        .remove(&account_id)
                        .expect("proof was requested and received")
                        .into_parts();

                    AccountInputs::new(partial_account.clone(), witness)
                },
            };

            return_foreign_account_inputs.push(foreign_account_inputs);
        }

        // Optionally retrieve block header if we don't have it
        if self.store.get_block_header_by_num(block_num).await?.is_none() {
            info!(
                "Getting current block header data to execute transaction with foreign account requirements"
            );
            let summary = self.sync_state().await?;

            if summary.block_num != block_num {
                let mut current_partial_mmr = self.build_current_partial_mmr().await?;
                self.get_and_store_authenticated_block(block_num, &mut current_partial_mmr)
                    .await?;
            }
        }

        Ok((Some(block_num), return_foreign_account_inputs))
    }

    pub(crate) fn build_executor<'store, 'auth, STORE: DataStore + Sync>(
        &'auth self,
        data_store: &'store STORE,
    ) -> Result<TransactionExecutor<'store, 'auth, STORE, AUTH>, TransactionExecutorError> {
        TransactionExecutor::with_options(
            data_store,
            self.authenticator.as_deref(),
            self.exec_options,
        )
    }
}

// TESTING HELPERS
// ================================================================================================

#[cfg(feature = "testing")]
impl<AUTH: TransactionAuthenticator + Sync + 'static> Client<AUTH> {
    pub async fn testing_prove_transaction(
        &mut self,
        tx_result: &TransactionResult,
    ) -> Result<ProvenTransaction, ClientError> {
        self.prove_transaction(tx_result, self.tx_prover.clone()).await
    }

    pub async fn testing_submit_proven_transaction(
        &mut self,
        proven_transaction: ProvenTransaction,
    ) -> Result<BlockNumber, ClientError> {
        self.submit_proven_transaction(proven_transaction).await
    }

    pub async fn testing_apply_transaction(
        &self,
        tx_result: TransactionResult,
    ) -> Result<(), ClientError> {
        self.apply_transaction(self.get_sync_height().await.unwrap(), tx_result).await
    }
}

// HELPERS
// ================================================================================================

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

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use miden_lib::account::auth::AuthRpoFalcon512;
    use miden_lib::transaction::TransactionKernel;
    use miden_objects::Word;
    use miden_objects::account::{
        AccountBuilder,
        AccountComponent,
        AuthSecretKey,
        StorageMap,
        StorageSlot,
    };
    use miden_objects::asset::{Asset, FungibleAsset};
    use miden_objects::crypto::dsa::rpo_falcon512::SecretKey;
    use miden_objects::note::NoteType;
    use miden_objects::testing::account_id::{
        ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET,
        ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
        ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
    };
    use miden_tx::utils::{Deserializable, Serializable};

    use super::PaymentNoteDescription;
    use crate::tests::create_test_client;
    use crate::transaction::{TransactionRequestBuilder, TransactionResult};

    #[tokio::test]
    async fn transaction_creates_two_notes() {
        let (mut client, _, keystore) = Box::pin(create_test_client()).await;
        let asset_1: Asset =
            FungibleAsset::new(ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET.try_into().unwrap(), 123)
                .unwrap()
                .into();
        let asset_2: Asset =
            FungibleAsset::new(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET.try_into().unwrap(), 500)
                .unwrap()
                .into();

        let secret_key = SecretKey::new();
        let pub_key = secret_key.public_key();
        keystore.add_key(&AuthSecretKey::RpoFalcon512(secret_key)).unwrap();

        let wallet_component = AccountComponent::compile(
            "
                export.::miden::contracts::wallets::basic::receive_asset
                export.::miden::contracts::wallets::basic::move_asset_to_note
            ",
            TransactionKernel::assembler(),
            vec![StorageSlot::Value(Word::default()), StorageSlot::Map(StorageMap::default())],
        )
        .unwrap()
        .with_supports_all_types();

        let account = AccountBuilder::new(Default::default())
            .with_component(wallet_component)
            .with_auth_component(AuthRpoFalcon512::new(pub_key))
            .with_assets([asset_1, asset_2])
            .build_existing()
            .unwrap();

        client.add_account(&account, None, false).await.unwrap();
        client.sync_state().await.unwrap();
        let tx_request = TransactionRequestBuilder::new()
            .build_pay_to_id(
                PaymentNoteDescription::new(
                    vec![asset_1, asset_2],
                    account.id(),
                    ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE.try_into().unwrap(),
                ),
                NoteType::Private,
                client.rng(),
            )
            .unwrap();

        let tx_result = Box::pin(client.new_transaction(account.id(), tx_request)).await.unwrap();
        assert!(
            tx_result
                .created_notes()
                .get_note(0)
                .assets()
                .is_some_and(|assets| assets.num_assets() == 2)
        );
        // Prove and apply transaction
        Box::pin(client.testing_apply_transaction(tx_result.clone())).await.unwrap();

        // Test serialization
        let bytes: std::vec::Vec<u8> = tx_result.to_bytes();
        let decoded = TransactionResult::read_from_bytes(&bytes).unwrap();

        assert_eq!(tx_result, decoded);
    }
}
