use js_export_macro::js_export;
use miden_client::account::AccountCode as NativeAccountCode;

use super::word::Word;

/// A public interface of an account.
///
/// Account's public interface consists of a set of callable procedures, each committed to by its
/// root hash and paired with storage bounds (offset and size).
///
/// The full interface commitment hashes every procedure root together with its storage bounds so
/// that the account code uniquely captures the set of available calls.
#[derive(Clone)]
#[js_export]
pub struct AccountCode(NativeAccountCode);

#[js_export]
impl AccountCode {
    /// Returns the code commitment for the account.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns true if the account code exports a procedure with the given MAST root.
    #[js_export(js_name = "hasProcedure")]
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
