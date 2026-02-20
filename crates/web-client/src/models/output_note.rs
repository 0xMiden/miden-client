use miden_client::note::{
    Note as NativeNote,
    NoteHeader as NativeNoteHeader,
    PartialNote as NativePartialNote,
};
use miden_client::transaction::OutputNote as NativeOutputNote;

use crate::prelude::*;

use super::note::Note;
use super::note_assets::NoteAssets;
use super::note_header::NoteHeader;
use super::note_id::NoteId;
use super::note_metadata::NoteMetadata;
use super::partial_note::PartialNote;
use super::word::Word;

/// Representation of a note produced by a transaction (full, partial, or header-only).
#[derive(Clone)]
#[bindings]
pub struct OutputNote(NativeOutputNote);

// Shared methods (identical signatures and bodies across wasm/napi)
#[bindings]
impl OutputNote {
    /// Wraps a full note output.
    #[bindings(napi(factory))]
    pub fn full(note: &Note) -> OutputNote {
        let native_note: NativeNote = note.into();
        OutputNote(NativeOutputNote::Full(native_note))
    }

    /// Wraps a partial note containing assets and recipient only.
    #[bindings(napi(factory))]
    pub fn partial(partial_note: &PartialNote) -> OutputNote {
        let native_partial_note: NativePartialNote = partial_note.into();
        OutputNote(NativeOutputNote::Partial(native_partial_note))
    }

    /// Wraps only the header of a note.
    #[bindings(napi(factory))]
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
    #[bindings(wasm)]
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

    pub(crate) fn note(&self) -> &NativeOutputNote {
        &self.0
    }
}

// wasm-specific methods (self by value)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl OutputNote {
    /// Converts into a full note if the data is present.
    
    pub fn into_full(self) -> Option<Note> {
        match self.0 {
            NativeOutputNote::Full(note) => Some(note.into()),
            _ => None,
        }
    }
}

// napi-specific methods (self by ref with clone)
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl OutputNote {
    /// Converts into a full note if the data is present.
    #[napi]
    pub fn into_full(&self) -> Option<Note> {
        match self.0.clone() {
            NativeOutputNote::Full(note) => Some(note.into()),
            _ => None,
        }
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
