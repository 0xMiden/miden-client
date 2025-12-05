use miden_client::transaction::InputNote as NativeInputNote;
use wasm_bindgen::prelude::*;

use super::note::Note;
use super::note_id::NoteId;
use super::note_inclusion_proof::NoteInclusionProof;
use super::note_location::NoteLocation;
use super::word::Word;

/// Note supplied as an input to a transaction, optionally with authentication data.
#[derive(Clone)]
#[wasm_bindgen]
pub struct InputNote(pub(crate) NativeInputNote);

#[wasm_bindgen]
impl InputNote {
    // TODO: authenticated constructor

    // TODO: unauthenticated constructor

    /// Returns the identifier of the input note.
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the underlying note contents.
    pub fn note(&self) -> Note {
        self.0.note().into()
    }

    /// Returns the commitment to the note ID and metadata.
    pub fn commitment(&self) -> Word {
        self.0.note().commitment().into()
    }

    /// Returns the inclusion proof if the note is authenticated.
    pub fn proof(&self) -> Option<NoteInclusionProof> {
        self.0.proof().map(Into::into)
    }

    /// Returns the note's location within the commitment tree when available.
    pub fn location(&self) -> Option<NoteLocation> {
        self.0.location().map(Into::into)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeInputNote> for InputNote {
    fn from(native_note: NativeInputNote) -> Self {
        InputNote(native_note)
    }
}

impl From<&NativeInputNote> for InputNote {
    fn from(native_note: &NativeInputNote) -> Self {
        InputNote(native_note.clone())
    }
}
