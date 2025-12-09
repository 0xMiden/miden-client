use std::str::FromStr;

use miden_client::Felt as NativeFelt;
use miden_client::account::{AccountId as NativeAccountId, NetworkId as NativeNetworkId};
use miden_client::address::{
    Address,
    AddressInterface as NativeAccountInterface,
    RoutingParameters,
};
use wasm_bindgen::prelude::*;

use super::felt::Felt;
use crate::js_error_with_context;

/// Uniquely identifies a specific account.
///
/// A Miden account ID is a 120-bit value derived from the commitments to account code and storage,
/// and a random user-provided seed.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct AccountId(NativeAccountId);

#[wasm_bindgen]
#[repr(u8)]
pub enum NetworkId {
    /// Main network prefix (`mm`).
    Mainnet = 0,
    /// Public test network prefix (`mtst`).
    Testnet = 1,
    /// Developer network prefix (`mdev`).
    Devnet = 2,
}

#[wasm_bindgen]
#[repr(u8)]
pub enum AccountInterface {
    /// Basic wallet address interface.
    BasicWallet = 0,
}

#[wasm_bindgen]
impl AccountId {
    /// Builds an account ID from its hex string representation.
    #[wasm_bindgen(js_name = "fromHex")]
    pub fn from_hex(hex: &str) -> AccountId {
        let native_account_id = NativeAccountId::from_hex(hex).unwrap();
        AccountId(native_account_id)
    }

    /// Returns true if the ID refers to a faucet.
    #[wasm_bindgen(js_name = "isFaucet")]
    pub fn is_faucet(&self) -> bool {
        self.0.is_faucet()
    }

    /// Returns true if the ID refers to a regular account.
    #[wasm_bindgen(js_name = "isRegularAccount")]
    pub fn is_regular_account(&self) -> bool {
        self.0.is_regular_account()
    }

    /// Returns true if the account uses public storage.
    #[wasm_bindgen(js_name = "isPublic")]
    pub fn is_public(&self) -> bool {
        self.0.is_public()
    }

    /// Returns true if the account uses private storage.
    #[wasm_bindgen(js_name = "isPrivate")]
    pub fn is_private(&self) -> bool {
        self.0.is_private()
    }

    /// Returns true if the ID is reserved for network accounts.
    #[wasm_bindgen(js_name = "isNetwork")]
    pub fn is_network(&self) -> bool {
        self.0.is_network()
    }

    /// Returns the canonical hex representation of the account ID.
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

    /// Given a bech32 encoded string, return the matching Account ID for it.
    #[wasm_bindgen(js_name = "fromBech32")]
    pub fn from_bech32(bech_32_encoded_id: &str) -> Result<AccountId, JsValue> {
        let to_decode = {
            // Since a bech32 string can have additional data, this split
            // should be able to fetch the account id, which is what we want
            // to decode.
            // Reference: https://github.com/0xMiden/miden-base/blob/150a8066c5a4b4011c4f3e55f9435921ad3835f3/docs/src/account/address.md#structure
            if let Some((account_id, _routing_params)) = bech_32_encoded_id.split_once('_') {
                account_id
            } else {
                bech_32_encoded_id
            }
        };
        let (_, account_id) = NativeAccountId::from_bech32(to_decode).map_err(|err| {
            js_error_with_context(err, "could not interpret input as a bech32-encoded account id")
        })?;
        Ok(account_id.into())
    }

    /// Returns the prefix field element storing metadata about version, type, and storage mode.
    pub fn prefix(&self) -> Felt {
        let native_felt: NativeFelt = self.0.prefix().as_felt();
        native_felt.into()
    }

    /// Returns the suffix field element derived from the account seed.
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
