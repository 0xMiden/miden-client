use alloc::string::String;

use miden_client::account::StorageSlotName as NativeStorageSlotName;
use miden_client::rpc::domain::account::{
    AccountStorageRequirements as NativeAccountStorageRequirements,
    StorageMapKey as NativeStorageMapKey,
};
use crate::prelude::*;

use crate::models::word::Word;
use crate::platform::{self, JsResult};

/// Storage slot index paired with map keys that must be present.
#[bindings]
#[derive(Clone)]
pub struct SlotAndKeys {
    storage_slot_name: String,
    storage_map_keys: Vec<Word>,
}

// wasm: constructor takes Vec<Word>
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl SlotAndKeys {
    /// Creates a new slot-and-keys entry.
    #[wasm_bindgen(constructor)]
    pub fn new(storage_slot_name: String, storage_map_keys: Vec<Word>) -> SlotAndKeys {
        SlotAndKeys { storage_slot_name, storage_map_keys }
    }
}

// napi: constructor takes Vec<&Word>
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl SlotAndKeys {
    /// Creates a new slot-and-keys entry.
    #[bindings(constructor)]
    pub fn new(storage_slot_name: String, storage_map_keys: Vec<&Word>) -> SlotAndKeys {
        SlotAndKeys {
            storage_slot_name,
            storage_map_keys: storage_map_keys.into_iter().cloned().collect(),
        }
    }
}

// Shared getters
#[bindings]
impl SlotAndKeys {
    /// Returns the slot name.
    
    pub fn storage_slot_name(&self) -> String {
        self.storage_slot_name.clone()
    }

    /// Returns the storage map keys required for this slot.
    
    pub fn storage_map_keys(&self) -> Vec<Word> {
        self.storage_map_keys.clone()
    }
}

#[derive(Clone)]
#[bindings]
pub struct AccountStorageRequirements(NativeAccountStorageRequirements);

// Shared constructor
#[bindings]
impl AccountStorageRequirements {
    /// Creates empty storage requirements.
    #[bindings(constructor)]
    pub fn new() -> AccountStorageRequirements {
        AccountStorageRequirements(NativeAccountStorageRequirements::default())
    }
}

// wasm: from_slot_and_keys_array takes Vec<SlotAndKeys>
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AccountStorageRequirements {
    /// Builds storage requirements from a list of slot/key pairs.
    
    pub fn from_slot_and_keys_array(
        slots_and_keys: Vec<SlotAndKeys>,
    ) -> JsResult<AccountStorageRequirements> {
        let mut intermediate: Vec<(NativeStorageSlotName, Vec<NativeStorageMapKey>)> =
            Vec::with_capacity(slots_and_keys.len());

        for sk in slots_and_keys {
            let slot_name = NativeStorageSlotName::new(sk.storage_slot_name)
                .map_err(|err| platform::error_with_context(err, "invalid storage slot name"))?;

            let native_keys: Vec<NativeStorageMapKey> =
                sk.storage_map_keys.into_iter().map(Into::into).collect();

            intermediate.push((slot_name, native_keys));
        }

        let native_req = NativeAccountStorageRequirements::new(
            intermediate
                .iter()
                .map(|(slot_name, keys_vec)| (slot_name.clone(), keys_vec.iter())),
        );

        Ok(AccountStorageRequirements(native_req))
    }
}

// napi: from_slot_and_keys_array takes Vec<&SlotAndKeys>, has factory attr
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl AccountStorageRequirements {
    /// Builds storage requirements from a list of slot/key pairs.
    #[bindings(factory)]
    pub fn from_slot_and_keys_array(
        slots_and_keys: Vec<&SlotAndKeys>,
    ) -> JsResult<AccountStorageRequirements> {
        let mut intermediate: Vec<(NativeStorageSlotName, Vec<NativeStorageMapKey>)> =
            Vec::with_capacity(slots_and_keys.len());

        for sk in slots_and_keys {
            let slot_name = NativeStorageSlotName::new(sk.storage_slot_name.clone())
                .map_err(|err| platform::error_with_context(err, "invalid storage slot name"))?;

            let native_keys: Vec<NativeStorageMapKey> =
                sk.storage_map_keys.iter().map(|w| w.clone().into()).collect();

            intermediate.push((slot_name, native_keys));
        }

        let native_req = NativeAccountStorageRequirements::new(
            intermediate
                .iter()
                .map(|(slot_name, keys_vec)| (slot_name.clone(), keys_vec.iter())),
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
