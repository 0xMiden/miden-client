use miden_client::store::StoreError;

use super::WebStore;

mod js_bindings;
use js_bindings::idxdb_force_import_store;
use wasm_bindgen::JsValue;

use crate::promise::await_ok;

impl WebStore {
    pub async fn force_import_store(&self, store_dump: JsValue) -> Result<(), StoreError> {
        let promise = idxdb_force_import_store(store_dump);
        await_ok(promise, "Failed to import store").await?;
        Ok(())
    }
}
