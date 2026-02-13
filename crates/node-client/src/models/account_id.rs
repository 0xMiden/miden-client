use std::str::FromStr;

use miden_client::account::{AccountId as NativeAccountId, NetworkId as NativeNetworkId};
use miden_client::address::{
    Address,
    AddressId,
    AddressInterface as NativeAccountInterface,
    RoutingParameters,
};
use napi::bindgen_prelude::*;

use super::felt::Felt;
use super::{napi_delegate, napi_wrap};

napi_wrap!(copy AccountId wraps NativeAccountId);

napi_delegate!(impl AccountId {
    /// Returns true if the ID refers to a faucet.
    delegate is_faucet -> bool;
    /// Returns true if the ID refers to a regular account.
    delegate is_regular_account -> bool;
    /// Returns true if the account uses public storage.
    delegate is_public -> bool;
    /// Returns true if the account uses private storage.
    delegate is_private -> bool;
    /// Returns true if the ID is reserved for network accounts.
    delegate is_network -> bool;
});

#[napi]
impl AccountId {
    /// Builds an account ID from its hex string representation.
    #[napi]
    pub fn from_hex(hex: String) -> Result<AccountId> {
        let native = NativeAccountId::from_hex(&hex).map_err(|err| {
            napi::Error::from_reason(format!("Failed to parse AccountId from hex: {err}"))
        })?;
        Ok(AccountId(native))
    }

    /// Returns the canonical hex representation of the account ID.
    #[napi(js_name = "toString")]
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }

    /// Returns the prefix field element.
    #[napi]
    pub fn prefix(&self) -> Felt {
        self.0.prefix().as_felt().into()
    }

    /// Returns the suffix field element.
    #[napi]
    pub fn suffix(&self) -> Felt {
        self.0.suffix().into()
    }

    /// Converts to bech32 representation with a given network.
    #[napi]
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
    #[napi]
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
