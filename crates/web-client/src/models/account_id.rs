use std::str::FromStr;

use miden_client::Felt as NativeFelt;
use miden_client::account::AccountId as NativeAccountId;
use miden_client::address::{
    Address, AddressInterface as NativeAccountInterface, NetworkId as NativeNetworkId,
    RoutingParameters,
};
use wasm_bindgen::prelude::*;

use super::felt::Felt;
use crate::js_error_with_context;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct AccountId(NativeAccountId);

#[wasm_bindgen]
#[repr(u8)]
pub enum NetworkId {
    Mainnet = 0,
    Testnet = 1,
    Devnet = 2,
}

#[wasm_bindgen]
#[repr(u8)]
pub enum AccountInterface {
    BasicWallet = 0,
}

#[wasm_bindgen]
impl AccountId {
    #[wasm_bindgen(js_name = "fromHex")]
    pub fn from_hex(hex: &str) -> AccountId {
        let native_account_id = NativeAccountId::from_hex(hex).unwrap();
        AccountId(native_account_id)
    }

    #[wasm_bindgen(js_name = "isFaucet")]
    pub fn is_faucet(&self) -> bool {
        self.0.is_faucet()
    }

    #[wasm_bindgen(js_name = "isRegularAccount")]
    pub fn is_regular_account(&self) -> bool {
        self.0.is_regular_account()
    }

    #[wasm_bindgen(js_name = "isPublic")]
    pub fn is_public(&self) -> bool {
        self.0.is_public()
    }

    #[wasm_bindgen(js_name = "isPrivate")]
    pub fn is_private(&self) -> bool {
        self.0.is_private()
    }

    #[wasm_bindgen(js_name = "isNetwork")]
    pub fn is_network(&self) -> bool {
        self.0.is_network()
    }

    #[wasm_bindgen(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Will turn the Account ID into its bech32 string representation. To avoid a potential
    /// wrongful encoding, this function will expect only IDs for either mainnet ("mm"),
    /// testnet ("mtst") or devnet ("mdev"). To use a custom bech32 prefix, see
    /// `Self::to_bech_32_custom`.
    #[wasm_bindgen(js_name = "toBech32")]
    pub fn to_bech32(
        &self,
        network_id: NetworkId,
        account_interface: AccountInterface,
    ) -> Result<String, JsValue> {
        let network_id: NativeNetworkId = network_id.into();

        let routing_params = RoutingParameters::new(account_interface.into());
        let address = Address::new(self.0)
            .with_routing_parameters(routing_params)
            .map_err(|err| js_error_with_context(err, "failed to set routing parameters"))?;
        Ok(address.encode(network_id))
    }

    /// Turn this Account ID into its bech32 string representation. This method accepts a custom
    /// network ID.
    #[wasm_bindgen(js_name = "toBech32Custom")]
    pub fn to_bech32_custom(
        &self,
        custom_network_id: &str,
        account_interface: AccountInterface,
    ) -> Result<String, JsValue> {
        let network_id = NativeNetworkId::from_str(custom_network_id)
            .map_err(|err| js_error_with_context(err, "given network id is not valid"))?;

        let routing_params = RoutingParameters::new(account_interface.into());
        let address = Address::new(self.0)
            .with_routing_parameters(routing_params)
            .map_err(|err| js_error_with_context(err, "failed to set routing parameters"))?;
        Ok(address.encode(network_id))
    }

    pub fn prefix(&self) -> Felt {
        let native_felt: NativeFelt = self.0.prefix().as_felt();
        native_felt.into()
    }

    pub fn suffix(&self) -> Felt {
        let native_felt: NativeFelt = self.0.suffix();
        native_felt.into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountId> for AccountId {
    fn from(native_account_id: NativeAccountId) -> Self {
        AccountId(native_account_id)
    }
}

impl From<&NativeAccountId> for AccountId {
    fn from(native_account_id: &NativeAccountId) -> Self {
        AccountId(*native_account_id)
    }
}

impl From<AccountId> for NativeAccountId {
    fn from(account_id: AccountId) -> Self {
        account_id.0
    }
}

impl From<&AccountId> for NativeAccountId {
    fn from(account_id: &AccountId) -> Self {
        account_id.0
    }
}

impl From<NetworkId> for NativeNetworkId {
    fn from(value: NetworkId) -> Self {
        match value {
            NetworkId::Mainnet => NativeNetworkId::Mainnet,
            NetworkId::Testnet => NativeNetworkId::Testnet,
            NetworkId::Devnet => NativeNetworkId::Devnet,
        }
    }
}

impl From<AccountInterface> for NativeAccountInterface {
    fn from(account_interface: AccountInterface) -> Self {
        match account_interface {
            AccountInterface::BasicWallet => NativeAccountInterface::BasicWallet,
        }
    }
}
