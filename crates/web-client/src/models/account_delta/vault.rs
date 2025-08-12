use miden_objects::account::{
    AccountId as NativeAccountId,
    AccountVaultDelta as NativeAccountVaultDelta,
    FungibleAssetDelta as NativeFungibleAssetDelta,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::account_id::AccountId;
use crate::models::fungible_asset::FungibleAsset;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountVaultDelta(NativeAccountVaultDelta);

#[wasm_bindgen]
impl AccountVaultDelta {
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<AccountVaultDelta, JsValue> {
        deserialize_from_uint8array::<NativeAccountVaultDelta>(bytes).map(AccountVaultDelta)
    }

    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn fungible(&self) -> FungibleAssetDelta {
        self.0.fungible().into()
    }

    #[wasm_bindgen(js_name = "addedFungibleAssets")]
    pub fn added_fungible_assets(&self) -> Vec<FungibleAsset> {
        self.0
            .added_assets()
            .filter_map(|asset| {
                if asset.is_fungible() {
                    Some(asset.unwrap_fungible().into())
                } else {
                    None
                }
            })
            .collect()
    }

    #[wasm_bindgen(js_name = "removedFungibleAssets")]
    pub fn removed_fungible_assets(&self) -> Vec<FungibleAsset> {
        self.0
            .removed_assets()
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

#[derive(Clone)]
#[wasm_bindgen]
pub struct FungibleAssetDeltaItem {
    faucet_id: AccountId,
    amount: i64,
}

#[wasm_bindgen]
impl FungibleAssetDeltaItem {
    #[wasm_bindgen(getter, js_name = "faucetId")]
    pub fn faucet_id(&self) -> AccountId {
        self.faucet_id
    }

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

#[derive(Clone)]
#[wasm_bindgen]
pub struct FungibleAssetDelta(NativeFungibleAssetDelta);

#[wasm_bindgen]
impl FungibleAssetDelta {
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<FungibleAssetDelta, JsValue> {
        deserialize_from_uint8array::<NativeFungibleAssetDelta>(bytes).map(FungibleAssetDelta)
    }

    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn amount(&self, faucet_id: &AccountId) -> Option<i64> {
        let native_faucet_id: NativeAccountId = faucet_id.into();
        self.0.amount(&native_faucet_id)
    }

    #[wasm_bindgen(js_name = "numAssets")]
    pub fn num_assets(&self) -> usize {
        self.0.num_assets()
    }

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
