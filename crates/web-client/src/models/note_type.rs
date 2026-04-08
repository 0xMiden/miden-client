use miden_client::note::NoteType as NativeNoteType;
use wasm_bindgen::prelude::*;

/// Visibility level for note contents when published to the network.
// Keep these masks in sync with `miden-protocol/src/note/note_type.rs`
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NoteType {
    /// Notes with this type have only their hash published to the network.
    Private = 0b10,

    /// Notes with this type are fully shared with the network.
    Public = 0b01,
}

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
