use miden_client::account::AccountId as NativeAccountId;
use miden_client::asset::{
    AccountVaultDelta as NativeAccountVaultDelta,
    FungibleAssetDelta as NativeFungibleAssetDelta,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::account_id::AccountId;
use crate::models::fungible_asset::FungibleAsset;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// `AccountVaultDelta` stores the difference between the initial and final account vault states.
///
/// The difference is represented as follows:
/// - `fungible`: a binary tree map of fungible asset balance changes in the account vault.
/// - `non_fungible`: a binary tree map of non-fungible assets that were added to or removed from
///   the account vault.
#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountVaultDelta(NativeAccountVaultDelta);

#[wasm_bindgen]
impl AccountVaultDelta {
    /// Serializes the vault delta into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a vault delta from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<AccountVaultDelta, JsValue> {
        deserialize_from_uint8array::<NativeAccountVaultDelta>(bytes).map(AccountVaultDelta)
    }

    /// Returns true if no assets are changed.
    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the fungible portion of the delta.
    pub fn fungible(&self) -> FungibleAssetDelta {
        self.0.fungible().into()
    }

    /// Returns the fungible assets that increased.
    #[wasm_bindgen(js_name = "addedFungibleAssets")]
    pub fn added_fungible_assets(&self) -> Vec<FungibleAsset> {
        self.0
            .fungible()
            .iter()
            .filter(|&(_, &value)| value > 0)
            .map(|(faucet_id, &diff)| FungibleAsset::new(&faucet_id.into(), diff.unsigned_abs()))
            .collect()
    }

    /// Returns the fungible assets that decreased.
    #[wasm_bindgen(js_name = "removedFungibleAssets")]
    pub fn removed_fungible_assets(&self) -> Vec<FungibleAsset> {
        self.0
            .fungible()
            .iter()
            .filter(|&(_, &value)| value < 0)
            .map(|(faucet_id, &diff)| FungibleAsset::new(&faucet_id.into(), diff.unsigned_abs()))
            .collect()
    }
}

/// A single fungible asset change in the vault delta.
#[derive(Clone)]
#[wasm_bindgen]
pub struct FungibleAssetDeltaItem {
    faucet_id: AccountId,
    amount: i64,
}

#[wasm_bindgen]
impl FungibleAssetDeltaItem {
    /// Returns the faucet ID this delta refers to.
    #[wasm_bindgen(getter, js_name = "faucetId")]
    pub fn faucet_id(&self) -> AccountId {
        self.faucet_id
    }

    /// Returns the signed amount change (positive adds assets, negative removes).
    #[wasm_bindgen(getter)]
    pub fn amount(&self) -> i64 {
        self.amount
    }
}

impl From<(&NativeAccountId, &i64)> for FungibleAssetDeltaItem {
    fn from(native_fungible_asset_delta_item: (&NativeAccountId, &i64)) -> Self {
        Self {
            faucet_id: (*native_fungible_asset_delta_item.0).into(),
            amount: *native_fungible_asset_delta_item.1,
        }
    }
}

/// Aggregated fungible deltas keyed by faucet ID.
#[derive(Clone)]
#[wasm_bindgen]
pub struct FungibleAssetDelta(NativeFungibleAssetDelta);

#[wasm_bindgen]
impl FungibleAssetDelta {
    /// Serializes the fungible delta into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a fungible delta from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<FungibleAssetDelta, JsValue> {
        deserialize_from_uint8array::<NativeFungibleAssetDelta>(bytes).map(FungibleAssetDelta)
    }

    /// Returns true if no fungible assets are affected.
    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the delta amount for a given faucet, if present.
    pub fn amount(&self, faucet_id: &AccountId) -> Option<i64> {
        let native_faucet_id: NativeAccountId = faucet_id.into();
        self.0.amount(&native_faucet_id)
    }

    /// Returns the number of distinct fungible assets in the delta.
    #[wasm_bindgen(js_name = "numAssets")]
    pub fn num_assets(&self) -> usize {
        self.0.num_assets()
    }

    /// Returns all fungible asset deltas as a list.
    pub fn assets(&self) -> Vec<FungibleAssetDeltaItem> {
        self.0.iter().map(Into::into).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountVaultDelta> for AccountVaultDelta {
    fn from(native_account_vault_delta: NativeAccountVaultDelta) -> Self {
        Self(native_account_vault_delta)
    }
}

impl From<&NativeAccountVaultDelta> for AccountVaultDelta {
    fn from(native_account_vault_delta: &NativeAccountVaultDelta) -> Self {
        Self(native_account_vault_delta.clone())
    }
}

impl From<AccountVaultDelta> for NativeAccountVaultDelta {
    fn from(account_vault_delta: AccountVaultDelta) -> Self {
        account_vault_delta.0
    }
}

impl From<&AccountVaultDelta> for NativeAccountVaultDelta {
    fn from(account_vault_delta: &AccountVaultDelta) -> Self {
        account_vault_delta.0.clone()
    }
}

impl From<NativeFungibleAssetDelta> for FungibleAssetDelta {
    fn from(native_fungible_asset_delta: NativeFungibleAssetDelta) -> Self {
        Self(native_fungible_asset_delta)
    }
}

impl From<&NativeFungibleAssetDelta> for FungibleAssetDelta {
    fn from(native_fungible_asset_delta: &NativeFungibleAssetDelta) -> Self {
        Self(native_fungible_asset_delta.clone())
    }
}

impl From<FungibleAssetDelta> for NativeFungibleAssetDelta {
    fn from(fungible_asset_delta: FungibleAssetDelta) -> Self {
        fungible_asset_delta.0
    }
}

impl From<&FungibleAssetDelta> for NativeFungibleAssetDelta {
    fn from(fungible_asset_delta: &FungibleAssetDelta) -> Self {
        fungible_asset_delta.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fungible_delta_sign_classification_excludes_zero() {
        let deltas = [10_i64, 0_i64, -5_i64];

        let added: Vec<i64> = deltas.iter().copied().filter(|&v| v > 0).collect();
        let removed: Vec<i64> = deltas.iter().copied().filter(|&v| v < 0).collect();

        assert_eq!(added, vec![10]);
        assert_eq!(removed, vec![-5]);
    }
}
