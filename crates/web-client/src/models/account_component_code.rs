use miden_client::account::component::AccountComponentCode as NativeAccountComponentCode;
use wasm_bindgen::prelude::*;

use crate::models::library::Library;

#[derive(Debug, Clone)]
#[wasm_bindgen]
/// A Library that has been assembled for use as component code.
pub struct AccountComponentCode(NativeAccountComponentCode);

#[wasm_bindgen]
impl AccountComponentCode {
    /// Returns the underlying Library
    #[wasm_bindgen(js_name = "asLibrary")]
    pub fn as_library(&self) -> Result<Library, JsValue> {
        let native_library = self.0.as_library();
        Ok(native_library.into())
    }
}

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
