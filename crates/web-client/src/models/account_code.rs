use miden_client::account::AccountCode as NativeAccountCode;
use wasm_bindgen::prelude::*;

use super::word::Word;

/// Code commitment and metadata for an account.
#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountCode(NativeAccountCode);

#[wasm_bindgen]
impl AccountCode {
    /// Returns the code commitment for the account.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns true if the account code exports a procedure with the given MAST root.
    #[wasm_bindgen(js_name = "hasProcedure")]
    pub fn has_procedure(&self, mast_root: Word) -> bool {
        self.0.has_procedure(mast_root.into())
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
