use miden_client::store::StoreError;
use wasm_bindgen_futures::JsFuture;

mod js_bindings;

impl super::WebStore {
    // TRANSPORT
    // --------------------------------------------------------------------------------------------

    /// Gets the transport layer cursor from IndexedDB.
    pub async fn get_transport_layer_cursor(&self) -> Result<u64, StoreError> {
        use js_bindings::idxdb_get_transport_layer_cursor;

        let promise = idxdb_get_transport_layer_cursor();
        let js_value = JsFuture::from(promise).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to get transport layer cursor: {js_error:?}"))
        })?;

        // Convert JS value to u64
        let cursor = js_value.as_f64()
            .ok_or_else(|| StoreError::DatabaseError("invalid cursor value".to_string()))?
            as u64;

        Ok(cursor)
    }

    /// Updates the transport layer cursor in IndexedDB.
    pub async fn update_transport_layer_cursor(&self, cursor: u64) -> Result<(), StoreError> {
        use js_bindings::idxdb_update_transport_layer_cursor;

        let promise = idxdb_update_transport_layer_cursor(cursor);
        JsFuture::from(promise).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to update transport layer cursor: {js_error:?}"))
        })?;

        Ok(())
    }
}
