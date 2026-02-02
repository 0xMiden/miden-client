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
///
/// Note: This type is named `NoteInputs` in the JavaScript API for backwards compatibility,
/// but internally uses `NoteStorage` from miden-protocol.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteInputs(NativeNoteStorage);

#[wasm_bindgen]
impl NoteInputs {
    /// Creates note storage from a list of field elements.
    #[wasm_bindgen(constructor)]
    pub fn new(felt_array: &FeltArray) -> NoteInputs {
        let native_felts = felt_array.into();
        let native_note_storage = NativeNoteStorage::new(native_felts).unwrap();
        NoteInputs(native_note_storage)
    }

    /// Returns the raw storage items as an array of field elements.
    pub fn values(&self) -> Vec<Felt> {
        self.0.items().iter().map(Into::into).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteStorage> for NoteInputs {
    fn from(native_note_storage: NativeNoteStorage) -> Self {
        NoteInputs(native_note_storage)
    }
}

impl From<&NativeNoteStorage> for NoteInputs {
    fn from(native_note_storage: &NativeNoteStorage) -> Self {
        NoteInputs(native_note_storage.clone())
    }
}

impl From<NoteInputs> for NativeNoteStorage {
    fn from(note_inputs: NoteInputs) -> Self {
        note_inputs.0
    }
}

impl From<&NoteInputs> for NativeNoteStorage {
    fn from(note_inputs: &NoteInputs) -> Self {
        note_inputs.0.clone()
    }
}
