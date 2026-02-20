use miden_client::note::NoteAttachmentKind as NativeNoteAttachmentKind;

use crate::prelude::*;

/// Defines the payload shape of a note attachment.
#[bindings(wasm(derive(Clone, Copy)), napi(string_enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteAttachmentKind {
    None,
    Word,
    Array,
}

// Compile-time check to keep enum values aligned.
#[cfg(feature = "wasm")]
const _: () = {
    assert!(NativeNoteAttachmentKind::None as u8 == NoteAttachmentKind::None as u8);
    assert!(NativeNoteAttachmentKind::Word as u8 == NoteAttachmentKind::Word as u8);
    assert!(NativeNoteAttachmentKind::Array as u8 == NoteAttachmentKind::Array as u8);
};

impl From<NativeNoteAttachmentKind> for NoteAttachmentKind {
    fn from(value: NativeNoteAttachmentKind) -> Self {
        match value {
            NativeNoteAttachmentKind::None => NoteAttachmentKind::None,
            NativeNoteAttachmentKind::Word => NoteAttachmentKind::Word,
            NativeNoteAttachmentKind::Array => NoteAttachmentKind::Array,
        }
    }
}

impl From<&NativeNoteAttachmentKind> for NoteAttachmentKind {
    fn from(value: &NativeNoteAttachmentKind) -> Self {
        (*value).into()
    }
}

impl From<NoteAttachmentKind> for NativeNoteAttachmentKind {
    fn from(value: NoteAttachmentKind) -> Self {
        match value {
            NoteAttachmentKind::None => NativeNoteAttachmentKind::None,
            NoteAttachmentKind::Word => NativeNoteAttachmentKind::Word,
            NoteAttachmentKind::Array => NativeNoteAttachmentKind::Array,
        }
    }
}

impl From<&NoteAttachmentKind> for NativeNoteAttachmentKind {
    fn from(value: &NoteAttachmentKind) -> Self {
        (*value).into()
    }
}
