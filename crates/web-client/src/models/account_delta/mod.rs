use miden_client::account::AccountDelta as NativeAccountDelta;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::account_id::AccountId;
use crate::models::felt::Felt;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// WASM wrapper around [`miden_client::account::AccountDelta`].
///
/// Captures the changes applied to an account after executing a transaction.
#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountDelta(NativeAccountDelta);

pub mod storage;
pub mod vault;

use storage::AccountStorageDelta;
use vault::AccountVaultDelta;

#[wasm_bindgen]
impl AccountDelta {
    /// Serializes the delta into raw bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes an account delta from bytes produced by [`serialize`].
    ///
    /// @throws Throws if the bytes cannot be parsed.
    pub fn deserialize(bytes: &Uint8Array) -> Result<AccountDelta, JsValue> {
        deserialize_from_uint8array::<NativeAccountDelta>(bytes).map(AccountDelta)
    }

    /// Returns the identifier of the account the delta belongs to.
    pub fn id(&self) -> AccountId {
        self.0.id().into()
    }

    #[wasm_bindgen(js_name = "isEmpty")]
    /// Returns `true` if the delta does not change vault, storage, or nonce.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the storage-specific changes contained in this delta.
    pub fn storage(&self) -> AccountStorageDelta {
        self.0.storage().into()
    }
    /// Returns the vault-specific changes contained in this delta.
    pub fn vault(&self) -> AccountVaultDelta {
        self.0.vault().into()
    }

    #[wasm_bindgen(js_name = "nonceDelta")]
    /// Returns the nonce delta applied to the account.
    pub fn nonce_delta(&self) -> Felt {
        self.0.nonce_delta().into()
    }
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

impl From<AccountDelta> for NativeAccountDelta {
    fn from(account_delta: AccountDelta) -> Self {
        account_delta.0
    }
}

impl From<&AccountDelta> for NativeAccountDelta {
    fn from(account_delta: &AccountDelta) -> Self {
        account_delta.0.clone()
    }
}
