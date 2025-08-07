use miden_objects::account::AccountVaultDelta as NativeAccountVaultDelta;
use wasm_bindgen::prelude::*;

use super::fungible_asset_delta::FungibleAssetDelta;

#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountVaultDelta(NativeAccountVaultDelta);

#[wasm_bindgen]
impl AccountVaultDelta {
    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn fungible(&self) -> FungibleAssetDelta {
        self.0.fungible().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountVaultDelta> for AccountVaultDelta {
    fn from(native_account_vault_delta: NativeAccountVaultDelta) -> Self {
        AccountVaultDelta(native_account_vault_delta)
    }
}

impl From<&NativeAccountVaultDelta> for AccountVaultDelta {
    fn from(native_account_vault_delta: &NativeAccountVaultDelta) -> Self {
        AccountVaultDelta(native_account_vault_delta.clone())
    }
}
