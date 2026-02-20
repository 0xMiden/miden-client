use core::str::FromStr;

use miden_client::Felt as NativeFelt;
use miden_client::account::{AccountId as NativeAccountId, NetworkId as NativeNetworkId};
use miden_client::address::{
    Address,
    AddressId,
    AddressInterface as NativeAccountInterface,
    CustomNetworkId,
    RoutingParameters,
};

use crate::prelude::*;
use super::felt::Felt;

/// Uniquely identifies a specific account.
///
/// A Miden account ID is a 120-bit value derived from the commitments to account code and storage,
/// and a random user-provided seed.
#[bindings]
#[derive(Clone, Copy, Debug)]
pub struct AccountId(pub(crate) NativeAccountId);

/// The type of a Miden network.
#[derive(Default)]
#[bindings(napi(string_enum))]
pub enum NetworkType {
    /// Main network prefix (`mm`).
    #[default]
    Mainnet,
    /// Public test network prefix (`mtst`).
    Testnet,
    /// Developer network prefix (`mdev`).
    Devnet,
    /// Custom network prefix.
    Custom,
}

/// The identifier of a Miden network.
#[bindings]
pub struct NetworkId {
    // Specific type of the network ID.
    network_type: NetworkType,
    // custom prefix is only used when the inner network is set to custom
    custom: Option<CustomNetworkId>,
}

#[bindings]
impl NetworkId {
    #[bindings(napi(factory))]
    pub fn mainnet() -> NetworkId {
        NetworkId {
            network_type: NetworkType::Mainnet,
            custom: None,
        }
    }

    #[bindings(napi(factory))]
    pub fn testnet() -> NetworkId {
        NetworkId {
            network_type: NetworkType::Testnet,
            custom: None,
        }
    }

    #[bindings(napi(factory))]
    pub fn devnet() -> NetworkId {
        NetworkId {
            network_type: NetworkType::Devnet,
            custom: None,
        }
    }

    /// Builds a custom network ID from a provided custom prefix.
    ///
    /// Returns an error if the prefix is invalid.
    #[bindings(napi(factory))]
    pub fn custom(custom_prefix: String) -> JsResult<NetworkId> {
        let custom = CustomNetworkId::from_str(&custom_prefix)
            .map_err(|err| platform::error_with_context(err, "Error building custom id prefix"))?;

        Ok(NetworkId {
            network_type: NetworkType::Custom,
            custom: Some(custom),
        })
    }
}

/// Account interface type.
#[bindings(napi(string_enum))]
pub enum AccountInterface {
    /// Basic wallet address interface.
    BasicWallet,
}

#[bindings]
impl AccountId {
    /// Returns true if the ID refers to a faucet.
    pub fn is_faucet(&self) -> bool {
        self.0.is_faucet()
    }

    /// Returns true if the ID refers to a regular account.
    pub fn is_regular_account(&self) -> bool {
        self.0.is_regular_account()
    }

    /// Returns true if the account uses public storage.
    pub fn is_public(&self) -> bool {
        self.0.is_public()
    }

    /// Returns true if the account uses private storage.
    pub fn is_private(&self) -> bool {
        self.0.is_private()
    }

    /// Returns true if the ID is reserved for network accounts.
    pub fn is_network(&self) -> bool {
        self.0.is_network()
    }

    /// Returns the canonical hex representation of the account ID.
    #[bindings(napi(js_name = "toString"))]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string_js(&self) -> String {
        self.0.to_string()
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

#[bindings]
impl AccountId {
    /// Builds an account ID from its hex string representation.
    #[bindings(napi(factory))]
    pub fn from_hex(hex: String) -> JsResult<AccountId> {
        let native_account_id = NativeAccountId::from_hex(&hex)
            .map_err(|e| platform::error_from_string(&format!("Invalid account ID hex: {e}")))?;
        Ok(AccountId(native_account_id))
    }

    /// Given a bech32 encoded string, return the matching Account ID for it.
    #[bindings(napi(factory))]
    pub fn from_bech32(bech_32_encoded_id: String) -> JsResult<AccountId> {
        let (_, address) = Address::decode(&bech_32_encoded_id).map_err(|err| {
            platform::error_with_context(
                err,
                "could not interpret input as a bech32-encoded account id",
            )
        })?;
        match address.id() {
            AddressId::AccountId(account_id) => Ok(account_id.into()),
            _unsupported => Err(platform::error_from_string(
                "bech32 string decoded into an unsupported address kind",
            )),
        }
    }
}

// to_bech32 must stay separate: wasm takes NetworkId by value, napi by reference
#[cfg(feature = "wasm")]
#[bindings(wasm)]
impl AccountId {
    /// Will turn the Account ID into its bech32 string representation.
    #[bindings(wasm(js_name = "toBech32"))]
    pub fn to_bech32(
        &self,
        network_id: NetworkId,
        account_interface: AccountInterface,
    ) -> JsResult<String> {
        let network_id: NativeNetworkId = network_id.into();

        let routing_params = RoutingParameters::new(account_interface.into());
        let address = Address::new(self.0)
            .with_routing_parameters(routing_params)
            .map_err(|err| platform::error_with_context(err, "failed to set routing parameters"))?;
        Ok(address.encode(network_id))
    }
}

#[cfg(feature = "napi")]
#[bindings(napi)]
impl AccountId {
    /// Will turn the Account ID into its bech32 string representation.
    pub fn to_bech32(
        &self,
        network_id: &NetworkId,
        account_interface: AccountInterface,
    ) -> JsResult<String> {
        let native_network_id: NativeNetworkId = network_id.into();
        let routing_params = RoutingParameters::new(account_interface.into());
        let address = Address::new(self.0)
            .with_routing_parameters(routing_params)
            .map_err(|err| platform::error_with_context(err, "failed to set routing parameters"))?;
        Ok(address.encode(native_network_id))
    }
}

impl AccountId {
    pub(crate) fn as_native(&self) -> &NativeAccountId {
        &self.0
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

// wasm takes NetworkId by value
#[cfg(feature = "wasm")]
impl From<NetworkId> for NativeNetworkId {
    fn from(value: NetworkId) -> Self {
        match value.network_type {
            NetworkType::Mainnet => NativeNetworkId::Mainnet,
            NetworkType::Testnet => NativeNetworkId::Testnet,
            NetworkType::Devnet => NativeNetworkId::Devnet,
            NetworkType::Custom => {
                let custom_prefix =
                    value.custom.expect("custom network id constructor implies existing prefix");
                NativeNetworkId::from_str(custom_prefix.as_str())
                    .expect("custom network id constructor implies valid prefix")
            },
        }
    }
}

// napi takes NetworkId by reference
#[cfg(feature = "napi")]
impl From<&NetworkId> for NativeNetworkId {
    fn from(value: &NetworkId) -> Self {
        match value.network_type {
            NetworkType::Mainnet => NativeNetworkId::Mainnet,
            NetworkType::Testnet => NativeNetworkId::Testnet,
            NetworkType::Devnet => NativeNetworkId::Devnet,
            NetworkType::Custom => {
                let custom_prefix = value
                    .custom
                    .as_ref()
                    .expect("custom network id constructor implies existing prefix");
                NativeNetworkId::from_str(custom_prefix.as_str())
                    .expect("custom network id constructor implies valid prefix")
            },
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
