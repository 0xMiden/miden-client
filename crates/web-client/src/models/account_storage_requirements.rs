use miden_client::rpc::domain::account::{
    AccountStorageRequirements as NativeAccountStorageRequirements,
    StorageMapKey as NativeStorageMapKey,
};
use wasm_bindgen::prelude::*;

use crate::models::word::Word;

/// Helper structure representing a storage slot index with the keys to retain.
#[wasm_bindgen]
#[derive(Clone)]
pub struct SlotAndKeys {
    storage_slot_index: u8,
    storage_map_keys: Vec<Word>,
}

#[wasm_bindgen]
impl SlotAndKeys {
    #[wasm_bindgen(constructor)]
    /// Creates a new [`SlotAndKeys`] entry.
    pub fn new(storage_slot_index: u8, storage_map_keys: Vec<Word>) -> SlotAndKeys {
        SlotAndKeys { storage_slot_index, storage_map_keys }
    }

    /// Returns the index of the storage slot.
    pub fn storage_slot_index(&self) -> u8 {
        self.storage_slot_index
    }

    /// Returns the map keys that must be available for this slot.
    pub fn storage_map_keys(&self) -> Vec<Word> {
        self.storage_map_keys.clone()
    }
}

/// Wrapper around storage requirements returned by RPC endpoints.
#[wasm_bindgen]
pub struct AccountStorageRequirements(NativeAccountStorageRequirements);

#[wasm_bindgen]
impl AccountStorageRequirements {
    #[wasm_bindgen(constructor)]
    /// Creates empty storage requirements.
    pub fn new() -> AccountStorageRequirements {
        AccountStorageRequirements(NativeAccountStorageRequirements::default())
    }

    #[wasm_bindgen(js_name = "fromSlotAndKeysArray")]
    /// Builds requirements from a list of slot/key pairs.
    pub fn from_slot_and_keys_array(
        slots_and_keys: Vec<SlotAndKeys>,
    ) -> Result<AccountStorageRequirements, JsValue> {
        let mut intermediate: Vec<(u8, Vec<NativeStorageMapKey>)> =
            Vec::with_capacity(slots_and_keys.len());

        for sk in slots_and_keys {
            let native_keys: Vec<NativeStorageMapKey> =
                sk.storage_map_keys.into_iter().map(Into::into).collect();

            intermediate.push((sk.storage_slot_index, native_keys));
        }

        let native_req = NativeAccountStorageRequirements::new(
            intermediate.iter().map(|(slot_index, keys_vec)| (*slot_index, keys_vec.iter())),
        );

        Ok(AccountStorageRequirements(native_req))
    }
}

impl Default for AccountStorageRequirements {
    fn default() -> Self {
        Self::new()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<AccountStorageRequirements> for NativeAccountStorageRequirements {
    fn from(account_storage_requirements: AccountStorageRequirements) -> Self {
        account_storage_requirements.0
    }
}

impl From<&AccountStorageRequirements> for NativeAccountStorageRequirements {
    fn from(account_storage_requirements: &AccountStorageRequirements) -> Self {
        account_storage_requirements.0.clone()
    }
}

impl From<NativeAccountStorageRequirements> for AccountStorageRequirements {
    fn from(native_account_storage_requirements: NativeAccountStorageRequirements) -> Self {
        AccountStorageRequirements(native_account_storage_requirements)
    }
}

impl From<&NativeAccountStorageRequirements> for AccountStorageRequirements {
    fn from(native_account_storage_requirements: &NativeAccountStorageRequirements) -> Self {
        AccountStorageRequirements(native_account_storage_requirements.clone())
    }
}
