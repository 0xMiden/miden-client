use alloc::vec::Vec;

use miden_objects::block::BlockNumber;
use miden_objects::transaction::ExecutedTransaction;

use crate::note::NoteUpdateTracker;
use crate::sync::NoteTagRecord;

// TRANSACTION STORE UPDATE
// ================================================================================================

/// Represents the changes that need to be applied to the client store as a result of a
/// transaction execution.
pub struct TransactionStoreUpdate {
    /// Details of the executed transaction to be inserted.
    executed_transaction: ExecutedTransaction,
    /// Block number at which the transaction was submitted.
    submission_height: BlockNumber,
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
    /// - `note_updates`: The note updates that need to be applied to the store after the
    ///   transaction execution.
    /// - `new_tags`: New note tags that were need to be tracked because of created notes.
    pub fn new(
        executed_transaction: ExecutedTransaction,
        submission_height: BlockNumber,
        note_updates: NoteUpdateTracker,
        new_tags: Vec<NoteTagRecord>,
    ) -> Self {
        Self {
            executed_transaction,
            submission_height,
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

    /// Returns the note updates that need to be applied after the transaction execution.
    pub fn note_updates(&self) -> &NoteUpdateTracker {
        &self.note_updates
    }

    /// Returns the new tags that were created as part of the transaction.
    pub fn new_tags(&self) -> &[NoteTagRecord] {
        &self.new_tags
    }
}
