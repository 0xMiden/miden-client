use miden_client::account::StorageMap as NativeStorageMap;
use wasm_bindgen::prelude::*;

use crate::models::word::Word;

#[wasm_bindgen]
pub struct StorageMap(NativeStorageMap);

#[wasm_bindgen]
impl StorageMap {
    #[wasm_bindgen(constructor)]
    pub fn new() -> StorageMap {
        StorageMap(NativeStorageMap::new())
    }

    pub fn insert(&mut self, key: &Word, value: &Word) -> Word {
        self.0.insert(key.into(), value.into()).into()
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
