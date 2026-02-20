#[cfg(feature = "wasm")]
use miden_client::account::{
    AccountStorage as NativeAccountStorage,
    StorageSlotContent,
    StorageSlotName,
};
use crate::prelude::*;

use crate::models::word::Word;

/// Represents a key-value entry in a storage map (wasm only).
#[cfg(feature = "wasm")]
#[wasm_bindgen]
#[derive(Clone)]
pub struct JsStorageMapEntry {
    pub(crate) root: String,
    pub(crate) key: String,
    pub(crate) value: String,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl JsStorageMapEntry {
    #[wasm_bindgen(getter)]
    pub fn root(&self) -> String { self.root.clone() }
    #[wasm_bindgen(getter)]
    pub fn key(&self) -> String { self.key.clone() }
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> String { self.value.clone() }
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
#[bindings]
pub struct AccountStorage(NativeAccountStorage);

/// Represents a key-value entry in a storage map (napi only).
#[cfg(feature = "napi")]
#[napi_derive::napi(object)]
pub struct StorageMapEntry {
    pub root: String,
    pub key: String,
    pub value: String,
}

// Methods with identical signatures
#[bindings]
impl AccountStorage {
    /// Returns the commitment to the full account storage.
    pub fn commitment(&self) -> Word {
        self.0.to_commitment().into()
    }

    /// Returns the names of all storage slots on this account.
    pub fn get_slot_names(&self) -> Vec<String> {
        self.0.slots().iter().map(|slot| slot.name().as_str().to_string()).collect()
    }
}

// wasm: &str params, JsStorageMapEntry
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AccountStorage {
    /// Returns the value stored at the given slot name, if any.
    
    pub fn get_item(&self, slot_name: &str) -> Option<Word> {
        let slot_name = StorageSlotName::new(slot_name).ok()?;
        self.0.get_item(&slot_name).ok().map(Into::into)
    }

    /// Returns the value for a key in the map stored at the given slot, if any.
    
    pub fn get_map_item(&self, slot_name: &str, key: &Word) -> Option<Word> {
        match StorageSlotName::new(slot_name) {
            Ok(slot_name) => self.0.get_map_item(&slot_name, key.into()).ok().map(Into::into),
            Err(_) => None,
        }
    }

    /// Get all key-value pairs from the map slot identified by `slot_name`.
    /// Returns `undefined` if the slot isn't a map or doesn't exist.
    /// Returns `[]` if the map exists but is empty.
    
    pub fn get_map_entries(&self, slot_name: &str) -> Option<Vec<JsStorageMapEntry>> {
        let slot = self.0.slots().iter().find(|slot| slot.name().as_str() == slot_name)?;
        let StorageSlotContent::Map(map) = slot.content() else {
            return None;
        };

        Some(
            map.entries()
                .map(|(key, value)| JsStorageMapEntry {
                    root: map.root().to_hex(),
                    key: key.to_hex(),
                    value: value.to_hex(),
                })
                .collect(),
        )
    }
}

// napi: String params, local StorageMapEntry
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl AccountStorage {
    /// Returns the value stored at the given slot name, if any.
    pub fn get_item(&self, slot_name: String) -> Option<Word> {
        let slot_name = StorageSlotName::new(slot_name).ok()?;
        self.0.get_item(&slot_name).ok().map(Into::into)
    }

    /// Returns the value for a key in the map stored at the given slot, if any.
    pub fn get_map_item(&self, slot_name: String, key: &Word) -> Option<Word> {
        match StorageSlotName::new(slot_name) {
            Ok(slot_name) => self.0.get_map_item(&slot_name, key.into()).ok().map(Into::into),
            Err(_) => None,
        }
    }

    /// Get all key-value pairs from the map slot identified by `slot_name`.
    /// Returns `undefined` if the slot isn't a map or doesn't exist.
    /// Returns `[]` if the map exists but is empty.
    pub fn get_map_entries(&self, slot_name: String) -> Option<Vec<StorageMapEntry>> {
        let slot = self.0.slots().iter().find(|slot| slot.name().as_str() == slot_name)?;
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
