use alloc::format;
use alloc::string::String;

use miden_client::store::StoreError;
use wasm_bindgen_futures::JsFuture;

use super::WebStore;

mod js_bindings;
use js_bindings::idxdb_export_store;

impl WebStore {
    pub async fn export_store(&self) -> Result<String, StoreError> {
        let promise = idxdb_export_store(self.db_id());
        let js_value = JsFuture::from(promise)
            .await
            .map_err(|err| StoreError::DatabaseError(format!("Failed to export store: {err:?}")))?;
        js_value
            .as_string()
            .ok_or_else(|| StoreError::DatabaseError("Export did not return a string".into()))
    }
}
