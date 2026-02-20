use miden_client::note::NoteType as NativeNoteType;

use crate::prelude::*;

/// Visibility level for note contents when published to the network.
#[bindings(wasm(derive(Clone, Copy)), napi(string_enum))]
#[derive(Debug, PartialEq, Eq)]
pub enum NoteType {
    /// Notes with this type have only their hash published to the network.
    Private,
    /// Notes with this type are fully shared with the network.
    Public,
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
