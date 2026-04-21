use js_export_macro::js_export;
use miden_client::account::{
    StorageSlot as NativeStorageSlot,
    StorageSlotName as NativeStorageSlotName,
};

use crate::models::storage_map::StorageMap;
use crate::models::word::Word;
use crate::platform::{JsErr, from_str_err};

/// A single storage slot value or map for an account component.
#[js_export]
#[derive(Clone)]
pub struct StorageSlot(NativeStorageSlot);

#[js_export]
impl StorageSlot {
    /// Creates a storage slot holding a single value.
    #[js_export(js_name = "fromValue")]
    pub fn from_value(name: String, value: &Word) -> Result<StorageSlot, JsErr> {
        let name = NativeStorageSlotName::new(name)
            .map_err(|err| from_str_err(&format!("invalid storage slot name: {err}")))?;

        Ok(NativeStorageSlot::with_value(name, value.into()).into())
    }

    /// Returns an empty value slot (zeroed).
    #[js_export(js_name = "emptyValue")]
    pub fn empty_value(name: String) -> Result<StorageSlot, JsErr> {
        let name = NativeStorageSlotName::new(name)
            .map_err(|err| from_str_err(&format!("invalid storage slot name: {err}")))?;

        Ok(NativeStorageSlot::with_empty_value(name).into())
    }

    /// Creates a storage slot backed by a map.
    pub fn map(name: String, storage_map: &StorageMap) -> Result<StorageSlot, JsErr> {
        let name = NativeStorageSlotName::new(name)
            .map_err(|err| from_str_err(&format!("invalid storage slot name: {err}")))?;

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

impl_napi_from_value!(StorageSlot);
