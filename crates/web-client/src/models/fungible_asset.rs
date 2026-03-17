use js_export_macro::js_export;
use miden_client::Word as NativeWord;
use miden_client::account::AccountId as NativeAccountId;
use miden_client::asset::{Asset as NativeAsset, FungibleAsset as FungibleAssetNative};

use super::account_id::AccountId;
use super::word::Word;
use crate::platform::JsU64;

/// A fungible asset.
///
/// A fungible asset consists of a faucet ID of the faucet which issued the asset as well as the
/// asset amount. Asset amount is guaranteed to be 2^63 - 1 or smaller.
#[derive(Clone, Copy)]
#[js_export]
pub struct FungibleAsset(FungibleAssetNative);

#[js_export]
impl FungibleAsset {
    /// Creates a fungible asset for the given faucet and amount.
    #[js_export(constructor)]
    pub fn new(faucet_id: &AccountId, amount: JsU64) -> FungibleAsset {
        FungibleAsset::new_inner(faucet_id, amount)
    }

    /// Returns the amount of fungible units.
    pub fn amount(&self) -> JsU64 {
        self.0.amount() as JsU64
    }

    /// Returns the faucet account that minted this asset.
    #[js_export(js_name = "faucetId")]
    pub fn faucet_id(&self) -> AccountId {
        self.0.faucet_id().into()
    }

    /// Encodes this asset into the word layout used in the vault.
    #[js_export(js_name = "intoWord")]
    pub fn into_word(&self) -> Word {
        let native_word: NativeWord = self.0.into();
        native_word.into()
    }
}

impl FungibleAsset {
    /// Internal constructor that takes a native u64 amount, usable from both platforms.
    pub(crate) fn new_inner(faucet_id: &AccountId, amount: u64) -> FungibleAsset {
        let native_faucet_id: NativeAccountId = faucet_id.into();
        let native_asset = FungibleAssetNative::new(native_faucet_id, amount).unwrap();
        FungibleAsset(native_asset)
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

impl_napi_from_value!(FungibleAsset);
