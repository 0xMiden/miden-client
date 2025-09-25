use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_objects::Word;
use miden_objects::account::{Account, AccountId};
use miden_objects::block::BlockNumber;
use miden_objects::note::{NoteDetails, NoteId, NoteRecipient, NoteTag};
use miden_objects::transaction::{
    AccountInputs,
    ExecutedTransaction,
    InputNote,
    InputNotes,
    ProvenTransaction,
    TransactionArgs,
    TransactionId,
};
use miden_tx::auth::TransactionAuthenticator;
use miden_tx::{NoteConsumptionChecker, TransactionExecutor};

use crate::rpc::NodeRpcClient;
use crate::store::data_store::ClientDataStore;
use crate::transaction::{TransactionProver, TransactionRequest, TransactionStoreUpdate};
use crate::{ClientError, DebugMode, TransactionPipelineError};

#[derive(Clone)]
pub struct TransactionPipeline {
    /// The RPC client used to communicate with the node.
    rpc_api: Arc<dyn NodeRpcClient + Send>,
    /// Indicates whether scripts should be assembled in debug mode or not.
    debug_mode: DebugMode,
    /// Transaction request that describes the transaction to run through the pipeline.
    transaction_request: TransactionRequest,
    /// Executed transaction produced after running the script.
    executed_transaction: Option<ExecutedTransaction>,
    /// Proven transaction generated after proving the execution.
    proven_transaction: Option<ProvenTransaction>,
    /// Future notes expected to be created as a result of the transaction.
    future_notes: Vec<(NoteDetails, NoteTag)>,
    /// Block height returned by the node after the proven transaction submission.
    submission_height: Option<BlockNumber>,
}

impl TransactionPipeline {
    /// Creates a new [`TransactionPipeline`].
    pub fn new(
        rpc_api: Arc<dyn NodeRpcClient + Send>,
        debug_mode: DebugMode,
        transaction_request: TransactionRequest,
    ) -> Self {
        Self {
            rpc_api,
            debug_mode,
            transaction_request,
            executed_transaction: None,
            proven_transaction: None,
            future_notes: Vec::new(),
            submission_height: None,
        }
    }

    // PIPELINE DISPATCHERS
    // --------------------------------------------------------------------------------------------

    /// Creates and executes a transaction specified by the request against the specified account,
    /// storing the resulting [`ExecutedTransaction`] inside the pipeline.
    ///
    /// If the transaction utilizes foreign account data, there is a chance that the client doesn't
    /// have the required block header in the local database. In these scenarios, a sync to
    /// the chain tip is performed, and the required block header is retrieved.
    ///
    /// # Errors
    ///
    /// - Returns [`ClientError::MissingOutputRecipients`] if the [`TransactionRequest`] output
    ///   notes are not a subset of executor's output notes.
    /// - Returns [`ClientError::TransactionPipelineError`] wrapping a
    ///   [`TransactionPipelineError::Executor`] if the execution fails.
    /// - Returns [`ClientError::TransactionPipelineError`] wrapping a
    ///   [`TransactionPipelineError::Request`] if the request is invalid.
    pub async fn execute_transaction(
        &mut self,
        // TODO: this should be a partial account
        account: Account,
        foreign_account_inputs: Vec<AccountInputs>,
        mut input_notes: InputNotes<InputNote>,
        executor: &TransactionExecutor<
            '_,
            '_,
            ClientDataStore,
            impl TransactionAuthenticator + Sync,
        >,
        block_ref: BlockNumber,
    ) -> Result<(), ClientError> {
        let output_recipients = self
            .transaction_request
            .expected_output_recipients()
            .cloned()
            .collect::<Vec<_>>();

        let future_notes: Vec<(NoteDetails, NoteTag)> =
            self.transaction_request.expected_future_notes().cloned().collect();

        let tx_script = self
            .transaction_request
            .build_transaction_script(&(&account).into(), self.debug_mode)?;

        let ignore_invalid_notes = self.transaction_request.ignore_invalid_input_notes();

        let tx_args = self
            .transaction_request
            .clone()
            .into_transaction_args(tx_script, foreign_account_inputs);

        if ignore_invalid_notes {
            // Remove invalid notes
            input_notes =
                get_valid_input_notes(account.id(), input_notes, &tx_args, executor, &block_ref)
                    .await?;
        }

        // Execute the transaction and get the witness
        let executed_transaction = executor
            .execute_transaction(account.id(), block_ref, input_notes, tx_args)
            .await?;

        validate_executed_transaction(&executed_transaction, &output_recipients)?;

        self.future_notes = future_notes;
        self.proven_transaction = None;
        self.submission_height = None;
        self.executed_transaction = Some(executed_transaction);

        Ok(())
    }

    /// Generates a proof for the executed transaction stored in this pipeline, caching the
    /// resulting [`ProvenTransaction`] for later submission.
    pub async fn prove_transaction(
        &mut self,
        prover: Arc<dyn TransactionProver + Send + Sync>,
    ) -> Result<ProvenTransaction, ClientError> {
        if let Some(proven) = &self.proven_transaction {
            return Ok(proven.clone());
        }

        let executed = self
            .executed_transaction
            .clone()
            .ok_or_else(|| ClientError::from(TransactionPipelineError::NotExecuted))?;

        let proof = prover.prove(executed.clone().into()).await?;
        self.proven_transaction = Some(proof.clone());

        Ok(proof)
    }

    /// Submits the proven transaction previously generated by [`Self::prove_transaction`].
    pub async fn submit_proven_transaction(&mut self) -> Result<BlockNumber, ClientError> {
        if let Some(height) = self.submission_height {
            return Ok(height);
        }

        let proven = self
            .proven_transaction
            .clone()
            .ok_or_else(|| ClientError::from(TransactionPipelineError::ProofNotGenerated))?;

        let submission_height = self.rpc_api.submit_proven_transaction(proven).await?;
        self.submission_height = Some(submission_height);

        Ok(submission_height)
    }

    // ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns a reference to the [`TransactionRequest`] of this pipeline.
    pub fn request(&self) -> &TransactionRequest {
        &self.transaction_request
    }

    /// Returns the [`TransactionId`] corresponding to the transaction, or an error if it has not
    /// yet been executed.
    pub fn id(&self) -> Result<TransactionId, ClientError> {
        Ok(self.executed_transaction()?.id())
    }

    /// Returns a reference to the [`TransactionRequest`] of this pipeline, or `None` if the
    /// executing step has not been performed.
    pub fn executed_transaction(&self) -> Result<&ExecutedTransaction, ClientError> {
        self.executed_transaction
            .as_ref()
            .ok_or_else(|| ClientError::from(TransactionPipelineError::NotExecuted))
    }

    /// Returns a reference to the [`ProvenTransaction`] of this pipeline, or `None` if the proving
    /// step was still not performed.
    pub fn proven_transaction(&self) -> Option<&ProvenTransaction> {
        self.proven_transaction.as_ref()
    }

    /// Returns the block number that was returned when the transaction was submitted to the
    /// network, or `None` if the submission step was still not performed.
    pub fn submission_height(&self) -> Option<BlockNumber> {
        self.submission_height
    }

    /// Returns a reference to notes that might be created as a result of future transactions.
    ///
    /// An example of this could be when a note created by this [`TransactionPipeline`] contains a
    /// script that creates other notes, which would be created when the first note is
    /// consumed.
    pub fn future_notes(&self) -> &[(NoteDetails, NoteTag)] {
        &self.future_notes
    }

    /// Returns the [`TransactionStoreUpdate`] using the submission height recorded by the
    /// pipeline via [`Self::submit_proven_transaction`].
    pub fn get_transaction_update(&self) -> Result<TransactionStoreUpdate, ClientError> {
        let executed = self
            .executed_transaction
            .clone()
            .ok_or_else(|| ClientError::from(TransactionPipelineError::NotExecuted))?;
        let submission_height = self
            .submission_height
            .ok_or_else(|| ClientError::from(TransactionPipelineError::NotSubmitted))?;

        Ok(TransactionStoreUpdate::new(
            executed,
            submission_height,
            self.future_notes.clone(),
        ))
    }

    /// Returns a [`TransactionStoreUpdate`] using the provided submission height.
    ///
    /// This is useful when the transaction has not been submitted yet but the caller still wants
    /// to persist the execution results locally (e.g. to mark a transaction as pending).
    pub fn get_transaction_update_with_height(
        &self,
        submission_height: BlockNumber,
    ) -> Result<TransactionStoreUpdate, ClientError> {
        let executed = self
            .executed_transaction
            .clone()
            .ok_or_else(|| ClientError::from(TransactionPipelineError::NotExecuted))?;

        let height = self.submission_height.unwrap_or(submission_height);

        Ok(TransactionStoreUpdate::new(executed, height, self.future_notes.clone()))
    }
}

// HELPERS
// ================================================================================================
async fn get_valid_input_notes(
    account_id: AccountId,
    mut input_notes: InputNotes<InputNote>,
    tx_args: &TransactionArgs,
    executor: &TransactionExecutor<'_, '_, ClientDataStore, impl TransactionAuthenticator + Sync>,
    block_ref: &BlockNumber,
) -> Result<InputNotes<InputNote>, ClientError> {
    loop {
        let execution = NoteConsumptionChecker::new(executor)
            .check_notes_consumability(
                account_id,
                *block_ref,
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
