use miden_client::rpc::domain::note::CommittedNote as NativeCommittedNote;
use wasm_bindgen::prelude::*;

use super::note_id::NoteId;
use super::note_inclusion_proof::NoteInclusionProof;
use super::note_metadata::NoteMetadata;
use super::note_type::NoteType;

/// Represents a note committed on chain.
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

    /// Returns the note type (public, private, etc.).
    #[wasm_bindgen(js_name = "noteType")]
    pub fn note_type(&self) -> NoteType {
        self.0.note_type().into()
    }

    /// Returns the note tag.
    pub fn tag(&self) -> u32 {
        self.0.tag().as_u32()
    }

    /// Returns the note metadata, if available.
    pub fn metadata(&self) -> Option<NoteMetadata> {
        self.0.metadata().map(Into::into)
    }

    /// Returns the inclusion proof for this note.
    #[wasm_bindgen(js_name = "inclusionProof")]
    pub fn inclusion_proof(&self) -> NoteInclusionProof {
        self.0.inclusion_proof().into()
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
