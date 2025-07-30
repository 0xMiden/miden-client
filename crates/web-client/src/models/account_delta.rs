use miden_objects::account::AccountDelta as NativeAccountDelta;
use wasm_bindgen::prelude::*;

use super::{
    account_storage_delta::AccountStorageDelta, account_vault_delta::AccountVaultDelta, felt::Felt,
};

#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountDelta(NativeAccountDelta);

#[wasm_bindgen]
impl AccountDelta {
    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn storage(&self) -> AccountStorageDelta {
        self.0.storage().into()
    }

    pub fn vault(&self) -> AccountVaultDelta {
        self.0.vault().into()
    }

    #[wasm_bindgen(js_name = "nonceDelta")]
    pub fn nonce_delta(&self) -> Felt {
        self.0.nonce_delta().into()
    }

    // TODO: into parts
    // pub fn into_parts(self) -> (AccountStorageDelta, AccountVaultDelta, Option<Felt>) {
    //     let (storage, vault, nonce) = self.0.into_parts();
    //     (storage.into(), vault.into(), nonce.map(|nonce| nonce.into()))
    // }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountDelta> for AccountDelta {
    fn from(native_account_delta: NativeAccountDelta) -> Self {
        AccountDelta(native_account_delta)
    }
}

impl From<&NativeAccountDelta> for AccountDelta {
    fn from(native_account_delta: &NativeAccountDelta) -> Self {
        AccountDelta(native_account_delta.clone())
    }
}
