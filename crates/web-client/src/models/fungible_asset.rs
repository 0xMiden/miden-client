use miden_client::Word as NativeWord;
use miden_client::account::AccountId as NativeAccountId;
use miden_client::asset::{Asset as NativeAsset, FungibleAsset as FungibleAssetNative};
use crate::prelude::*;

use super::account_id::AccountId;
use super::word::Word;
use crate::platform::JsResult;

/// A fungible asset.
///
/// A fungible asset consists of a faucet ID of the faucet which issued the asset as well as the
/// asset amount. Asset amount is guaranteed to be 2^63 - 1 or smaller.
#[bindings]
#[derive(Clone, Copy)]
pub struct FungibleAsset(pub(crate) FungibleAssetNative);

#[bindings]
impl FungibleAsset {
    /// Creates a fungible asset for the given faucet and amount.
    #[bindings(constructor)]
    pub fn new(faucet_id: &AccountId, amount: i64) -> JsResult<FungibleAsset> {
        let native_faucet_id: NativeAccountId = faucet_id.into();
        let native_asset = FungibleAssetNative::new(native_faucet_id, amount as u64)
            .map_err(|e| crate::platform::error_with_context(e, "Error creating FungibleAsset"))?;
        Ok(FungibleAsset(native_asset))
    }

    /// Returns the faucet account that minted this asset.
    pub fn faucet_id(&self) -> AccountId {
        self.0.faucet_id().into()
    }

    /// Encodes this asset into the word layout used in the vault.
    pub fn into_word(&self) -> Word {
        let native_word: NativeWord = self.0.into();
        native_word.into()
    }

    /// Returns the amount of fungible units.
    pub fn amount(&self) -> i64 {
        self.0.amount() as i64
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
