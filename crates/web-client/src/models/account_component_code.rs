use miden_client::account::component::AccountComponentCode as NativeAccountComponentCode;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct AccountComponentCode(NativeAccountComponentCode);

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountComponentCode> for AccountComponentCode {
    fn from(native_account_component: NativeAccountComponentCode) -> Self {
        AccountComponentCode(native_account_component)
    }
}

impl From<AccountComponentCode> for NativeAccountComponentCode {
    fn from(native_account_component: AccountComponentCode) -> Self {
        native_account_component.0
    }
}
