//! Provides a lazy iterator over consumed input notes.

use alloc::sync::Arc;

use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;

use crate::ClientError;
use crate::store::{InputNoteRecord, NoteFilter, Store};

/// A lazy iterator over consumed input notes.
///
/// Each call to [`InputNoteReader::next`] executes a store query and returns the
/// next matching note. Use builder methods to configure filters before
/// iterating.
pub struct InputNoteReader {
    store: Arc<dyn Store>,
    consumer: Option<AccountId>,
    block_start: Option<BlockNumber>,
    block_end: Option<BlockNumber>,
    offset: u32,
}

impl InputNoteReader {
    /// Creates a new `InputNoteReader` that iterates over consumed input notes.
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self {
            store,
            consumer: None,
            block_start: None,
            block_end: None,
            offset: 0,
        }
    }

    /// Filters notes by consumer account ID.
    #[must_use]
    pub fn for_consumer(mut self, account_id: AccountId) -> Self {
        self.consumer = Some(account_id);
        self
    }

    /// Restricts iteration to notes consumed at or after the given block.
    #[must_use]
    pub fn from_block(mut self, block: BlockNumber) -> Self {
        self.block_start = Some(block);
        self
    }

    /// Restricts iteration to notes consumed at or before the given block.
    #[must_use]
    pub fn to_block(mut self, block: BlockNumber) -> Self {
        self.block_end = Some(block);
        self
    }

    /// Returns the next consumed input note, or `None` when all matching notes have been
    /// returned.
    ///
    /// Each call executes a single store query.
    pub async fn next(&mut self) -> Result<Option<InputNoteRecord>, ClientError> {
        // TODO: The note filter should be configurable instead of hardcoding
        // `NoteFilter::Consumed`. This would allow iterating over input notes in any state
        // while keeping the lazy access.
        let note = self
            .store
            .get_input_note_by_offset(
                NoteFilter::Consumed,
                self.consumer,
                self.block_start,
                self.block_end,
                self.offset,
            )
            .await
            .map_err(ClientError::StoreError)?;

        if note.is_some() {
            self.offset += 1;
        }
        Ok(note)
    }

    /// Resets the reader to the beginning.
    pub fn reset(&mut self) {
        self.offset = 0;
    }
}
