use miden_client::account::AccountFile as NativeAccountFile;
#[cfg(feature = "napi")]
use miden_client::{Deserializable, Serializable};
#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;
use crate::prelude::*;

#[cfg(feature = "wasm")]
use wasm_bindgen_futures::js_sys::Uint8Array;
#[cfg(feature = "wasm")]
use crate::utils::{serialize_to_uint8array, deserialize_from_uint8array};

use crate::models::account::Account;
use crate::models::account_id::AccountId;
use crate::platform;
#[cfg(feature = "wasm")]

#[derive(Debug, Clone)]
#[bindings]
pub struct AccountFile(NativeAccountFile);

// Methods with identical signatures
#[bindings]
impl AccountFile {
    /// Returns the account ID.
    pub fn account_id(&self) -> AccountId {
        self.0.account.id().into()
    }

    /// Returns the account data.
    pub fn account(&self) -> Account {
        self.0.account.clone().into()
    }
}

// wasm: usize return, Uint8Array serialize/deserialize
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AccountFile {
    /// Returns the number of auth secret keys included.
    pub fn auth_secret_key_count(&self) -> usize {
        self.0.auth_secret_keys.len()
    }

    /// Serializes the `AccountFile` into a byte array
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a byte array into an `AccountFile`
    pub fn deserialize(bytes: &Uint8Array) -> platform::JsResult<AccountFile> {
        let native_account_file: NativeAccountFile = deserialize_from_uint8array(bytes)?;
        Ok(Self(native_account_file))
    }
}

// napi: i64 return, Buffer serialize/deserialize
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl AccountFile {
    /// Returns the number of auth secret keys included.
    pub fn auth_secret_key_count(&self) -> i64 {
        self.0.auth_secret_keys.len() as i64
    }

    /// Serializes the `AccountFile` into a byte buffer.
    #[napi]
    pub fn serialize(&self) -> Buffer {
        let bytes = self.0.to_bytes();
        Buffer::from(bytes)
    }

    /// Deserializes a byte buffer into an `AccountFile`.
    #[napi(factory)]
    pub fn deserialize(bytes: Buffer) -> platform::JsResult<AccountFile> {
        let native_account_file = NativeAccountFile::read_from_bytes(&bytes)
            .map_err(|e| {
                platform::error_with_context(e, "Error deserializing AccountFile")
            })?;
        Ok(Self(native_account_file))
    }
}

impl From<NativeAccountFile> for AccountFile {
    fn from(native_account_file: NativeAccountFile) -> Self {
        Self(native_account_file)
    }
}

impl From<AccountFile> for NativeAccountFile {
    fn from(account_file: AccountFile) -> Self {
        account_file.0
    }
}
