use miden_client::account::{
    StorageSlot as NativeStorageSlot, StorageSlotName as NativeStorageSlotName,
};
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
    pub fn from_value(name: &str, value: &Word) -> Result<StorageSlot, JsValue> {
        let name = NativeStorageSlotName::new(name)
            .map_err(|err| JsValue::from_str(&format!("invalid storage slot name: {err}")))?;

        Ok(NativeStorageSlot::with_value(name, value.into()).into())
    }

    /// Returns an empty value slot (zeroed).
    #[wasm_bindgen(js_name = "emptyValue")]
    pub fn empty_value(name: &str) -> Result<StorageSlot, JsValue> {
        let name = NativeStorageSlotName::new(name)
            .map_err(|err| JsValue::from_str(&format!("invalid storage slot name: {err}")))?;

        Ok(NativeStorageSlot::with_empty_value(name).into())
    }

    /// Creates a storage slot backed by a map.
    pub fn map(name: &str, storage_map: &StorageMap) -> Result<StorageSlot, JsValue> {
        let name = NativeStorageSlotName::new(name)
            .map_err(|err| JsValue::from_str(&format!("invalid storage slot name: {err}")))?;

        Ok(NativeStorageSlot::with_map(name, storage_map.into()).into())
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
