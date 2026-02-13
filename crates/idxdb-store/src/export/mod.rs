use alloc::format;
use alloc::vec::Vec;

use miden_client::store::StoreError;
use wasm_bindgen_futures::JsFuture;

use super::WebStore;

mod js_bindings;
use js_bindings::idxdb_export_store;

impl WebStore {
    pub async fn export_store(&self) -> Result<Vec<u8>, StoreError> {
        let promise = idxdb_export_store(self.db_id());
        let js_value = JsFuture::from(promise)
            .await
            .map_err(|err| StoreError::DatabaseError(format!("Failed to export store: {err:?}")))?;
        let json_string = js_value
            .as_string()
            .ok_or_else(|| StoreError::DatabaseError("Export did not return a string".into()))?;
        Ok(json_string.into_bytes())
    }
}
