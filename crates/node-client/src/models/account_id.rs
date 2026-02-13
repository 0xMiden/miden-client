use std::str::FromStr;

use miden_client::Felt as NativeFelt;
use miden_client::account::{AccountId as NativeAccountId, NetworkId as NativeNetworkId};
use miden_client::address::{
    Address,
    AddressId,
    AddressInterface as NativeAccountInterface,
    RoutingParameters,
};
use napi::bindgen_prelude::*;

use super::felt::Felt;
use super::napi_wrap;

napi_wrap!(copy AccountId wraps NativeAccountId);

#[napi]
impl AccountId {
    /// Builds an account ID from its hex string representation.
    #[napi(js_name = "fromHex")]
    pub fn from_hex(hex: String) -> Result<AccountId> {
        let native = NativeAccountId::from_hex(&hex).map_err(|err| {
            napi::Error::from_reason(format!("Failed to parse AccountId from hex: {err}"))
        })?;
        Ok(AccountId(native))
    }

    /// Returns true if the ID refers to a faucet.
    #[napi(js_name = "isFaucet")]
    pub fn is_faucet(&self) -> bool {
        self.0.is_faucet()
    }

    /// Returns true if the ID refers to a regular account.
    #[napi(js_name = "isRegularAccount")]
    pub fn is_regular_account(&self) -> bool {
        self.0.is_regular_account()
    }

    /// Returns true if the account uses public storage.
    #[napi(js_name = "isPublic")]
    pub fn is_public(&self) -> bool {
        self.0.is_public()
    }

    /// Returns true if the account uses private storage.
    #[napi(js_name = "isPrivate")]
    pub fn is_private(&self) -> bool {
        self.0.is_private()
    }

    /// Returns true if the ID is reserved for network accounts.
    #[napi(js_name = "isNetwork")]
    pub fn is_network(&self) -> bool {
        self.0.is_network()
    }

    /// Returns the canonical hex representation of the account ID.
    #[napi(js_name = "toString")]
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }

    /// Returns the prefix field element.
    #[napi]
    pub fn prefix(&self) -> Felt {
        let native_felt: NativeFelt = self.0.prefix().as_felt();
        native_felt.into()
    }

    /// Returns the suffix field element.
    #[napi]
    pub fn suffix(&self) -> Felt {
        let native_felt: NativeFelt = self.0.suffix();
        native_felt.into()
    }

    /// Converts to bech32 representation with a given network.
    #[napi(js_name = "toBech32")]
    pub fn to_bech32(&self, network: String) -> Result<String> {
        let network_id = parse_network_id(&network)?;
        let routing_params = RoutingParameters::new(NativeAccountInterface::BasicWallet);
        let address =
            Address::new(self.0).with_routing_parameters(routing_params).map_err(|err| {
                napi::Error::from_reason(format!("failed to set routing parameters: {err}"))
            })?;
        Ok(address.encode(network_id))
    }

    /// Given a bech32 encoded string, return the matching Account ID for it.
    #[napi(js_name = "fromBech32")]
    pub fn from_bech32(bech32: String) -> Result<AccountId> {
        let (_, address) = Address::decode(&bech32).map_err(|err| {
            napi::Error::from_reason(format!(
                "could not interpret input as a bech32-encoded account id: {err}"
            ))
        })?;
        match address.id() {
            AddressId::AccountId(account_id) => Ok(AccountId(account_id)),
            _ => Err(napi::Error::from_reason(
                "bech32 string decoded into an unsupported address kind",
            )),
        }
    }
}

fn parse_network_id(network: &str) -> Result<NativeNetworkId> {
    match network {
        "mainnet" => Ok(NativeNetworkId::Mainnet),
        "testnet" => Ok(NativeNetworkId::Testnet),
        "devnet" => Ok(NativeNetworkId::Devnet),
        other => NativeNetworkId::from_str(other)
            .map_err(|err| napi::Error::from_reason(format!("Invalid network id: {err}"))),
    }
}
