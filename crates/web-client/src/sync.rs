use miden_client::asset::{Asset as NativeAsset, FungibleAsset as NativeFungibleAsset};
use miden_client::note::build_swap_tag as native_build_swap_tag;
use miden_client::sync::SyncSummary as NativeSyncSummary;
use miden_client::utils::{Deserializable, Serializable};
use wasm_bindgen::prelude::*;

use crate::models::account_id::AccountId;
use crate::models::sync_summary::SyncSummary;
use crate::models::{NoteTag, NoteType};
use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    /// Syncs the client state with the node.
    ///
    /// This method coordinates concurrent sync calls using the Web Locks API when available,
    /// with an in-process mutex fallback for older browsers. If a sync is already in progress,
    /// subsequent callers will wait and receive the same result (coalescing behavior).
    #[wasm_bindgen(js_name = "syncState")]
    pub async fn sync_state(&mut self) -> Result<SyncSummary, JsValue> {
        self.sync_state_with_timeout(0).await
    }

    /// Syncs the client state with the node with an optional timeout.
    ///
    /// This method coordinates concurrent sync calls using the Web Locks API when available,
    /// with an in-process mutex fallback for older browsers. If a sync is already in progress,
    /// subsequent callers will wait and receive the same result (coalescing behavior).
    ///
    /// # Arguments
    /// * `timeout_ms` - Timeout in milliseconds (0 = no timeout)
    #[wasm_bindgen(js_name = "syncStateWithTimeout")]
    pub async fn sync_state_with_timeout(
        &mut self,
        timeout_ms: u32,
    ) -> Result<SyncSummary, JsValue> {
        // Clone the store Arc to avoid borrow conflicts with self
        let store = self.store.clone().ok_or(JsValue::from_str("Store not initialized"))?;

        // Acquire the sync lock
        let lock_handle = store
            .acquire_sync_lock(timeout_ms)
            .await
            .map_err(|err| js_error_with_context(err, "failed to acquire sync lock"))?;

        if lock_handle.acquired {
            // We acquired the lock - perform the sync
            let client = self.get_mut_inner().ok_or(JsValue::from_str("Client not initialized"))?;

            match client.sync_state().await {
                Ok(sync_summary) => {
                    // Release the lock with the serialized result
                    let serialized = sync_summary.to_bytes();
                    store.release_sync_lock(serialized);
                    Ok(sync_summary.into())
                },
                Err(err) => {
                    // Release the lock with error, passing the error message to waiters
                    let error_message = format!("failed to sync state: {err}");
                    store.release_sync_lock_with_error(Some(error_message.clone()));
                    Err(js_error_with_context(err, "failed to sync state"))
                },
            }
        } else {
            // We're coalescing - use the result from the in-progress sync
            let coalesced_bytes = lock_handle
                .coalesced_result
                .ok_or_else(|| JsValue::from_str("Coalesced sync lock handle missing result"))?;

            let sync_summary =
                NativeSyncSummary::read_from_bytes(&coalesced_bytes).map_err(|err| {
                    JsValue::from_str(&format!(
                        "Failed to deserialize coalesced sync result: {err}"
                    ))
                })?;

            Ok(sync_summary.into())
        }
    }

    #[wasm_bindgen(js_name = "getSyncHeight")]
    pub async fn get_sync_height(&mut self) -> Result<u32, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let sync_height = client
                .get_sync_height()
                .await
                .map_err(|err| js_error_with_context(err, "failed to get sync height"))?;

            Ok(sync_height.as_u32())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "buildSwapTag")]
    pub fn build_swap_tag(
        note_type: NoteType,
        offered_asset_faucet_id: &AccountId,
        offered_asset_amount: u64,
        requested_asset_faucet_id: &AccountId,
        requested_asset_amount: u64,
    ) -> Result<NoteTag, JsValue> {
        let offered_fungible_asset: NativeAsset =
            NativeFungibleAsset::new(offered_asset_faucet_id.into(), offered_asset_amount)
                .map_err(|err| {
                    js_error_with_context(err, "failed to create offered fungible asset")
                })?
                .into();

        let requested_fungible_asset: NativeAsset =
            NativeFungibleAsset::new(requested_asset_faucet_id.into(), requested_asset_amount)
                .map_err(|err| {
                    js_error_with_context(err, "failed to create requested fungible asset")
                })?
                .into();

        let native_note_tag = native_build_swap_tag(
            note_type.into(),
            &offered_fungible_asset,
            &requested_fungible_asset,
        )
        .map_err(|err| js_error_with_context(err, "failed to build swap tag"))?;

        Ok(native_note_tag.into())
    }
}
