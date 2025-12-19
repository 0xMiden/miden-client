use miden_client::note::{Note as NativeNote, NoteInclusionProof as NativeNoteInclusionProof};
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
    /// Creates an authenticated input note from a note and its inclusion proof.
    ///
    /// An authenticated note has a proof of inclusion in the block's note tree,
    /// which is required for consuming the note in a transaction.
    pub fn authenticated(note: &Note, inclusion_proof: &NoteInclusionProof) -> InputNote {
        let native_note: NativeNote = note.into();
        let native_proof: NativeNoteInclusionProof = inclusion_proof.clone().into();
        InputNote(NativeInputNote::authenticated(native_note, native_proof))
    }

    /// Creates an unauthenticated input note from note details.
    ///
    /// An unauthenticated note can be consumed in a transaction as long as the note exists in the
    /// network as of the transaction batch in which the consume transaction is included.
    pub fn unauthenticated(note: &Note) -> InputNote {
        InputNote(NativeInputNote::unauthenticated(note.clone().into()))
    }

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
