#[cfg(feature = "napi")]
use miden_client::Felt as NativeFelt;
use miden_client::note::NoteStorage as NativeNoteStorage;

use super::felt::Felt;
#[cfg(feature = "wasm")]
use crate::models::miden_arrays::FeltArray;
use crate::prelude::*;

/// A container for note storage items.
///
/// A note can be associated with up to 1024 storage items. Each item is represented by a single
/// field element. Thus, note storage can contain up to ~8 KB of data.
///
/// All storage items associated with a note can be reduced to a single commitment which is
/// computed as an RPO256 hash over the storage elements.
#[bindings]
#[derive(Clone)]
pub struct NoteStorage(pub(crate) NativeNoteStorage);

#[bindings]
impl NoteStorage {
    #[bindings]
    pub fn items(&self) -> Vec<Felt> {
        self.0.items().iter().map(Into::into).collect()
    }
}

// Platform-specific constructors that differ
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl NoteStorage {
    #[wasm_bindgen(constructor)]
    pub fn new(felt_array: &FeltArray) -> NoteStorage {
        let wrapper_felts: Vec<Felt> = felt_array.into();
        let native_felts: Vec<miden_client::Felt> = wrapper_felts.into_iter().map(Into::into).collect();
        let native_note_storage = NativeNoteStorage::new(native_felts).unwrap();
        NoteStorage(native_note_storage)
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl NoteStorage {
    #[napi(constructor)]
    pub fn new(felts: Vec<&Felt>) -> JsResult<NoteStorage> {
        let native_felts: Vec<NativeFelt> = felts.into_iter().map(|f| f.into()).collect();
        let native_note_storage = NativeNoteStorage::new(native_felts)
            .map_err(|e| platform::error_with_context(e, "Error creating NoteStorage"))?;
        Ok(NoteStorage(native_note_storage))
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
