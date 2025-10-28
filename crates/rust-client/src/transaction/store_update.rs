use alloc::vec::Vec;

use miden_objects::block::BlockNumber;
use miden_objects::note::{NoteDetails, NoteTag};
use miden_objects::transaction::ExecutedTransaction;
use miden_tx::utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable};

use crate::note::NoteUpdateTracker;
use crate::sync::NoteTagRecord;

// TRANSACTION STORE UPDATE
// ================================================================================================

/// Represents the changes that need to be applied to the client store as a result of a
/// transaction execution.
#[derive(Debug, Clone)]
pub struct TransactionStoreUpdate {
    /// Details of the executed transaction to be inserted.
    executed_transaction: ExecutedTransaction,
    /// Block number at which the transaction was submitted.
    submission_height: BlockNumber,
    /// Future notes that are expected to be created as a result of the transaction.
    future_notes: Vec<(NoteDetails, NoteTag)>,
    /// Information about note changes after the transaction execution.
    note_updates: NoteUpdateTracker,
    /// New note tags to be tracked.
    new_tags: Vec<NoteTagRecord>,
}

impl TransactionStoreUpdate {
    /// Creates a new [`TransactionStoreUpdate`] instance without note update information.
    ///
    /// # Arguments
    /// - `executed_transaction`: The executed transaction details.
    /// - `submission_height`: The block number at which the transaction was submitted.
    /// - `future_notes`: Notes expected to be received in follow-up transactions (e.g. swap
    ///   paybacks).
    pub fn new(
        executed_transaction: ExecutedTransaction,
        submission_height: BlockNumber,
        future_notes: Vec<(NoteDetails, NoteTag)>,
    ) -> Self {
        Self {
            executed_transaction,
            submission_height,
            future_notes,
            note_updates: NoteUpdateTracker::default(),
            new_tags: Vec::new(),
        }
    }

    /// Creates a new [`TransactionStoreUpdate`] populated with note update information.
    pub fn with_note_updates(
        executed_transaction: ExecutedTransaction,
        submission_height: BlockNumber,
        future_notes: Vec<(NoteDetails, NoteTag)>,
        note_updates: NoteUpdateTracker,
        new_tags: Vec<NoteTagRecord>,
    ) -> Self {
        Self {
            executed_transaction,
            submission_height,
            future_notes,
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

    /// Returns the future notes that should be tracked as a result of the transaction.
    pub fn future_notes(&self) -> &[(NoteDetails, NoteTag)] {
        &self.future_notes
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

impl Serializable for TransactionStoreUpdate {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.executed_transaction.write_into(target);
        self.submission_height.write_into(target);
        self.future_notes.write_into(target);
    }
}

impl Deserializable for TransactionStoreUpdate {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let executed_transaction = ExecutedTransaction::read_from(source)?;
        let submission_height = BlockNumber::read_from(source)?;
        let future_notes = Vec::<(NoteDetails, NoteTag)>::read_from(source)?;

        Ok(Self {
            executed_transaction,
            submission_height,
            future_notes,
            note_updates: NoteUpdateTracker::default(),
            new_tags: Vec::new(),
        })
    }
}
