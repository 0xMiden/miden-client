use miden_objects::account::AccountStorageDelta as NativeAccountStorageDelta;
use wasm_bindgen::prelude::*;

use super::word::Word;

#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountStorageDelta(NativeAccountStorageDelta);

#[wasm_bindgen]
impl AccountStorageDelta {
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
        AccountStorageDelta(native_account_storage_delta)
    }
}

impl From<&NativeAccountStorageDelta> for AccountStorageDelta {
    fn from(native_account_storage_delta: &NativeAccountStorageDelta) -> Self {
        AccountStorageDelta(native_account_storage_delta.clone())
    }
}
