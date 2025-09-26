use idxdb_store::account::JsStorageMapEntry;
use miden_objects::account::{AccountStorage as NativeAccountStorage, StorageSlot};
use wasm_bindgen::prelude::*;

use crate::models::word::Word;

#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountStorage(NativeAccountStorage);

#[wasm_bindgen]
impl AccountStorage {
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    #[wasm_bindgen(js_name = "getItem")]
    pub fn get_item(&self, index: u8) -> Option<Word> {
        self.0.get_item(index).ok().map(Into::into)
    }

    #[wasm_bindgen(js_name = "getMapItem")]
    pub fn get_map_item(&self, index: u8, key: &Word) -> Option<Word> {
        self.0.get_map_item(index, key.into()).ok().map(Into::into)
    }

    /// Returns all entries from the storage map at the given index.
    /// Returns None if the slot is not a map or if the index is out of bounds.
    #[wasm_bindgen(js_name = "getMapEntries")]
    pub fn get_map_entries(&self, index: u8) -> Option<Vec<JsStorageMapEntry>> {
        let slots = self.0.slots();
        match slots.get(index as usize) {
            Some(StorageSlot::Map(map)) => Some(
                map.entries()
                    .map(|(key, value)| JsStorageMapEntry {
                        root: map.root().to_hex(),
                        key: key.to_hex(),
                        value: value.to_hex(),
                    })
                    .collect(),
            ),
            _ => None,
        }
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountStorage> for AccountStorage {
    fn from(native_account_storage: NativeAccountStorage) -> Self {
        AccountStorage(native_account_storage)
    }
}

impl From<&NativeAccountStorage> for AccountStorage {
    fn from(native_account_storage: &NativeAccountStorage) -> Self {
        AccountStorage(native_account_storage.clone())
    }
}
