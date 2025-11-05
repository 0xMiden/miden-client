use miden_client::account::StorageSlot as NativeStorageSlot;
use wasm_bindgen::prelude::*;

use crate::models::storage_map::StorageMap;
use crate::models::word::Word;

/// Represents a single storage slot within an account (value or map).
#[wasm_bindgen]
pub struct StorageSlot(NativeStorageSlot);

#[wasm_bindgen]
impl StorageSlot {
    #[wasm_bindgen(js_name = "fromValue")]
    /// Creates a slot holding a value word.
    pub fn from_value(value: &Word) -> StorageSlot {
        NativeStorageSlot::Value(value.into()).into()
    }

    #[wasm_bindgen(js_name = "emptyValue")]
    /// Returns an empty value slot.
    pub fn empty_value() -> StorageSlot {
        NativeStorageSlot::empty_value().into()
    }

    /// Creates a slot containing a storage map.
    pub fn map(storage_map: &StorageMap) -> StorageSlot {
        NativeStorageSlot::Map(storage_map.into()).into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeStorageSlot> for StorageSlot {
    fn from(native_storage_slot: NativeStorageSlot) -> Self {
        StorageSlot(native_storage_slot)
    }
}

impl From<&NativeStorageSlot> for StorageSlot {
    fn from(native_storage_slot: &NativeStorageSlot) -> Self {
        StorageSlot(native_storage_slot.clone())
    }
}

impl From<StorageSlot> for NativeStorageSlot {
    fn from(storage_slot: StorageSlot) -> Self {
        storage_slot.0
    }
}

impl From<&StorageSlot> for NativeStorageSlot {
    fn from(storage_slot: &StorageSlot) -> Self {
        storage_slot.0.clone()
    }
}
