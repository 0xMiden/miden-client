use miden_client::asset::AccountStorageDelta as NativeAccountStorageDelta;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountStorageDelta(NativeAccountStorageDelta);

#[wasm_bindgen]
impl AccountStorageDelta {
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<AccountStorageDelta, JsValue> {
        deserialize_from_uint8array::<NativeAccountStorageDelta>(bytes)
            .map(AccountStorageDelta)
            .map_err(Into::into)
    }

    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn values(&self) -> Vec<Word> {
        self.0.values().values().copied().map(Into::into).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountStorageDelta> for AccountStorageDelta {
    fn from(native_account_storage_delta: NativeAccountStorageDelta) -> Self {
        Self(native_account_storage_delta)
    }
}

impl From<&NativeAccountStorageDelta> for AccountStorageDelta {
    fn from(native_account_storage_delta: &NativeAccountStorageDelta) -> Self {
        Self(native_account_storage_delta.clone())
    }
}

impl From<AccountStorageDelta> for NativeAccountStorageDelta {
    fn from(account_storage_delta: AccountStorageDelta) -> Self {
        account_storage_delta.0
    }
}

impl From<&AccountStorageDelta> for NativeAccountStorageDelta {
    fn from(account_storage_delta: &AccountStorageDelta) -> Self {
        account_storage_delta.0.clone()
    }
}
