use idxdb_store::account::JsStorageMapEntry;
use miden_client::account::{AccountStorage as NativeAccountStorage, StorageSlot};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;

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

    /// Get all key-value pairs from the map slot at `index`.
    /// Returns `undefined` if the slot isn't a map or `index` is out of bounds (0-255).
    /// Returns `[]` if the map exists but is empty.
    ///
    /// WARNING: This method allocates the entire map into memory.
    /// For large maps, use `forEachMapEntry` instead for better memory efficiency.
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

    /// Stream all key-value pairs from the map slot at `index` via a callback function.
    /// This is a memory-efficient alternative to `getMapEntries` for large maps.
    ///
    /// The callback receives a `JsStorageMapEntry` object with `root`, `key`, and `value` fields.
    /// Entries are processed one at a time without allocating an intermediate vector.
    ///
    /// Returns an error if:
    /// - The slot at `index` is not a map
    /// - `index` is out of bounds (0-255)
    /// - The callback throws an error during execution
    #[wasm_bindgen(js_name = "forEachMapEntry")]
    pub fn for_each_map_entry(
        &self,
        index: u8,
        callback: &js_sys::Function,
    ) -> Result<(), JsValue> {
        let slots = self.0.slots();
        match slots.get(index as usize) {
            Some(StorageSlot::Map(map)) => {
                for (key, value) in map.entries() {
                    let entry = JsStorageMapEntry {
                        root: map.root().to_hex(),
                        key: key.to_hex(),
                        value: value.to_hex(),
                    };

                    callback
                        .call1(&JsValue::UNDEFINED, &JsValue::from(entry))
                        .map_err(|e| JsValue::from_str(&format!("Callback failed: {e:?}")))?;
                }
                Ok(())
            },
            _ => Err(JsValue::from_str("Invalid map index or slot is not a map")),
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
