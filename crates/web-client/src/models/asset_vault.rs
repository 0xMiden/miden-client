use miden_client::asset::AssetVault as NativeAssetVault;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::fungible_asset::FungibleAsset;
use super::word::Word;

/// Sparse Merkle tree of assets held by an account.
#[derive(Clone)]
#[wasm_bindgen]
pub struct AssetVault(NativeAssetVault);

#[wasm_bindgen]
impl AssetVault {
    /// Returns the root commitment of the asset vault tree.
    pub fn root(&self) -> Word {
        self.0.root().into()
    }

    /// Returns the balance for the given fungible faucet, or zero if absent.
    #[wasm_bindgen(js_name = "getBalance")]
    pub fn get_balance(&self, faucet_id: &AccountId) -> u64 {
        self.0.get_balance(faucet_id.into()).unwrap()
    }

    /// Returns the fungible assets contained in this vault.
    #[wasm_bindgen(js_name = "fungibleAssets")]
    pub fn fungible_assets(&self) -> Vec<FungibleAsset> {
        self.0
            .assets()
            .filter_map(|asset| {
                if asset.is_fungible() {
                    Some(asset.unwrap_fungible().into())
                } else {
                    None // TODO: Support non fungible assets
                }
            })
            .collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAssetVault> for AssetVault {
    fn from(native_asset_vault: NativeAssetVault) -> Self {
        AssetVault(native_asset_vault)
    }
}

impl From<&NativeAssetVault> for AssetVault {
    fn from(native_asset_vault: &NativeAssetVault) -> Self {
        AssetVault(native_asset_vault.clone())
    }
}
