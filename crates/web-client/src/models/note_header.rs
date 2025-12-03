use miden_client::note::NoteHeader as NativeNoteHeader;
use wasm_bindgen::prelude::*;

use super::note_id::NoteId;
use super::note_metadata::NoteMetadata;
use super::word::Word;

/// Public portion of a note containing its ID and metadata commitment.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteHeader(NativeNoteHeader);

#[wasm_bindgen]
impl NoteHeader {
    // TODO: new()

    /// Returns the unique identifier for the note.
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the public metadata attached to the note.
    pub fn metadata(&self) -> NoteMetadata {
        self.0.metadata().into()
    }

    /// Returns a commitment to the note ID and metadata.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteHeader> for NoteHeader {
    fn from(native_note_header: NativeNoteHeader) -> Self {
        NoteHeader(native_note_header)
    }
}

impl From<&NativeNoteHeader> for NoteHeader {
    fn from(native_note_header: &NativeNoteHeader) -> Self {
        NoteHeader(*native_note_header)
    }
}

impl From<NoteHeader> for NativeNoteHeader {
    fn from(note_header: NoteHeader) -> Self {
        note_header.0
    }
}

impl From<&NoteHeader> for NativeNoteHeader {
    fn from(note_header: &NoteHeader) -> Self {
        note_header.0
    }
}
