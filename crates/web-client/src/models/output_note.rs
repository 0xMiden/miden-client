use miden_client::note::{
    Note as NativeNote,
    NoteHeader as NativeNoteHeader,
    PartialNote as NativePartialNote,
};
use miden_client::transaction::OutputNote as NativeOutputNote;
use wasm_bindgen::prelude::*;

use super::note::Note;
use super::note_assets::NoteAssets;
use super::note_header::NoteHeader;
use super::note_id::NoteId;
use super::note_metadata::NoteMetadata;
use super::partial_note::PartialNote;
use super::word::Word;
use crate::models::miden_arrays::OutputNoteArray;

/// Representation of a note produced by a transaction (full, partial, or header-only).
#[derive(Clone)]
#[wasm_bindgen]
pub struct OutputNote(NativeOutputNote);

#[wasm_bindgen]
impl OutputNote {
    /// Wraps a full note output.
    pub fn full(note: &Note) -> OutputNote {
        let native_note: NativeNote = note.into();
        OutputNote(NativeOutputNote::Full(native_note))
    }

    /// Wraps a partial note containing assets and recipient only.
    pub fn partial(partial_note: &PartialNote) -> OutputNote {
        let native_partial_note: NativePartialNote = partial_note.into();
        OutputNote(NativeOutputNote::Partial(native_partial_note))
    }

    /// Wraps only the header of a note.
    pub fn header(note_header: &NoteHeader) -> OutputNote {
        let native_note_header: NativeNoteHeader = note_header.into();
        OutputNote(NativeOutputNote::Header(native_note_header))
    }

    /// Returns the assets if they are present.
    pub fn assets(&self) -> Option<NoteAssets> {
        self.0.assets().map(Into::into)
    }

    /// Returns the note ID for this output.
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the recipient digest if the recipient is known.
    #[wasm_bindgen(js_name = "recipientDigest")]
    pub fn recipient_digest(&self) -> Option<Word> {
        self.0.recipient_digest().map(Into::into)
    }

    /// Returns the metadata that accompanies this output.
    pub fn metadata(&self) -> NoteMetadata {
        self.0.metadata().into()
    }

    /// Returns a more compact representation if possible (e.g. dropping details).
    #[must_use]
    pub fn shrink(&self) -> OutputNote {
        self.0.shrink().into()
    }

    /// Converts into a full note if the data is present.
    #[wasm_bindgen(js_name = "intoFull")]
    pub fn into_full(self) -> Option<Note> {
        match self.0 {
            NativeOutputNote::Full(note) => Some(note.into()),
            _ => None,
        }
    }

    pub(crate) fn note(&self) -> &NativeOutputNote {
        &self.0
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeOutputNote> for OutputNote {
    fn from(native_output_note: NativeOutputNote) -> Self {
        OutputNote(native_output_note)
    }
}

impl From<&NativeOutputNote> for OutputNote {
    fn from(native_output_note: &NativeOutputNote) -> Self {
        OutputNote(native_output_note.clone())
    }
}

impl From<OutputNote> for NativeOutputNote {
    fn from(output_note: OutputNote) -> Self {
        output_note.0
    }
}

impl From<&OutputNote> for NativeOutputNote {
    fn from(output_note: &OutputNote) -> Self {
        output_note.0.clone()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<OutputNoteArray> for Vec<NativeOutputNote> {
    fn from(output_notes_array: OutputNoteArray) -> Self {
        output_notes_array.__inner.into_iter().map(Into::into).collect()
    }
}

impl From<&OutputNoteArray> for Vec<NativeOutputNote> {
    fn from(output_notes_array: &OutputNoteArray) -> Self {
        output_notes_array.__inner.iter().map(Into::into).collect()
    }
}
