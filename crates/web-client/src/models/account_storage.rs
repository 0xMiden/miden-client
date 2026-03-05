use miden_client::account::{
    AccountStorage as NativeAccountStorage,
    StorageSlotContent,
    StorageSlotName,
};
use js_export_macro::js_export;
#[cfg(feature = "browser")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "nodejs")]
use napi_derive::napi;

use crate::models::word::Word;

#[cfg_attr(feature = "browser", wasm_bindgen(getter_with_clone, inspectable))]
#[cfg_attr(feature = "nodejs", napi(object))]
#[derive(Clone)]
pub struct StorageMapEntry {
    #[cfg_attr(feature = "browser", wasm_bindgen(js_name = "root"))]
    pub root: String,
    #[cfg_attr(feature = "browser", wasm_bindgen(js_name = "key"))]
    pub key: String,
    #[cfg_attr(feature = "browser", wasm_bindgen(js_name = "value"))]
    pub value: String,
}

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
#[js_export]
pub struct AccountStorage(NativeAccountStorage);

#[js_export]
impl AccountStorage {
    /// Returns the commitment to the full account storage.
    pub fn commitment(&self) -> Word {
        self.0.to_commitment().into()
    }

    /// Returns the value stored at the given slot name, if any.
    #[js_export(js_name = "getItem")]
    pub fn get_item(&self, slot_name: String) -> Option<Word> {
        let slot_name = StorageSlotName::new(slot_name).ok()?;
        self.0.get_item(&slot_name).ok().map(Into::into)
    }

    /// Returns the names of all storage slots on this account.
    #[js_export(js_name = "getSlotNames")]
    pub fn get_slot_names(&self) -> Vec<String> {
        self.0.slots().iter().map(|slot| slot.name().as_str().to_string()).collect()
    }

    /// Returns the value for a key in the map stored at the given slot, if any.
    #[js_export(js_name = "getMapItem")]
    pub fn get_map_item(&self, slot_name: String, key: &Word) -> Option<Word> {
        match StorageSlotName::new(slot_name) {
            Ok(slot_name) => self.0.get_map_item(&slot_name, key.into()).ok().map(Into::into),
            Err(_) => None,
        }
    }

    /// Get all key-value pairs from the map slot identified by `slot_name`.
    /// Returns `undefined` if the slot isn't a map or doesn't exist.
    /// Returns `[]` if the map exists but is empty.
    #[js_export(js_name = "getMapEntries")]
    pub fn get_map_entries(&self, slot_name: String) -> Option<Vec<StorageMapEntry>> {
        let slot = self.0.slots().iter().find(|slot| slot.name().as_str() == slot_name.as_str())?;
        let StorageSlotContent::Map(map) = slot.content() else {
            return None;
        };

        Some(
            map.entries()
                .map(|(key, value)| StorageMapEntry {
                    root: map.root().to_hex(),
                    key: key.to_hex(),
                    value: value.to_hex(),
                })
                .collect(),
        )
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
