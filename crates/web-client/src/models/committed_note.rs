use miden_client::rpc::domain::note::CommittedNote as NativeCommittedNote;
use wasm_bindgen::prelude::*;

use super::note_id::NoteId;
use super::note_metadata::NoteMetadata;
use super::sparse_merkle_path::SparseMerklePath;

/// Represents a note committed on chain, as returned by `syncNotes`.
#[derive(Clone)]
#[wasm_bindgen]
pub struct CommittedNote(NativeCommittedNote);

#[wasm_bindgen]
impl CommittedNote {
    /// Returns the note ID.
    #[wasm_bindgen(js_name = "noteId")]
    pub fn note_id(&self) -> NoteId {
        (*self.0.note_id()).into()
    }

    /// Returns the note index in the block's note tree.
    #[wasm_bindgen(js_name = "noteIndex")]
    pub fn note_index(&self) -> u16 {
        self.0.note_index()
    }

    /// Returns the inclusion path for the note.
    #[wasm_bindgen(js_name = "inclusionPath")]
    pub fn inclusion_path(&self) -> SparseMerklePath {
        self.0.inclusion_path().into()
    }

    /// Returns the note metadata.
    pub fn metadata(&self) -> NoteMetadata {
        self.0.metadata().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeCommittedNote> for CommittedNote {
    fn from(native_note: NativeCommittedNote) -> Self {
        CommittedNote(native_note)
    }
}

impl From<&NativeCommittedNote> for CommittedNote {
    fn from(native_note: &NativeCommittedNote) -> Self {
        CommittedNote(native_note.clone())
    }
}
