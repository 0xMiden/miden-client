use miden_client::account::AccountDelta as NativeAccountDelta;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::account_id::AccountId;
use crate::models::felt::Felt;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// [`AccountDelta`] stores the differences between two account states.
///
/// The differences are represented as follows:
/// - `storage`: an [`AccountStorageDelta`] that contains the changes to the account storage.
/// - `vault`: an [`AccountVaultDelta`] object that contains the changes to the account vault.
/// - `nonce`: if the nonce of the account has changed, the new nonce is stored here.
///
/// TODO: add ability to trace account code updates.
#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountDelta(NativeAccountDelta);

pub mod storage;
pub mod vault;

use storage::AccountStorageDelta;
use vault::AccountVaultDelta;

#[wasm_bindgen]
impl AccountDelta {
    /// Serializes the account delta into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes an account delta from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<AccountDelta, JsValue> {
        deserialize_from_uint8array::<NativeAccountDelta>(bytes).map(AccountDelta)
    }

    /// Returns the affected account ID.
    pub fn id(&self) -> AccountId {
        self.0.id().into()
    }

    /// Returns true if there are no changes.
    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the storage delta.
    pub fn storage(&self) -> AccountStorageDelta {
        self.0.storage().into()
    }
    /// Returns the vault delta.
    pub fn vault(&self) -> AccountVaultDelta {
        self.0.vault().into()
    }

    /// Returns the nonce change.
    #[wasm_bindgen(js_name = "nonceDelta")]
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
