use miden_client::asset::AssetVault as NativeAssetVault;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::fungible_asset::FungibleAsset;
use super::word::Word;

/// A container for an unlimited number of assets.
///
/// An asset vault can contain an unlimited number of assets. The assets are stored in a Sparse
/// Merkle tree as follows:
/// - For fungible assets, the index of a node is defined by the issuing faucet ID, and the value of
///   the node is the asset itself. Thus, for any fungible asset there will be only one node in the
///   tree.
/// - For non-fungible assets, the index is defined by the asset itself, and the asset is also the
///   value of the node.
///
/// An asset vault can be reduced to a single hash which is the root of the Sparse Merkle Tree.
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
