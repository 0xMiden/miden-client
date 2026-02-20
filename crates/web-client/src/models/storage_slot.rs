use miden_client::account::{
    StorageSlot as NativeStorageSlot,
    StorageSlotName as NativeStorageSlotName,
};
use crate::prelude::*;

use crate::models::storage_map::StorageMap;
use crate::models::word::Word;
use crate::platform::{self, JsResult};

/// A single storage slot value or map for an account component.
#[bindings]
#[derive(Clone)]
pub struct StorageSlot(NativeStorageSlot);

#[bindings]
impl StorageSlot {
    /// Creates a storage slot holding a single value.
    #[bindings(js_name = "fromValue", factory)]
    pub fn from_value(name: String, value: &Word) -> JsResult<StorageSlot> {
        let name = NativeStorageSlotName::new(name)
            .map_err(|err| platform::error_with_context(err, "invalid storage slot name"))?;

        Ok(NativeStorageSlot::with_value(name, value.into()).into())
    }

    /// Returns an empty value slot (zeroed).
    #[bindings(js_name = "emptyValue", factory)]
    pub fn empty_value(name: String) -> JsResult<StorageSlot> {
        let name = NativeStorageSlotName::new(name)
            .map_err(|err| platform::error_with_context(err, "invalid storage slot name"))?;

        Ok(NativeStorageSlot::with_empty_value(name).into())
    }

    /// Creates a storage slot backed by a map.
    #[bindings(factory)]
    pub fn map(name: String, storage_map: &StorageMap) -> JsResult<StorageSlot> {
        let name = NativeStorageSlotName::new(name)
            .map_err(|err| platform::error_with_context(err, "invalid storage slot name"))?;

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
