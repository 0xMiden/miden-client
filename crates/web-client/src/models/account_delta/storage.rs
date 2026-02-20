use miden_client::asset::AccountStorageDelta as NativeAccountStorageDelta;
use crate::prelude::*;

use crate::models::word::Word;

/// `AccountStorageDelta` stores the differences between two states of account storage.
///
/// The delta consists of two maps:
/// - A map containing the updates to value storage slots. The keys in this map are indexes of the
///   updated storage slots and the values are the new values for these slots.
/// - A map containing updates to storage maps. The keys in this map are indexes of the updated
///   storage slots and the values are corresponding storage map delta objects.
#[bindings]
#[derive(Clone)]
pub struct AccountStorageDelta(NativeAccountStorageDelta);

#[bindings]
impl AccountStorageDelta {
    /// Serializes the storage delta into bytes.
    pub fn serialize(&self) -> JsBytes {
        platform::serialize_to_bytes(&self.0)
    }

    /// Returns true if no storage slots are changed.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the new values for modified storage slots.
    pub fn values(&self) -> Vec<Word> {
        self.0
            .values()
            .map(|(_slot_name, value)| value)
            .copied()
            .map(Into::into)
            .collect()
    }

    /// Deserializes a storage delta from bytes.
    #[bindings(factory)]
    pub fn deserialize(bytes: &JsBytes) -> JsResult<AccountStorageDelta> {
        platform::deserialize_from_bytes::<NativeAccountStorageDelta>(bytes)
            .map(AccountStorageDelta)
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
