use miden_client::account::AccountComponentCode as NativeAccountComponentCode;

use crate::prelude::*;
use crate::models::library::Library;

#[derive(Debug, Clone)]
#[bindings]
/// A Library that has been assembled for use as component code.
pub struct AccountComponentCode(NativeAccountComponentCode);

// wasm: as_library returns Result<Library, JsValue>
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AccountComponentCode {
    /// Returns the underlying Library
    pub fn as_library(&self) -> Result<Library, JsValue> {
        let native_library = self.0.as_library();
        Ok(native_library.into())
    }
}

// napi: as_library returns Library directly
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl AccountComponentCode {
    /// Returns the underlying Library.
    pub fn as_library(&self) -> Library {
        let native_library = self.0.as_library();
        native_library.into()
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
