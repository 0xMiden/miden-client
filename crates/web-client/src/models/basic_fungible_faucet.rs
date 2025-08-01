use miden_lib::account::faucets::BasicFungibleFaucet as NativeBasicFungibleFaucet;
use miden_objects::account::Account as NativeAccount;
use wasm_bindgen::prelude::*;

use super::{account::Account, felt::Felt, token_symbol::TokenSymbol};
use crate::js_error_with_context;

#[wasm_bindgen]
pub struct BasicFungibleFaucet(NativeBasicFungibleFaucet);

#[wasm_bindgen]
impl BasicFungibleFaucet {
    #[wasm_bindgen(js_name = "getFaucetDetailsFromAccount")]
    pub fn get_faucet_details_from_account(account: Account) -> Result<Self, JsValue> {
        let native_account: NativeAccount = account.into();
        let native_faucet = NativeBasicFungibleFaucet::try_from(native_account).map_err(|e| {
            js_error_with_context(e, "failed to get basic fungible faucet details from account")
        })?;
        Ok(native_faucet.into())
    }

    pub fn symbol(&self) -> TokenSymbol {
        self.0.symbol().into()
    }

    pub fn decimals(&self) -> u8 {
        self.0.decimals()
    }

    #[wasm_bindgen(js_name = "maxSupply")]
    pub fn max_supply(&self) -> Felt {
        self.0.max_supply().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeBasicFungibleFaucet> for BasicFungibleFaucet {
    fn from(native_basic_fungible_faucet: NativeBasicFungibleFaucet) -> Self {
        BasicFungibleFaucet(native_basic_fungible_faucet)
    }
}

impl From<BasicFungibleFaucet> for NativeBasicFungibleFaucet {
    fn from(basic_fungible_faucet: BasicFungibleFaucet) -> Self {
        basic_fungible_faucet.0
    }
}
