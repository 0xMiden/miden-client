use miden_client::account::AccountDelta as NativeAccountDelta;
use crate::prelude::*;

use crate::models::account_id::AccountId;
use crate::models::felt::Felt;

/// `AccountDelta` stores the differences between two account states.
///
/// The differences are represented as follows:
/// - `storage`: an `AccountStorageDelta` that contains the changes to the account storage.
/// - `vault`: an `AccountVaultDelta` object that contains the changes to the account vault.
/// - `nonce`: if the nonce of the account has changed, the new nonce is stored here.
#[bindings]
#[derive(Clone)]
pub struct AccountDelta(NativeAccountDelta);

pub mod storage;
pub mod vault;

use storage::AccountStorageDelta;
use vault::AccountVaultDelta;

#[bindings]
impl AccountDelta {
    /// Serializes the account delta into bytes.
    pub fn serialize(&self) -> JsBytes {
        platform::serialize_to_bytes(&self.0)
    }

    /// Returns the affected account ID.
    pub fn id(&self) -> AccountId {
        self.0.id().into()
    }

    /// Returns true if there are no changes.
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
    #[bindings]
    pub fn nonce_delta(&self) -> Felt {
        self.0.nonce_delta().into()
    }

    /// Deserializes an account delta from bytes.
    #[bindings(factory)]
    pub fn deserialize(bytes: &JsBytes) -> JsResult<AccountDelta> {
        platform::deserialize_from_bytes::<NativeAccountDelta>(bytes).map(AccountDelta)
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
