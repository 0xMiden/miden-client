use js_export_macro::js_export;
use miden_client::note::NoteStorage as NativeNoteStorage;

use super::felt::Felt;
use crate::models::miden_arrays::FeltArray;
use crate::platform::{JsErr, from_str_err};

/// A container for note storage items.
///
/// A note can be associated with up to 1024 storage items. Each item is represented by a single
/// field element. Thus, note storage can contain up to ~8 KB of data.
///
/// All storage items associated with a note can be reduced to a single commitment which is
/// computed as an RPO256 hash over the storage elements.
#[derive(Clone)]
#[js_export]
pub struct NoteStorage(NativeNoteStorage);

#[js_export]
impl NoteStorage {
    /// Creates note storage from a list of field elements.
    #[js_export(constructor)]
    pub fn new(felt_array: FeltArray) -> Result<NoteStorage, JsErr> {
        let native_felts = super::felt::felt_array_to_native_vec(&felt_array);
        let native_note_storage = NativeNoteStorage::new(native_felts)
            .map_err(|err| from_str_err(&format!("Invalid note storage: {err}")))?;
        Ok(NoteStorage(native_note_storage))
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
