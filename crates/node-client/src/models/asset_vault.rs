use miden_client::asset::AssetVault as NativeAssetVault;

use super::account_id::AccountId;
use super::napi_wrap;

napi_wrap!(clone AssetVault wraps NativeAssetVault);

#[napi]
impl AssetVault {
    /// Returns the balance for the given fungible faucet, or zero if absent.
    #[napi(js_name = "getBalance")]
    pub fn get_balance(&self, faucet_id: &AccountId) -> u64 {
        self.0.get_balance(faucet_id.0).unwrap()
    }
}
