use miden_client::account::StorageMap as NativeStorageMap;
use wasm_bindgen::prelude::*;

use crate::models::word::Word;

/// An account storage map is a sparse merkle tree of depth 64.
///
/// It can be used to store a large amount of data in an account than would be otherwise possible
/// using just the account's storage slots. This works by storing the root of the map's underlying
/// SMT in one account storage slot. Each map entry is a leaf in the tree and its inclusion is
/// proven while retrieving it (e.g. via `AccountStorage::get_map_item`).
///
/// As a side-effect, this also means that _not all_ entries of the map have to be present at
/// transaction execution time in order to access or modify the map. It is sufficient if _just_ the
/// accessed/modified items are present in the advice provider.
///
/// Because the keys of the map are user-chosen and thus not necessarily uniformly distributed, the
/// tree could be imbalanced and made less efficient. To mitigate that, the keys used in the storage
/// map are hashed before they are inserted into the SMT, which creates a uniform distribution. The
/// original keys are retained in a separate map. This causes redundancy but allows for
/// introspection of the map, e.g. by querying the set of stored (original) keys which is useful in
/// debugging and explorer scenarios.
#[wasm_bindgen]
#[derive(Clone)]
pub struct StorageMap(NativeStorageMap);

#[wasm_bindgen]
impl StorageMap {
    /// Creates an empty storage map.
    #[wasm_bindgen(constructor)]
    pub fn new() -> StorageMap {
        StorageMap(NativeStorageMap::new())
    }

    /// Inserts a key/value pair, returning any previous value.
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
