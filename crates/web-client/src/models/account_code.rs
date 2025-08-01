use miden_objects::account::AccountCode as NativeAccountCode;
use wasm_bindgen::prelude::*;

use super::word::Word;

#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountCode(NativeAccountCode);

#[wasm_bindgen]
impl AccountCode {
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountCode> for AccountCode {
    fn from(native_account_code: NativeAccountCode) -> Self {
        AccountCode(native_account_code)
    }
}

impl From<&NativeAccountCode> for AccountCode {
    fn from(native_account_code: &NativeAccountCode) -> Self {
        AccountCode(native_account_code.clone())
    }
}
