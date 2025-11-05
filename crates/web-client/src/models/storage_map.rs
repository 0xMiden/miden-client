use miden_client::account::StorageMap as NativeStorageMap;
use wasm_bindgen::prelude::*;

use crate::models::word::Word;

/// Key/value storage map used within accounts.
#[wasm_bindgen]
pub struct StorageMap(NativeStorageMap);

#[wasm_bindgen]
impl StorageMap {
    #[wasm_bindgen(constructor)]
    /// Creates an empty storage map.
    pub fn new() -> StorageMap {
        StorageMap(NativeStorageMap::new())
    }

    /// Inserts or updates a key and returns the previous value (or zero if absent).
    pub fn insert(&mut self, key: &Word, value: &Word) -> Word {
        self.0.insert(key.into(), value.into()).unwrap_or_default().into()
    }
}

impl Default for StorageMap {
    fn default() -> Self {
        StorageMap::new()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeStorageMap> for StorageMap {
    fn from(native_storage_map: NativeStorageMap) -> Self {
        StorageMap(native_storage_map)
    }
}

impl From<&NativeStorageMap> for StorageMap {
    fn from(native_storage_map: &NativeStorageMap) -> Self {
        StorageMap(native_storage_map.clone())
    }
}

impl From<StorageMap> for NativeStorageMap {
    fn from(storage_map: StorageMap) -> Self {
        storage_map.0
    }
}

impl From<&StorageMap> for NativeStorageMap {
    fn from(storage_map: &StorageMap) -> Self {
        storage_map.0.clone()
    }
}
