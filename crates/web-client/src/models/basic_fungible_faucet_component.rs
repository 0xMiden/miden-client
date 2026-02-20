use miden_client::account::Account as NativeAccount;
use miden_client::account::component::BasicFungibleFaucet as NativeBasicFungibleFaucet;

use super::account::Account;
use super::felt::Felt;
use super::token_symbol::TokenSymbol;
use crate::platform;
use crate::prelude::*;

/// Provides metadata for a basic fungible faucet account component.
#[bindings]
pub struct BasicFungibleFaucetComponent(NativeBasicFungibleFaucet);

// Shared methods
#[bindings]
impl BasicFungibleFaucetComponent {
    /// Returns the faucet's token symbol.
    #[bindings(getter)]
    pub fn symbol(&self) -> TokenSymbol {
        self.0.symbol().into()
    }

    /// Returns the number of decimal places for the token.
    #[bindings(getter)]
    pub fn decimals(&self) -> u8 {
        self.0.decimals()
    }

    /// Returns the maximum token supply.
    #[bindings(getter)]
    pub fn max_supply(&self) -> Felt {
        self.0.max_supply().into()
    }
}

// from_account: wasm takes owned Account, napi takes &Account + factory
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl BasicFungibleFaucetComponent {
    
    pub fn from_account(account: Account) -> platform::JsResult<Self> {
        let native_account: NativeAccount = account.into();
        let native_faucet = NativeBasicFungibleFaucet::try_from(native_account).map_err(|e| {
            platform::error_with_context(
                e,
                "failed to get basic fungible faucet details from account",
            )
        })?;
        Ok(native_faucet.into())
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl BasicFungibleFaucetComponent {
    #[napi(factory)]
    pub fn from_account(account: &Account) -> platform::JsResult<Self> {
        let native_account: NativeAccount = account.into();
        let native_faucet = NativeBasicFungibleFaucet::try_from(native_account).map_err(|e| {
            platform::error_with_context(
                e,
                "failed to get basic fungible faucet details from account",
            )
        })?;
        Ok(native_faucet.into())
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
