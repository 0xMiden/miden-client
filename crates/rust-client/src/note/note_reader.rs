//! Provides a lazy iterator over output notes.

use alloc::sync::Arc;

use miden_protocol::account::AccountId;

use crate::ClientError;
use crate::store::{NoteFilter, OutputNoteRecord, Store};

/// A lazy iterator over output notes.
///
/// Each call to [`OutputNoteReader::next`] executes a store query and returns the
/// next matching note. Use builder methods to configure filters before
/// iterating.
pub struct OutputNoteReader {
    store: Arc<dyn Store>,
    status: NoteFilter,
    sender: Option<AccountId>,
    offset: u32,
}

impl OutputNoteReader {
    /// Creates a new `OutputNoteReader` for output notes with the given status filter.
    pub fn new(store: Arc<dyn Store>, status: NoteFilter) -> Self {
        Self { store, status, sender: None, offset: 0 }
    }

    /// Filters notes by sender account ID.
    #[must_use]
    pub fn for_sender(mut self, account_id: AccountId) -> Self {
        self.sender = Some(account_id);
        self
    }

    /// Returns the next output note, or `None` when all matching notes have been returned.
    ///
    /// Each call executes a single store query.
    pub async fn next(&mut self) -> Result<Option<OutputNoteRecord>, ClientError> {
        let note = self
            .store
            .get_output_note_by_offset(self.status.clone(), self.sender, self.offset)
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
