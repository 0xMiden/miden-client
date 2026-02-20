use miden_client::account::AccountId as NativeAccountId;
use miden_client::asset::{
    AccountVaultDelta as NativeAccountVaultDelta,
    FungibleAssetDelta as NativeFungibleAssetDelta,
};
use crate::prelude::*;

use crate::models::account_id::AccountId;
#[cfg(feature = "wasm")]
use crate::models::fungible_asset::FungibleAsset;

/// `AccountVaultDelta` stores the difference between the initial and final account vault states.
///
/// The difference is represented as follows:
/// - `fungible`: a binary tree map of fungible asset balance changes in the account vault.
/// - `non_fungible`: a binary tree map of non-fungible assets that were added to or removed from
///   the account vault.
#[bindings]
#[derive(Clone)]
pub struct AccountVaultDelta(NativeAccountVaultDelta);

#[bindings]
impl AccountVaultDelta {
    /// Serializes the vault delta into bytes.
    pub fn serialize(&self) -> JsBytes {
        platform::serialize_to_bytes(&self.0)
    }

    /// Returns true if no assets are changed.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the fungible portion of the delta.
    pub fn fungible(&self) -> FungibleAssetDelta {
        self.0.fungible().into()
    }

    /// Deserializes a vault delta from bytes.
    #[bindings(factory)]
    pub fn deserialize(bytes: &JsBytes) -> JsResult<AccountVaultDelta> {
        platform::deserialize_from_bytes::<NativeAccountVaultDelta>(bytes).map(AccountVaultDelta)
    }
}

// WASM-only methods (added/removed assets use FungibleAsset which is wasm-specific here)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AccountVaultDelta {
    /// Returns the fungible assets that increased.
    
    pub fn added_fungible_assets(&self) -> Vec<FungibleAsset> {
        self.0
            .fungible()
            .iter()
            .filter(|&(_, &value)| value > 0)
            .map(|(faucet_id, &diff)| FungibleAsset::new(&faucet_id.into(), diff.unsigned_abs() as i64).unwrap())
            .collect()
    }

    /// Returns the fungible assets that decreased.
    
    pub fn removed_fungible_assets(&self) -> Vec<FungibleAsset> {
        self.0
            .fungible()
            .iter()
            .filter(|&(_, &value)| value < 0)
            .map(|(faucet_id, &diff)| FungibleAsset::new(&faucet_id.into(), diff.unsigned_abs() as i64).unwrap())
            .collect()
    }
}

/// A single fungible asset change in the vault delta.
#[bindings]
#[derive(Clone)]
pub struct FungibleAssetDeltaItem {
    faucet_id: AccountId,
    amount: i64,
}

#[bindings]
impl FungibleAssetDeltaItem {
    /// Returns the faucet ID this delta refers to.
    #[bindings(getter)]
    pub fn faucet_id(&self) -> AccountId {
        self.faucet_id
    }

    /// Returns the signed amount change (positive adds assets, negative removes).
    #[bindings(getter)]
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
#[bindings]
#[derive(Clone)]
pub struct FungibleAssetDelta(NativeFungibleAssetDelta);

#[bindings]
impl FungibleAssetDelta {
    /// Serializes the fungible delta into bytes.
    pub fn serialize(&self) -> JsBytes {
        platform::serialize_to_bytes(&self.0)
    }

    /// Returns true if no fungible assets are affected.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the delta amount for a given faucet, if present.
    pub fn amount(&self, faucet_id: &AccountId) -> Option<i64> {
        let native_faucet_id: NativeAccountId = faucet_id.into();
        self.0.amount(&native_faucet_id)
    }

    /// Returns the number of distinct fungible assets in the delta.
    #[bindings]
    pub fn num_assets(&self) -> i64 {
        self.0.num_assets() as i64
    }

    /// Returns all fungible asset deltas as a list.
    pub fn assets(&self) -> Vec<FungibleAssetDeltaItem> {
        self.0.iter().map(Into::into).collect()
    }

    /// Deserializes a fungible delta from bytes.
    #[bindings(factory)]
    pub fn deserialize(bytes: &JsBytes) -> JsResult<FungibleAssetDelta> {
        platform::deserialize_from_bytes::<NativeFungibleAssetDelta>(bytes).map(FungibleAssetDelta)
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
    #[test]
    fn fungible_delta_sign_classification_excludes_zero() {
        let deltas = [10_i64, 0_i64, -5_i64];

        let added: Vec<i64> = deltas.iter().copied().filter(|&v| v > 0).collect();
        let removed: Vec<i64> = deltas.iter().copied().filter(|&v| v < 0).collect();

        assert_eq!(added, vec![10]);
        assert_eq!(removed, vec![-5]);
    }
}
