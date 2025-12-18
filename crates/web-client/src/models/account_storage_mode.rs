use core::str::FromStr;

use miden_client::account::AccountStorageMode as NativeAccountStorageMode;
use wasm_bindgen::prelude::*;

/// Storage visibility mode for an account.
#[wasm_bindgen]
#[derive(Clone)]
pub struct AccountStorageMode(NativeAccountStorageMode);

#[wasm_bindgen]
impl AccountStorageMode {
    /// Creates a private storage mode.
    pub fn private() -> AccountStorageMode {
        AccountStorageMode(NativeAccountStorageMode::Private)
    }

    /// Creates a public storage mode.
    pub fn public() -> AccountStorageMode {
        AccountStorageMode(NativeAccountStorageMode::Public)
    }

    /// Creates a network storage mode.
    pub fn network() -> AccountStorageMode {
        AccountStorageMode(NativeAccountStorageMode::Network)
    }

    /// Parses a storage mode from its string representation.
    #[wasm_bindgen(js_name = "tryFromStr")]
    pub fn try_from_str(s: &str) -> Result<AccountStorageMode, JsValue> {
        let mode = NativeAccountStorageMode::from_str(s)
            .map_err(|e| JsValue::from_str(&format!("Invalid AccountStorageMode string: {e:?}")))?;
        Ok(AccountStorageMode(mode))
    }

    /// Returns the storage mode as a string.
    #[wasm_bindgen(js_name = "asStr")]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<AccountStorageMode> for NativeAccountStorageMode {
    fn from(storage_mode: AccountStorageMode) -> Self {
        storage_mode.0
    }
}

impl From<&AccountStorageMode> for NativeAccountStorageMode {
    fn from(storage_mode: &AccountStorageMode) -> Self {
        storage_mode.0
    }
}

impl AccountStorageMode {
    /// Returns true if the storage mode is public.
    pub fn is_public(&self) -> bool {
        self.0 == NativeAccountStorageMode::Public
    }
}
