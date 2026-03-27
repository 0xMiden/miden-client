use miden_client::note::Note as NativeNote;
use miden_client::transaction::RawOutputNote as NativeRawOutputNote;
use wasm_bindgen::prelude::*;

use super::note::Note;
use super::note_assets::NoteAssets;
use super::note_id::NoteId;
use super::note_metadata::NoteMetadata;
use super::word::Word;
use crate::models::miden_arrays::RawOutputNoteArray;

/// Representation of a note produced by a transaction (full or partial).
#[derive(Clone)]
#[wasm_bindgen]
pub struct RawOutputNote(NativeRawOutputNote);

#[wasm_bindgen]
impl RawOutputNote {
    /// Wraps a full note output.
    pub fn full(note: &Note) -> RawOutputNote {
        let native_note: NativeNote = note.into();
        RawOutputNote(NativeRawOutputNote::Full(native_note))
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

impl From<NativeRawOutputNote> for RawOutputNote {
    fn from(raw_output_note: NativeRawOutputNote) -> Self {
        RawOutputNote(raw_output_note)
    }
}

impl From<&NativeRawOutputNote> for RawOutputNote {
    fn from(raw_output_note: &NativeRawOutputNote) -> Self {
        RawOutputNote(raw_output_note.clone())
    }
}

impl From<RawOutputNote> for NativeRawOutputNote {
    fn from(output_note: RawOutputNote) -> Self {
        output_note.0
    }
}

impl From<&RawOutputNote> for NativeRawOutputNote {
    fn from(output_note: &RawOutputNote) -> Self {
        output_note.0.clone()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<RawOutputNoteArray> for Vec<NativeRawOutputNote> {
    fn from(output_notes_array: RawOutputNoteArray) -> Self {
        output_notes_array.__inner.into_iter().map(Into::into).collect()
    }
}

impl From<&RawOutputNoteArray> for Vec<NativeRawOutputNote> {
    fn from(output_notes_array: &RawOutputNoteArray) -> Self {
        output_notes_array.__inner.iter().cloned().map(Into::into).collect()
    }
}
