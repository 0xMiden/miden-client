use miden_client::account::StorageSlot as NativeStorageSlot;
use wasm_bindgen::prelude::*;

use crate::models::storage_map::StorageMap;
use crate::models::word::Word;

/// A single storage slot value or map for an account component.
#[wasm_bindgen]
#[derive(Clone)]
pub struct StorageSlot(NativeStorageSlot);

#[wasm_bindgen]
impl StorageSlot {
    /// Creates a storage slot holding a single value.
    #[wasm_bindgen(js_name = "fromValue")]
    pub fn from_value(value: &Word) -> StorageSlot {
        NativeStorageSlot::Value(value.into()).into()
    }

    /// Returns an empty value slot (zeroed).
    #[wasm_bindgen(js_name = "emptyValue")]
    pub fn empty_value() -> StorageSlot {
        NativeStorageSlot::empty_value().into()
    }

    /// Creates a storage slot backed by a map.
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
