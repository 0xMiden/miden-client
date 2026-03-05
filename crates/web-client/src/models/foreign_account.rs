use js_export_macro::js_export;
use miden_client::transaction::ForeignAccount as NativeForeignAccount;

use crate::js_error_with_context;
use crate::models::account_id::AccountId;
use crate::models::account_storage_requirements::AccountStorageRequirements;
use crate::platform::JsErr;

/// Description of a foreign account referenced by a transaction.
#[js_export]
#[derive(Clone)]
pub struct ForeignAccount(NativeForeignAccount);

#[js_export]
impl ForeignAccount {
    /// Creates a foreign account entry for a public account with given storage requirements.
    pub fn public(
        account_id: &AccountId,
        storage_requirements: AccountStorageRequirements,
    ) -> Result<ForeignAccount, JsErr> {
        let native_foreign_account =
            NativeForeignAccount::public(account_id.into(), storage_requirements.into())
                .map_err(|e| js_error_with_context(e, "Failed to create public foreign account"));

        Ok(ForeignAccount(native_foreign_account?))
    }

    /// Returns the required storage slots/keys for this foreign account.
    pub fn storage_slot_requirements(&self) -> AccountStorageRequirements {
        self.0.storage_slot_requirements().into()
    }

    /// Returns the ID of the foreign account.
    pub fn account_id(&self) -> AccountId {
        self.0.account_id().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<ForeignAccount> for NativeForeignAccount {
    fn from(foreign_account: ForeignAccount) -> Self {
        foreign_account.0
    }
}

impl From<&ForeignAccount> for NativeForeignAccount {
    fn from(foreign_account: &ForeignAccount) -> Self {
        foreign_account.0.clone()
    }
}

impl_napi_from_value!(ForeignAccount);
