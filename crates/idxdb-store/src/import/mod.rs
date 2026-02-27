use alloc::string::String;

use miden_client::store::StoreError;
use wasm_bindgen::JsValue;

use super::IdxdbStore;

mod js_bindings;
use js_bindings::idxdb_force_import_store;

use crate::promise::await_ok;

impl IdxdbStore {
    pub async fn import_store(&self, data: String) -> Result<(), StoreError> {
        let js_value = JsValue::from_str(&data);
        let promise = idxdb_force_import_store(self.db_id(), js_value);
        await_ok(promise, "Failed to import store").await?;
        Ok(())
    }
}
