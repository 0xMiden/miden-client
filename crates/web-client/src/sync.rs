use miden_client::asset::{Asset as NativeAsset, FungibleAsset as NativeFungibleAsset};
use miden_client::note::build_swap_tag as native_build_swap_tag;

use crate::prelude::*;
use crate::WebClient;
use crate::models::account_id::AccountId;
use crate::models::note_tag::NoteTag;
use crate::models::note_type::NoteType;
use crate::models::sync_summary::SyncSummary;

#[bindings]
impl WebClient {
    /// Performs the sync operation to bring the client state up to date with the network.
    ///
    /// On wasm, concurrent call coordination is handled at the JavaScript layer using the Web
    /// Locks API. Do not call this method directly from JS — use `syncState()` instead.
    #[bindings(js_name = "syncStateImpl")]
    pub async fn sync_state(&self) -> platform::JsResult<SyncSummary> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        // On napi, we need assert_send because StateSync contains Arc<dyn OnNoteReceived>
        // which is not Send+Sync. The MutexGuard ensures exclusive access.
        #[cfg(feature = "wasm")]
        let sync_summary = client.sync_state().await;
        #[cfg(feature = "napi")]
        let sync_summary = unsafe { crate::assert_send(client.sync_state()) }.await;

        let sync_summary =
            sync_summary.map_err(|err| platform::error_with_context(err, "failed to sync state"))?;

        Ok(sync_summary.into())
    }

    #[bindings(js_name = "getSyncHeight")]
    pub async fn get_sync_height(&self) -> platform::JsResult<u32> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        #[cfg(feature = "wasm")]
        let sync_height = client.get_sync_height().await;
        #[cfg(feature = "napi")]
        let sync_height = unsafe { crate::assert_send(client.get_sync_height()) }.await;

        let sync_height = sync_height
            .map_err(|err| platform::error_with_context(err, "failed to get sync height"))?;

        Ok(sync_height.as_u32())
    }

    #[bindings(js_name = "buildSwapTag")]
    pub fn build_swap_tag(
        &self,
        note_type: NoteType,
        offered_asset_faucet_id: &AccountId,
        offered_asset_amount: i64,
        requested_asset_faucet_id: &AccountId,
        requested_asset_amount: i64,
    ) -> platform::JsResult<NoteTag> {
        let offered_fungible_asset: NativeAsset =
            NativeFungibleAsset::new(offered_asset_faucet_id.into(), offered_asset_amount as u64)
                .map_err(|err| {
                    platform::error_with_context(err, "failed to create offered fungible asset")
                })?
                .into();

        let requested_fungible_asset: NativeAsset = NativeFungibleAsset::new(
            requested_asset_faucet_id.into(),
            requested_asset_amount as u64,
        )
        .map_err(|err| {
            platform::error_with_context(err, "failed to create requested fungible asset")
        })?
        .into();

        let native_note_tag = native_build_swap_tag(
            note_type.into(),
            &offered_fungible_asset,
            &requested_fungible_asset,
        );

        Ok(native_note_tag.into())
    }
}
