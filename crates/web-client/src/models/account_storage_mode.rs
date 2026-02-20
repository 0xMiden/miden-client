use core::str::FromStr;

use miden_client::account::AccountStorageMode as NativeAccountStorageMode;

use crate::prelude::*;

/// Storage visibility mode for an account.
#[bindings]
#[derive(Clone)]
pub struct AccountStorageMode(NativeAccountStorageMode);

// Methods with identical signatures across wasm and napi
#[cfg(feature = "wasm")]
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
    
    pub fn try_from_str(s: &str) -> JsResult<AccountStorageMode> {
        let mode = NativeAccountStorageMode::from_str(s)
            .map_err(|err| platform::error_from_string(&format!("Invalid AccountStorageMode string: {err:?}")))?;
        Ok(AccountStorageMode(mode))
    }

    /// Returns the storage mode as a string.
    
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl AccountStorageMode {
    /// Creates a private storage mode.
    #[napi(factory)]
    pub fn private() -> AccountStorageMode {
        AccountStorageMode(NativeAccountStorageMode::Private)
    }

    /// Creates a public storage mode.
    #[napi(factory)]
    pub fn public() -> AccountStorageMode {
        AccountStorageMode(NativeAccountStorageMode::Public)
    }

    /// Creates a network storage mode.
    #[napi(factory)]
    pub fn network() -> AccountStorageMode {
        AccountStorageMode(NativeAccountStorageMode::Network)
    }

    /// Parses a storage mode from its string representation.
    #[napi(factory)]
    pub fn try_from_str(s: String) -> JsResult<AccountStorageMode> {
        let mode = NativeAccountStorageMode::from_str(&s)
            .map_err(|err| platform::error_from_string(&format!("Invalid AccountStorageMode string: {err:?}")))?;
        Ok(AccountStorageMode(mode))
    }

    /// Returns the storage mode as a string.
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
