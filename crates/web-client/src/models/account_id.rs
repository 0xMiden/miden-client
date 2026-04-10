use alloc::str::FromStr;

use js_export_macro::js_export;
use miden_client::Felt as NativeFelt;
use miden_client::account::{AccountId as NativeAccountId, NetworkId as NativeNetworkId};
use miden_client::address::{
    Address,
    AddressId,
    AddressInterface as NativeAccountInterface,
    CustomNetworkId,
    RoutingParameters,
};

use super::felt::Felt;
use crate::js_error_with_context;
use crate::platform::{JsErr, from_str_err};

/// Uniquely identifies a specific account.
///
/// A Miden account ID is a 120-bit value derived from the commitments to account code and storage,
/// and a random user-provided seed.
#[js_export]
#[derive(Clone, Copy, Debug)]
pub struct AccountId(NativeAccountId);

/// The type of a Miden network.
#[derive(Clone, Copy)]
#[js_export]
pub enum NetworkType {
    /// Main network prefix (`mm`).
    Mainnet = 0,
    /// Public test network prefix (`mtst`).
    Testnet = 1,
    /// Developer network prefix (`mdev`).
    Devnet = 2,
    /// Custom network prefix.
    Custom = 3,
}

/// The identifier of a Miden network.
#[derive(Clone)]
#[js_export]
pub struct NetworkId {
    // Specific type of the network ID.
    network_type: NetworkType,
    // custom prefix is only used when the inner network is set to custom
    custom: Option<CustomNetworkId>,
}

#[js_export]
impl NetworkId {
    pub fn mainnet() -> NetworkId {
        NetworkId {
            network_type: NetworkType::Mainnet,
            custom: None,
        }
    }

    pub fn testnet() -> NetworkId {
        NetworkId {
            network_type: NetworkType::Testnet,
            custom: None,
        }
    }

    pub fn devnet() -> NetworkId {
        NetworkId {
            network_type: NetworkType::Devnet,
            custom: None,
        }
    }

    /// Builds a custom network ID from a provided custom prefix.
    ///
    /// Returns an error if the prefix is invalid.
    pub fn custom(custom_prefix: String) -> Result<NetworkId, JsErr> {
        let custom = CustomNetworkId::from_str(&custom_prefix)
            .map_err(|err| js_error_with_context(err, "Error building custom id prefix"))?;

        Ok(NetworkId {
            network_type: NetworkType::Custom,
            custom: Some(custom),
        })
    }
}

#[js_export]
#[repr(u8)]
pub enum AccountInterface {
    /// Basic wallet address interface.
    BasicWallet = 0,
}

// Internal methods accessible from Rust code (not processed by napi/wasm_bindgen).
impl AccountId {
    /// Returns true if the account uses public storage.
    pub(crate) fn is_public(&self) -> bool {
        self.0.is_public()
    }

    /// Returns true if the account uses private storage.
    pub(crate) fn is_private(&self) -> bool {
        self.0.is_private()
    }

    /// Returns true if the ID is reserved for network accounts.
    pub(crate) fn is_network(&self) -> bool {
        self.0.is_network()
    }

    pub(crate) fn as_native(&self) -> &NativeAccountId {
        &self.0
    }
}

#[js_export]
impl AccountId {
    /// Builds an account ID from its hex string representation.
    ///
    /// Returns an error if the provided string is not a valid hex-encoded account ID.
    #[js_export(js_name = "fromHex")]
    pub fn from_hex(hex: String) -> Result<AccountId, JsErr> {
        let native_account_id = NativeAccountId::from_hex(&hex)
            .map_err(|err| js_error_with_context(err, "error instantiating AccountId from hex"))?;
        Ok(AccountId(native_account_id))
    }

    /// Builds an account ID from its prefix and suffix field elements.
    ///
    /// This is useful when the account ID components are stored separately (e.g., in storage
    /// maps) and need to be recombined into an `AccountId`.
    ///
    /// Returns an error if the provided felts do not form a valid account ID.
    #[js_export(js_name = "fromPrefixSuffix")]
    pub fn from_prefix_suffix(prefix: &Felt, suffix: &Felt) -> Result<AccountId, JsErr> {
        let prefix_felt: NativeFelt = (*prefix).into();
        let suffix_felt: NativeFelt = (*suffix).into();
        let native_account_id = NativeAccountId::try_from_elements(suffix_felt, prefix_felt)
            .map_err(|err| {
                js_error_with_context(err, "error instantiating AccountId from prefix and suffix")
            })?;
        Ok(AccountId(native_account_id))
    }

    /// Returns true if the ID refers to a faucet.
    #[js_export(js_name = "isFaucet")]
    pub fn is_faucet(&self) -> bool {
        self.0.is_faucet()
    }

    /// Returns true if the ID refers to a regular account.
    #[js_export(js_name = "isRegularAccount")]
    pub fn is_regular_account(&self) -> bool {
        self.0.is_regular_account()
    }

    /// Returns true if the account uses public storage.
    #[js_export(js_name = "isPublic")]
    pub fn js_is_public(&self) -> bool {
        self.0.is_public()
    }

    /// Returns true if the account uses private storage.
    #[js_export(js_name = "isPrivate")]
    pub fn js_is_private(&self) -> bool {
        self.0.is_private()
    }

    /// Returns true if the ID is reserved for network accounts.
    #[js_export(js_name = "isNetwork")]
    pub fn js_is_network(&self) -> bool {
        self.0.is_network()
    }

    /// Returns the canonical hex representation of the account ID.
    #[js_export(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Will turn the Account ID into its bech32 string representation.
    #[js_export(js_name = "toBech32")]
    pub fn to_bech32(
        &self,
        network_id: NetworkId,
        account_interface: AccountInterface,
    ) -> Result<String, JsErr> {
        let network_id: NativeNetworkId = network_id.into();

        let routing_params = RoutingParameters::new(account_interface.into());
        let address = Address::new(self.0).with_routing_parameters(routing_params);
        Ok(address.encode(network_id))
    }

    /// Given a bech32 encoded string, return the matching Account ID for it.
    #[js_export(js_name = "fromBech32")]
    pub fn from_bech32(bech_32_encoded_id: String) -> Result<AccountId, JsErr> {
        // Since a bech32 encodes an account id + a routing parameter,
        // we can use Address::decode to fetch the account id.
        // Reference: https://github.com/0xMiden/miden-base/blob/150a8066c5a4b4011c4f3e55f9435921ad3835f3/docs/src/account/address.md#structure
        let (_, address) = Address::decode(&bech_32_encoded_id).map_err(|err| {
            js_error_with_context(err, "could not interpret input as a bech32-encoded account id")
        })?;
        match address.id() {
            AddressId::AccountId(account_id) => Ok(account_id.into()),
            _unsupported => {
                Err(from_str_err("bech32 string decoded into an unsupported address kind"))
            },
        }
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

impl From<AccountInterface> for NativeAccountInterface {
    fn from(account_interface: AccountInterface) -> Self {
        match account_interface {
            AccountInterface::BasicWallet => NativeAccountInterface::BasicWallet,
        }
    }
}

impl_napi_from_value!(AccountId);
impl_napi_from_value!(NetworkId);
