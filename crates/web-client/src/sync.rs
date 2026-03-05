use js_export_macro::js_export;
use miden_client::asset::{Asset as NativeAsset, FungibleAsset as NativeFungibleAsset};
use miden_client::note::build_swap_tag as native_build_swap_tag;

use crate::models::account_id::AccountId;
use crate::models::sync_summary::SyncSummary;
use crate::models::{NoteTag, NoteType};
use crate::platform::{JsErr, JsU64, from_str_err, maybe_wrap_send};
use crate::{WebClient, js_error_with_context};

#[js_export]
impl WebClient {
    #[js_export(js_name = "buildSwapTag")]
    pub fn build_swap_tag(
        note_type: NoteType,
        offered_asset_faucet_id: &AccountId,
        offered_asset_amount: JsU64,
        requested_asset_faucet_id: &AccountId,
        requested_asset_amount: JsU64,
    ) -> Result<NoteTag, JsErr> {
        let offered_asset_amount = offered_asset_amount as u64;
        let requested_asset_amount = requested_asset_amount as u64;

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
        );

        Ok(native_note_tag.into())
    }

    /// Internal implementation of `sync_state`.
    ///
    /// This method performs the actual sync operation. Concurrent call coordination
    /// is handled at the JavaScript layer using the Web Locks API.
    ///
    /// **Note:** Do not call this method directly. Use `syncState()` from JavaScript instead,
    /// which provides proper coordination for concurrent calls.
    #[js_export(js_name = "syncStateImpl")]
    pub async fn sync_state_impl(&self) -> Result<SyncSummary, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;

        let sync_summary = maybe_wrap_send(client.sync_state()).await
            .map_err(|err| js_error_with_context(err, "failed to sync state"))?;

        Ok(sync_summary.into())
    }

    #[js_export(js_name = "getSyncHeight")]
    pub async fn get_sync_height(&self) -> Result<u32, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let sync_height = client
            .get_sync_height()
            .await
            .map_err(|err| js_error_with_context(err, "failed to get sync height"))?;

        Ok(sync_height.as_u32())
    }
}
