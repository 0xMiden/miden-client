use miden_client::account::Account as NativeAccount;
use miden_client::account::component::BasicFungibleFaucet as NativeBasicFungibleFaucet;
use wasm_bindgen::prelude::*;

use super::account::Account;
use super::felt::Felt;
use super::token_symbol::TokenSymbol;
use crate::js_error_with_context;

/// View over a basic fungible faucet account component.
#[wasm_bindgen]
pub struct BasicFungibleFaucetComponent(NativeBasicFungibleFaucet);

#[wasm_bindgen]
impl BasicFungibleFaucetComponent {
    #[wasm_bindgen(js_name = "fromAccount")]
    /// Extracts faucet metadata from an existing account.
    ///
    /// @throws Throws if the account is not a basic fungible faucet.
    pub fn from_account(account: Account) -> Result<Self, JsValue> {
        let native_account: NativeAccount = account.into();
        let native_faucet = NativeBasicFungibleFaucet::try_from(native_account).map_err(|e| {
            js_error_with_context(e, "failed to get basic fungible faucet details from account")
        })?;
        Ok(native_faucet.into())
    }

    /// Returns the faucet token symbol.
    pub fn symbol(&self) -> TokenSymbol {
        self.0.symbol().into()
    }

    /// Returns the number of decimals used by the faucet token.
    pub fn decimals(&self) -> u8 {
        self.0.decimals()
    }

    #[wasm_bindgen(js_name = "maxSupply")]
    /// Returns the maximum supply the faucet is allowed to mint.
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
