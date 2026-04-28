use js_export_macro::js_export;
use miden_client::note::NoteType as NativeNoteType;

/// Visibility level for note contents when published to the network.
#[js_export]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NoteType {
    /// Notes with this type have only their hash published to the network.
    Private = 0b10,

    /// Notes with this type are fully shared with the network.
    Public = 0b01,
}

// Compile-time check to keep enum values aligned.
const _: () = {
    assert!(NativeNoteType::Private as u8 == NoteType::Private as u8);
    assert!(NativeNoteType::Public as u8 == NoteType::Public as u8);
};

impl From<NativeNoteType> for NoteType {
    fn from(value: NativeNoteType) -> Self {
        match value {
            NativeNoteType::Private => NoteType::Private,
            NativeNoteType::Public => NoteType::Public,
        }
    }
}

impl From<NoteType> for NativeNoteType {
    fn from(value: NoteType) -> Self {
        match value {
            NoteType::Private => NativeNoteType::Private,
            NoteType::Public => NativeNoteType::Public,
        }
    }
}
