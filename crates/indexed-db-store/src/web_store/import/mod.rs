use miden_client::store::StoreError;

use super::WebStore;

mod js_bindings;
use js_bindings::idxdb_force_import_store;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

impl WebStore {
    pub async fn force_import_store(&self, store_dump: JsValue) -> Result<(), StoreError> {
        let promise = idxdb_force_import_store(store_dump);
        JsFuture::from(promise)
            .await
            .map_err(|err| StoreError::DatabaseError(format!("Failed to import store: {err:?}")))?;
        Ok(())
    }
}
