use idxdb_store::account::JsStorageMapEntry;
use miden_client::account::{
    AccountStorage as NativeAccountStorage,
    StorageSlotContent,
    StorageSlotName,
};
use wasm_bindgen::prelude::*;

use crate::models::word::Word;

/// Account storage is composed of a variable number of index-addressable storage slots up to 255
/// slots in total.
///
/// Each slot has a type which defines its size and structure. Currently, the following types are
/// supported:
/// - `StorageSlot::Value`: contains a single Word of data (i.e., 32 bytes).
/// - `StorageSlot::Map`: contains a `StorageMap` which is a key-value map where both keys and
///   values are Words. The value of a storage slot containing a map is the commitment to the
///   underlying map.
#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountStorage(NativeAccountStorage);

#[wasm_bindgen]
impl AccountStorage {
    /// Returns the commitment to the full account storage.
    pub fn commitment(&self) -> Word {
        self.0.to_commitment().into()
    }

    /// Returns the value stored at the given slot name.
    /// For Value slots: returns the stored Word directly.
    /// For Map slots: returns the first entry's value (NOT the commitment hash).
    #[wasm_bindgen(js_name = "getItem")]
    pub fn get_item(&self, slot_name: &str) -> Option<Word> {
        let name = StorageSlotName::new(slot_name).ok()?;
        let slot = self.0.slots().iter().find(|s| s.name() == &name)?;

        match slot.content() {
            StorageSlotContent::Value(_) => self.0.get_item(&name).ok().map(Into::into),
            StorageSlotContent::Map(map) => {
                // Return first entry's value instead of the useless commitment hash
                map.entries().next().map(|(_, value)| value.into())
            },
        }
    }

    /// Returns the first felt of a storage slot as a number.
    /// Works for both Value and Map slots (for maps, returns first entry's first felt).
    #[wasm_bindgen(js_name = "getNumber")]
    #[allow(clippy::cast_precision_loss)]
    pub fn get_number(&self, slot_name: &str) -> Option<f64> {
        let word = self.get_item(slot_name)?;
        let native_word: miden_client::Word = word.into();
        Some(native_word[0].as_int() as f64)
    }

    /// Returns the names of all storage slots on this account.
    #[wasm_bindgen(js_name = "getSlotNames")]
    pub fn get_slot_names(&self) -> Vec<String> {
        self.0.slots().iter().map(|slot| slot.name().as_str().to_string()).collect()
    }

    /// Returns the value for a key in the map stored at the given slot, if any.
    #[wasm_bindgen(js_name = "getMapItem")]
    pub fn get_map_item(&self, slot_name: &str, key: &Word) -> Option<Word> {
        match StorageSlotName::new(slot_name) {
            Ok(slot_name) => self.0.get_map_item(&slot_name, key.into()).ok().map(Into::into),
            Err(_) => None,
        }
    }

    /// Smart read: returns the actual stored value regardless of slot type.
    /// - For Value slots: returns the stored Word directly.
    /// - For Map slots: returns the value at the given key, or the first entry's value if no key.
    #[wasm_bindgen(js_name = "readValue")]
    pub fn read_value(&self, slot_name: &str, key: Option<Word>) -> Option<Word> {
        let Ok(slot_name) = StorageSlotName::new(slot_name) else {
            return None;
        };

        let slot = self.0.slots().iter().find(|s| s.name() == &slot_name)?;

        match slot.content() {
            StorageSlotContent::Value(_) => self.0.get_item(&slot_name).ok().map(Into::into),
            StorageSlotContent::Map(map) => {
                if let Some(k) = key {
                    self.0.get_map_item(&slot_name, k.into()).ok().map(Into::into)
                } else {
                    map.entries().next().map(|(_, value)| value.into())
                }
            },
        }
    }

    /// Convenience: read a storage value and return the first felt as a number.
    /// Handles both Value and Map slots.
    #[wasm_bindgen(js_name = "readNumber")]
    #[allow(clippy::cast_precision_loss)]
    pub fn read_number(&self, slot_name: &str, key: Option<Word>) -> Option<f64> {
        let word = self.read_value(slot_name, key)?;
        let native_word: miden_client::Word = word.into();
        Some(native_word[0].as_int() as f64)
    }

    /// Get all key-value pairs from the map slot identified by `slot_name`.
    /// Returns `undefined` if the slot isn't a map or doesn't exist.
    /// Returns `[]` if the map exists but is empty.
    #[wasm_bindgen(js_name = "getMapEntries")]
    pub fn get_map_entries(&self, slot_name: &str) -> Option<Vec<JsStorageMapEntry>> {
        let slot = self.0.slots().iter().find(|slot| slot.name().as_str() == slot_name)?;
        let StorageSlotContent::Map(map) = slot.content() else {
            return None;
        };

        Some(JsStorageMapEntry::from_map(map, slot_name))
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
