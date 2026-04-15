//! Provides a lazy iterator over consumed input notes.

use alloc::sync::Arc;

use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;

use crate::ClientError;
use crate::store::{InputNoteRecord, NoteFilter, Store};

/// A lazy iterator over consumed input notes for a specific consumer account.
///
/// Each call to [`InputNoteReader::next`] executes a store query and returns the
/// next matching note. Use builder methods to configure filters before iterating.
///
/// # Ordering
///
/// Notes are returned in on-chain consumption order: first by block number, then by
/// per-account transaction order within the block.
pub struct InputNoteReader {
    store: Arc<dyn Store>,
    consumer: AccountId,
    block_range: Option<(BlockNumber, BlockNumber)>,
    offset: u32,
}

impl InputNoteReader {
    /// Creates a new `InputNoteReader` that iterates over consumed input notes
    /// for the given consumer account.
    ///
    /// The consumer is required because ordering is only guaranteed among notes
    /// consumed by the same account.
    pub fn new(store: Arc<dyn Store>, consumer: AccountId) -> Self {
        Self {
            store,
            consumer,
            block_range: None,
            offset: 0,
        }
    }

    /// Restricts iteration to notes consumed within the given block range (inclusive).
    #[must_use]
    pub fn in_block_range(mut self, from: BlockNumber, to: BlockNumber) -> Self {
        self.block_range = Some((from, to));
        self
    }

    /// Resets the iterator to the beginning.
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Returns the next consumed input note, or `None` when all matching notes have been
    /// returned.
    ///
    /// Each call executes a single store query.
    pub async fn next(&mut self) -> Result<Option<InputNoteRecord>, ClientError> {
        let (block_start, block_end) = match self.block_range {
            Some((from, to)) => (Some(from), Some(to)),
            None => (None, None),
        };

        // TODO: The note filter should be configurable instead of hardcoding `NoteFilter::Consumed`
        let note = self
            .store
            .get_input_note_by_offset(
                NoteFilter::Consumed,
                self.consumer,
                block_start,
                block_end,
                self.offset,
            )
            .await
            .map_err(ClientError::StoreError)?;

        if note.is_some() {
            self.offset += 1;
        }
        Ok(note)
    }
}
