use miden_client::asset::Asset as NativeAsset;
use miden_client::note::NoteAssets as NativeNoteAssets;
use wasm_bindgen::prelude::*;

use super::fungible_asset::FungibleAsset;

/// An asset container for a note.
///
/// A note must contain at least 1 asset and can contain up to 256 assets. No duplicates are
/// allowed, but the order of assets is unspecified.
///
/// All the assets in a note can be reduced to a single commitment which is computed by sequentially
/// hashing the assets. Note that the same list of assets can result in two different commitments if
/// the asset ordering is different.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteAssets(NativeNoteAssets);

#[wasm_bindgen]
impl NoteAssets {
    /// Creates a new asset list for a note.
    #[wasm_bindgen(constructor)]
    pub fn new(assets_array: Option<Vec<FungibleAsset>>) -> NoteAssets {
        let assets = assets_array.unwrap_or_default();
        let native_assets: Vec<NativeAsset> = assets.into_iter().map(Into::into).collect();
        NoteAssets(NativeNoteAssets::new(native_assets).unwrap())
    }

    /// Adds a fungible asset to the collection.
    pub fn push(&mut self, asset: &FungibleAsset) {
        self.0.add_asset(asset.into()).unwrap();
    }

    /// Returns all fungible assets contained in the note.
    #[wasm_bindgen(js_name = "fungibleAssets")]
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
