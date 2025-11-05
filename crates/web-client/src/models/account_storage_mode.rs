use core::str::FromStr;

use miden_client::account::AccountStorageMode as NativeAccountStorageMode;
use wasm_bindgen::prelude::*;

/// Storage mode configuration for an account (private, public, or network).
#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountStorageMode(NativeAccountStorageMode);

#[wasm_bindgen]
impl AccountStorageMode {
    /// Returns the private storage mode, where data stays local to the client.
    pub fn private() -> AccountStorageMode {
        AccountStorageMode(NativeAccountStorageMode::Private)
    }

    /// Returns the public storage mode, where data is fully public.
    pub fn public() -> AccountStorageMode {
        AccountStorageMode(NativeAccountStorageMode::Public)
    }

    /// Returns the network storage mode, where storage is managed by the network.
    pub fn network() -> AccountStorageMode {
        AccountStorageMode(NativeAccountStorageMode::Network)
    }

    #[wasm_bindgen(js_name = "tryFromStr")]
    /// Parses a storage mode from its string representation.
    ///
    /// @throws Throws if the provided string does not match a known mode.
    pub fn try_from_str(s: &str) -> Result<AccountStorageMode, JsValue> {
        let mode = NativeAccountStorageMode::from_str(s)
            .map_err(|e| JsValue::from_str(&format!("Invalid AccountStorageMode string: {e:?}")))?;
        Ok(AccountStorageMode(mode))
    }

    #[wasm_bindgen(js_name = "asStr")]
    /// Returns the string representation of the storage mode.
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
    pub fn is_public(&self) -> bool {
        self.0 == NativeAccountStorageMode::Public
    }
}
