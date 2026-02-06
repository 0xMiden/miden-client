use miden_client::store::AccountStatus as NativeAccountStatus;
use wasm_bindgen::prelude::*;

use super::word::Word;

/// Represents the status of an account tracked by the client.
///
/// The status of an account may change by local or external factors.
#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountStatus(NativeAccountStatus);

#[wasm_bindgen]
impl AccountStatus {
    /// Returns `true` if the account is new and hasn't been used yet.
    #[wasm_bindgen(js_name = "isNew")]
    pub fn is_new(&self) -> bool {
        self.0.is_new()
    }

    /// Returns `true` if the account is locked.
    ///
    /// A locked account has a local state that doesn't match the node's state,
    /// rendering it unusable for transactions.
    #[wasm_bindgen(js_name = "isLocked")]
    pub fn is_locked(&self) -> bool {
        self.0.is_locked()
    }

    /// Returns the account seed if available.
    ///
    /// The seed is available for:
    /// - New accounts (stored in the New status)
    /// - Locked private accounts with nonce=0 (preserved for reconstruction)
    pub fn seed(&self) -> Option<Word> {
        self.0.seed().map(Into::into)
    }

    /// Returns the status as a string representation.
    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js(&self) -> String {
        self.0.to_string()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountStatus> for AccountStatus {
    fn from(native: NativeAccountStatus) -> Self {
        AccountStatus(native)
    }
}

impl From<&NativeAccountStatus> for AccountStatus {
    fn from(native: &NativeAccountStatus) -> Self {
        AccountStatus(native.clone())
    }
}
