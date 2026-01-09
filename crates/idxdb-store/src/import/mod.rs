use miden_client::store::StoreError;

use super::WebStore;

mod js_bindings;
use js_bindings::idxdb_force_import_store;
use wasm_bindgen::JsValue;

use crate::promise::await_ok;

impl WebStore {
    pub async fn force_import_store(
        &self,
        store_dump: JsValue,
        store_name: &str,
    ) -> Result<(), StoreError> {
        let promise = idxdb_force_import_store(store_dump, crate::CLIENT_VERSION, store_name);
        await_ok(promise, "Failed to import store").await?;
        Ok(())
    }
}
