use miden_client::note::NoteAttachmentKind as NativeNoteAttachmentKind;
use wasm_bindgen::prelude::*;

/// Defines the payload shape of a note attachment.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NoteAttachmentKind {
    None = 0,
    Word = 1,
    Array = 2,
}

// Compile-time check to keep enum values aligned.
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
