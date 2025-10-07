use miden_client::account::AccountFile as NativeAccountFile;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct AccountFile(NativeAccountFile);

#[wasm_bindgen]
impl AccountFile {
    /// Serializes the `AccountFile` into a byte array
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a byte array into an `AccountFile`
    pub fn deserialize(bytes: &Uint8Array) -> Result<AccountFile, JsValue> {
        let native_account_file: NativeAccountFile = deserialize_from_uint8array(bytes)?;
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
