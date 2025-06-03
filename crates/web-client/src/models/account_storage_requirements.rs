use miden_client::rpc::domain::account::{AccountStorageRequirements as NativeAccountStorageRequirements, StorageMapKey as NativeStorageMapKey};
use wasm_bindgen::prelude::*;

use crate::{
    js_error_with_context,
    models::{rpo_digest::RpoDigest},
};

#[wasm_bindgen]
#[derive(Clone)]
pub struct SlotAndKeys {
    storage_slot_index: u8,
    storage_map_keys: Vec<RpoDigest>
}

#[wasm_bindgen]
impl SlotAndKeys {
    #[wasm_bindgen(constructor)]
    pub fn new(storage_slot_index: u8, storage_map_keys: Vec<RpoDigest>) -> SlotAndKeys {
        SlotAndKeys {
            storage_slot_index,
            storage_map_keys
        }
    }

    pub fn storage_slot_index(&self) -> u8 {
        self.storage_slot_index
    }

    pub fn storage_map_keys(&self) -> Vec<RpoDigest> {
        self.storage_map_keys.clone()
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct SlotAndKeysArray(Vec<SlotAndKeys>);

#[wasm_bindgen]
impl SlotAndKeysArray {
    #[wasm_bindgen(constructor)]
    pub fn new(slot_and_keys: Option<Vec<SlotAndKeys>>) -> SlotAndKeysArray {
        SlotAndKeysArray(slot_and_keys.unwrap_or_default())
    }

    pub fn push(&mut self, slot_and_keys: &SlotAndKeys) {
        self.0.push(slot_and_keys.clone());
    }
}

#[wasm_bindgen]
pub struct AccountStorageRequirements(NativeAccountStorageRequirements);

#[wasm_bindgen]
impl AccountStorageRequirements {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AccountStorageRequirements {
        AccountStorageRequirements(NativeAccountStorageRequirements::default())
    }

    #[wasm_bindgen]
    pub fn from_slots_array(
        slots_and_keys: &SlotAndKeysArray
    ) -> Result<AccountStorageRequirements, JsValue> {
        // (a) First, convert each SlotAndKeys into (u8, Vec<NativeDigest>).
        //     We collect these into a local Vec so that the Vec<NativeDigest> lives long enough.
        let mut intermediate: Vec<(u8, Vec<NativeStorageMapKey>)> = Vec::with_capacity(slots_and_keys.0.len());

        for sk in slots_and_keys.0.iter() {
            // Convert each RpoDigest → NativeDigest via `.into()`.
            let native_keys: Vec<NativeStorageMapKey> = sk
                .storage_map_keys
                .iter()
                .cloned()
                .map(|rpo| rpo.into())
                .collect();

            intermediate.push((sk.storage_slot_index, native_keys));
        }

        // (b) Now call the public `NativeAccountStorageRequirements::new(...)`.
        //     It expects an IntoIterator of `(u8, impl IntoIterator<Item=&StorageMapKey>)`.
        //     Here, our `intermediate` is `Vec<(u8, Vec<NativeDigest>)>`.  Calling `.iter()`
        //     on that Vec yields `(&u8, &Vec<NativeDigest>)`, so we map it into
        //     `(u8, Vec<NativeDigestIter>)` by copying the u8 and calling `Vec<NativeDigest>.iter()`.

        let native_req = NativeAccountStorageRequirements::new(
            intermediate
                .iter()
                .map(|(slot_index, keys_vec)| {
                    // `slot_index` is &u8; copy to u8
                    // `keys_vec.iter()` is an `Iterator<Item=&NativeDigest>`, which matches the
                    //    `impl IntoIterator<Item=&StorageMapKey>` requirement.
                    (*slot_index, keys_vec.iter())
                }),
        );

        Ok(AccountStorageRequirements(native_req))
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
