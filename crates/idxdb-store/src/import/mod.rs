use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use miden_client::store::StoreError;
use wasm_bindgen::JsValue;

use super::WebStore;

mod js_bindings;
use js_bindings::idxdb_force_import_store;

use crate::promise::await_ok;

impl WebStore {
    pub async fn import_store(&self, data: Vec<u8>) -> Result<(), StoreError> {
        let json_string = String::from_utf8(data).map_err(|err| {
            StoreError::DatabaseError(format!("Invalid UTF-8 in store data: {err}"))
        })?;
        let js_value = JsValue::from_str(&json_string);
        let promise = idxdb_force_import_store(self.db_id(), js_value);
        await_ok(promise, "Failed to import store").await?;
        Ok(())
    }
}
