use std::vec::Vec;

use miden_objects::note::NoteId;

use crate::{Client, ClientError};

impl Client {
    /// Waits for the specified notes to be committed.
    ///
    /// # Panics
    /// - If any of the specified notes is consumed.
    pub async fn wait_for_notes_committed(
        &mut self,
        mut note_ids: Vec<NoteId>,
    ) -> Result<(), ClientError> {
        self.wait_until(|summary| {
            // Remove notes that were committed
            note_ids.retain(|id| {
                !summary.committed_notes.iter().any(|committed_id| committed_id == id)
            });

            // If a note was consumed, panic
            assert!(
                !note_ids.iter().any(|id| summary.consumed_notes.contains(id)),
                "A note was consumed before it was committed"
            );

            note_ids.is_empty()
        })
        .await
    }

    /// Waits for the specified notes to be consumed.
    pub async fn wait_for_notes_consumed(
        &mut self,
        mut note_ids: Vec<NoteId>,
    ) -> Result<(), ClientError> {
        self.wait_until(|summary| {
            // Remove notes that were consumed
            note_ids.retain(|id| !summary.consumed_notes.contains(id));

            note_ids.is_empty()
        })
        .await
    }
}
