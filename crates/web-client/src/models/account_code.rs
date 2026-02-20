use miden_client::account::AccountCode as NativeAccountCode;

use crate::prelude::*;
use super::word::Word;

/// A public interface of an account.
///
/// Account's public interface consists of a set of callable procedures, each committed to by its
/// root hash and paired with storage bounds (offset and size).
///
/// The full interface commitment hashes every procedure root together with its storage bounds so
/// that the account code uniquely captures the set of available calls.
#[derive(Clone)]
#[bindings]
pub struct AccountCode(NativeAccountCode);

// Methods with identical signatures
#[bindings]
impl AccountCode {
    /// Returns the code commitment for the account.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }
}

// wasm: has_procedure takes owned Word
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AccountCode {
    /// Returns true if the account code exports a procedure with the given MAST root.
    pub fn has_procedure(&self, mast_root: Word) -> bool {
        self.0.has_procedure(mast_root.into())
    }
}

// napi: has_procedure takes &Word
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl AccountCode {
    /// Returns true if the account code exports a procedure with the given MAST root.
    #[napi]
    pub fn has_procedure(&self, mast_root: &Word) -> bool {
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
