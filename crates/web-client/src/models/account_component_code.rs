use js_export_macro::js_export;
use miden_client::account::AccountComponentCode as NativeAccountComponentCode;

use crate::models::library::Library;
use crate::platform::JsErr;

#[derive(Debug, Clone)]
#[js_export]
/// A Library that has been assembled for use as component code.
pub struct AccountComponentCode(NativeAccountComponentCode);

#[js_export]
impl AccountComponentCode {
    /// Returns the underlying Library
    #[js_export(js_name = "asLibrary")]
    pub fn as_library(&self) -> Result<Library, JsErr> {
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

impl_napi_from_value!(AccountComponentCode);
