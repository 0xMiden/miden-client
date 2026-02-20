use miden_client::asset::Asset as NativeAsset;
use miden_client::note::NoteAssets as NativeNoteAssets;
use crate::prelude::*;

use super::fungible_asset::FungibleAsset;

/// An asset container for a note.
///
/// A note must contain at least 1 asset and can contain up to 256 assets. No duplicates are
/// allowed, but the order of assets is unspecified.
///
/// All the assets in a note can be reduced to a single commitment which is computed by sequentially
/// hashing the assets. Note that the same list of assets can result in two different commitments if
/// the asset ordering is different.
#[bindings]
#[derive(Clone)]
pub struct NoteAssets(pub(crate) NativeNoteAssets);

#[bindings]
impl NoteAssets {
    #[bindings(constructor)]
    pub fn new(assets_array: Option<Vec<FungibleAsset>>) -> JsResult<NoteAssets> {
        let native_assets: Vec<NativeAsset> =
            assets_array.unwrap_or_default().into_iter().map(Into::into).collect();
        let native_note_assets = NativeNoteAssets::new(native_assets)
            .map_err(|e| platform::error_with_context(e, "creating NoteAssets"))?;
        Ok(NoteAssets(native_note_assets))
    }

    pub fn fungible_assets(&self) -> Vec<FungibleAsset> {
        self.0
            .iter()
            .filter_map(|asset| {
                if asset.is_fungible() {
                    Some(asset.unwrap_fungible().into())
                } else {
                    None
                }
            })
            .collect()
    }
}

// Platform-specific methods that differ between wasm and napi
#[cfg(feature = "wasm")]
impl NoteAssets {
    /// Adds a fungible asset to the collection.
    pub fn push(&mut self, asset: &FungibleAsset) {
        self.0.add_asset(asset.into()).unwrap();
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl NoteAssets {
    /// Adds a fungible asset to the collection.
    #[napi]
    pub fn push(&mut self, asset: &FungibleAsset) -> napi::Result<()> {
        self.0
            .add_asset(asset.into())
            .map_err(|e| platform::error_with_context(e, "adding asset"))?;
        Ok(())
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteAssets> for NoteAssets {
    fn from(native_note_assets: NativeNoteAssets) -> Self {
        NoteAssets(native_note_assets)
    }
}

impl From<&NativeNoteAssets> for NoteAssets {
    fn from(native_note_assets: &NativeNoteAssets) -> Self {
        NoteAssets(native_note_assets.clone())
    }
}

impl From<NoteAssets> for NativeNoteAssets {
    fn from(note_assets: NoteAssets) -> Self {
        note_assets.0
    }
}

impl From<&NoteAssets> for NativeNoteAssets {
    fn from(note_assets: &NoteAssets) -> Self {
        note_assets.0.clone()
    }
}
