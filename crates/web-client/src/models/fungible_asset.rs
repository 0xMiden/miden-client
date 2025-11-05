use miden_client::Word as NativeWord;
use miden_client::account::AccountId as NativeAccountId;
use miden_client::asset::{Asset as NativeAsset, FungibleAsset as FungibleAssetNative};
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::word::Word;

/// Represents a fungible asset amount associated with a faucet account.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct FungibleAsset(FungibleAssetNative);

#[wasm_bindgen]
impl FungibleAsset {
    #[wasm_bindgen(constructor)]
    /// Creates a new fungible asset reference for the provided faucet.
    pub fn new(faucet_id: &AccountId, amount: u64) -> FungibleAsset {
        let native_faucet_id: NativeAccountId = faucet_id.into();
        let native_asset = FungibleAssetNative::new(native_faucet_id, amount).unwrap();

        FungibleAsset(native_asset)
    }

    #[wasm_bindgen(js_name = "faucetId")]
    /// Returns the faucet identifier that issued the asset.
    pub fn faucet_id(&self) -> AccountId {
        self.0.faucet_id().into()
    }

    /// Returns the quantity of fungible assets.
    pub fn amount(&self) -> u64 {
        self.0.amount()
    }

    #[wasm_bindgen(js_name = "intoWord")]
    /// Converts the asset into its raw word representation.
    pub fn into_word(&self) -> Word {
        let native_word: NativeWord = self.0.into();
        native_word.into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<FungibleAsset> for NativeAsset {
    fn from(fungible_asset: FungibleAsset) -> Self {
        fungible_asset.0.into()
    }
}

impl From<&FungibleAsset> for NativeAsset {
    fn from(fungible_asset: &FungibleAsset) -> Self {
        fungible_asset.0.into()
    }
}

impl From<FungibleAssetNative> for FungibleAsset {
    fn from(native_asset: FungibleAssetNative) -> Self {
        FungibleAsset(native_asset)
    }
}

impl From<&FungibleAssetNative> for FungibleAsset {
    fn from(native_asset: &FungibleAssetNative) -> Self {
        FungibleAsset(*native_asset)
    }
}
