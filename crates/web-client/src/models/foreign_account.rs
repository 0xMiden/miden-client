use miden_client::transaction::ForeignAccount as NativeForeignAccount;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::account_id::AccountId;
use crate::models::account_storage_requirements::AccountStorageRequirements;

#[wasm_bindgen]
pub struct ForeignAccount(NativeForeignAccount);

#[wasm_bindgen]
impl ForeignAccount {
    pub fn public(
        account_id: AccountId,
        storage_requirements: AccountStorageRequirements,
    ) -> Result<ForeignAccount, JsValue> {
        let native_foreign_account =
            NativeForeignAccount::public(account_id.into(), storage_requirements.into())
                .map_err(|e| js_error_with_context(e, "Failed to create public foreign account"));

        Ok(ForeignAccount(native_foreign_account?))
    }

    pub fn storage_slot_requirements(&self) -> AccountStorageRequirements {
        self.0.storage_slot_requirements().into()
    }

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
