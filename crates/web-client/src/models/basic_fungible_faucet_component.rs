use miden_client::account::Account as NativeAccount;
use miden_client::account::component::BasicFungibleFaucet as NativeBasicFungibleFaucet;
use js_export_macro::js_export;

use super::account::Account;
use super::felt::Felt;
use super::token_symbol::TokenSymbol;
use crate::js_error_with_context;
use crate::platform::JsErr;

/// Provides metadata for a basic fungible faucet account component.
#[js_export]
pub struct BasicFungibleFaucetComponent(NativeBasicFungibleFaucet);

#[js_export]
impl BasicFungibleFaucetComponent {
    /// Extracts faucet metadata from an account.
    #[js_export(js_name = "fromAccount")]
    pub fn from_account(account: Account) -> Result<Self, JsErr> {
        let native_account: NativeAccount = account.into();
        let native_faucet = NativeBasicFungibleFaucet::try_from(native_account).map_err(|e| {
            js_error_with_context(e, "failed to get basic fungible faucet details from account")
        })?;
        Ok(native_faucet.into())
    }

    /// Returns the faucet's token symbol.
    pub fn symbol(&self) -> TokenSymbol {
        self.0.symbol().into()
    }

    /// Returns the number of decimal places for the token.
    pub fn decimals(&self) -> u8 {
        self.0.decimals()
    }

    /// Returns the maximum token supply.
    #[js_export(js_name = "maxSupply")]
    pub fn max_supply(&self) -> Felt {
        self.0.max_supply().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeBasicFungibleFaucet> for BasicFungibleFaucetComponent {
    fn from(native_basic_fungible_faucet: NativeBasicFungibleFaucet) -> Self {
        BasicFungibleFaucetComponent(native_basic_fungible_faucet)
    }
}

impl From<BasicFungibleFaucetComponent> for NativeBasicFungibleFaucet {
    fn from(basic_fungible_faucet: BasicFungibleFaucetComponent) -> Self {
        basic_fungible_faucet.0
    }
}
