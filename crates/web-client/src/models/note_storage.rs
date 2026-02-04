use miden_client::note::NoteStorage as NativeNoteStorage;
use wasm_bindgen::prelude::*;

use super::felt::Felt;
use crate::models::miden_arrays::FeltArray;

/// A container for note storage items.
///
/// A note can be associated with up to 1024 storage items. Each item is represented by a single
/// field element. Thus, note storage can contain up to ~8 KB of data.
///
/// All storage items associated with a note can be reduced to a single commitment which is
/// computed as an RPO256 hash over the storage elements.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteStorage(NativeNoteStorage);

#[wasm_bindgen]
impl NoteStorage {
    /// Creates note storage from a list of field elements.
    #[wasm_bindgen(constructor)]
    pub fn new(felt_array: &FeltArray) -> NoteStorage {
        let native_felts = felt_array.into();
        let native_note_storage = NativeNoteStorage::new(native_felts).unwrap();
        NoteStorage(native_note_storage)
    }

    /// Returns the raw storage items as an array of field elements.
    pub fn items(&self) -> Vec<Felt> {
        self.0.items().iter().map(Into::into).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteStorage> for NoteStorage {
    fn from(native_note_storage: NativeNoteStorage) -> Self {
        NoteStorage(native_note_storage)
    }
}

impl From<&NativeNoteStorage> for NoteStorage {
    fn from(native_note_storage: &NativeNoteStorage) -> Self {
        NoteStorage(native_note_storage.clone())
    }
}

impl From<NoteStorage> for NativeNoteStorage {
    fn from(note_storage: NoteStorage) -> Self {
        note_storage.0
    }
}

impl From<&NoteStorage> for NativeNoteStorage {
    fn from(note_storage: &NoteStorage) -> Self {
        note_storage.0.clone()
    }
}
