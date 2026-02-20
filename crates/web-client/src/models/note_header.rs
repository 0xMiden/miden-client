use miden_client::note::NoteHeader as NativeNoteHeader;

use super::note_id::NoteId;
use super::note_metadata::NoteMetadata;
use super::word::Word;
use crate::prelude::*;

/// Holds the strictly required, public information of a note.
///
/// See `NoteId` and `NoteMetadata` for additional details.
#[bindings]
#[derive(Clone)]
pub struct NoteHeader(NativeNoteHeader);

#[bindings]
impl NoteHeader {
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    pub fn metadata(&self) -> NoteMetadata {
        self.0.metadata().into()
    }

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
        NoteHeader(native_note_header.clone())
    }
}

impl From<NoteHeader> for NativeNoteHeader {
    fn from(note_header: NoteHeader) -> Self {
        note_header.0
    }
}

impl From<&NoteHeader> for NativeNoteHeader {
    fn from(note_header: &NoteHeader) -> Self {
        note_header.0.clone()
    }
}
