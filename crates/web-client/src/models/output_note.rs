use miden_client::note::{Note as NativeNote, PartialNote as NativePartialNote};
use miden_client::transaction::RawOutputNote as NativeRawOutputNote;
use wasm_bindgen::prelude::*;

use super::note::Note;
use super::note_assets::NoteAssets;
use super::note_id::NoteId;
use super::note_metadata::NoteMetadata;
use super::partial_note::PartialNote;
use super::word::Word;
use crate::models::miden_arrays::OutputNoteArray;

/// Representation of a note produced by a transaction (full or partial).
#[derive(Clone)]
#[wasm_bindgen]
pub struct OutputNote(NativeRawOutputNote);

#[wasm_bindgen]
impl OutputNote {
    /// Wraps a full note output.
    pub fn full(note: &Note) -> OutputNote {
        let native_note: NativeNote = note.into();
        OutputNote(NativeRawOutputNote::Full(native_note))
    }

    /// Wraps a partial note containing assets and recipient only.
    pub fn partial(partial_note: &PartialNote) -> OutputNote {
        let native_partial_note: NativePartialNote = partial_note.into();
        OutputNote(NativeRawOutputNote::Partial(native_partial_note))
    }

    /// Returns the assets if they are present.
    pub fn assets(&self) -> Option<NoteAssets> {
        Some(self.0.assets().into())
    }

    /// Returns the note ID for this output.
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the recipient digest.
    #[wasm_bindgen(js_name = "recipientDigest")]
    pub fn recipient_digest(&self) -> Word {
        self.0.recipient_digest().into()
    }

    /// Returns the metadata that accompanies this output.
    pub fn metadata(&self) -> NoteMetadata {
        self.0.metadata().into()
    }

    /// Converts into a full note if the data is present.
    #[wasm_bindgen(js_name = "intoFull")]
    pub fn into_full(self) -> Option<Note> {
        match self.0 {
            NativeRawOutputNote::Full(note) => Some(note.into()),
            NativeRawOutputNote::Partial(_) => None,
        }
    }

    pub(crate) fn note(&self) -> &NativeRawOutputNote {
        &self.0
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeRawOutputNote> for OutputNote {
    fn from(raw_output_note: NativeRawOutputNote) -> Self {
        OutputNote(raw_output_note)
    }
}

impl From<&NativeRawOutputNote> for OutputNote {
    fn from(raw_output_note: &NativeRawOutputNote) -> Self {
        OutputNote(raw_output_note.clone())
    }
}

impl From<OutputNote> for NativeRawOutputNote {
    fn from(output_note: OutputNote) -> Self {
        output_note.0
    }
}

impl From<&OutputNote> for NativeRawOutputNote {
    fn from(output_note: &OutputNote) -> Self {
        output_note.0.clone()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<OutputNoteArray> for Vec<NativeRawOutputNote> {
    fn from(output_notes_array: OutputNoteArray) -> Self {
        output_notes_array.into_iter().map(Into::into).collect()
    }
}

impl From<&OutputNoteArray> for Vec<NativeRawOutputNote> {
    fn from(output_notes_array: &OutputNoteArray) -> Self {
        output_notes_array.iter().cloned().map(Into::into).collect()
    }
}
