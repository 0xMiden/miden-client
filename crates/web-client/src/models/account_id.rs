use std::str::FromStr;

use miden_client::Felt as NativeFelt;
use miden_client::account::AccountId as NativeAccountId;
use miden_client::address::{
    AccountIdAddress,
    Address,
    AddressInterface as NativeAccountInterface,
    NetworkId as NativeNetworkId,
};
use wasm_bindgen::prelude::*;

use super::felt::Felt;
use crate::js_error_with_context;

/// Identifier for an account exposed to JavaScript.
///
/// Wraps [`miden_client::account::AccountId`] and provides convenience helpers for formatting and
/// network-aware conversions.
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct AccountId(NativeAccountId);

/// Known network prefixes supported by the SDK.
#[wasm_bindgen]
#[repr(u8)]
pub enum NetworkId {
    /// Miden mainnet.
    Mainnet = 0,
    /// Miden testnet.
    Testnet = 1,
    /// Miden devnet.
    Devnet = 2,
}

/// Identifies the interface contract implemented by an account.
#[wasm_bindgen]
#[repr(u8)]
pub enum AccountInterface {
    /// Account interface is unspecified.
    Unspecified = 0,
    /// Basic wallet interface.
    BasicWallet = 1,
}

#[wasm_bindgen]
impl AccountId {
    #[wasm_bindgen(js_name = "fromHex")]
    /// Parses an account identifier from a hex string.
    pub fn from_hex(hex: &str) -> AccountId {
        let native_account_id = NativeAccountId::from_hex(hex).unwrap();
        AccountId(native_account_id)
    }

    #[wasm_bindgen(js_name = "isFaucet")]
    /// Returns `true` if the identifier belongs to a faucet account.
    pub fn is_faucet(&self) -> bool {
        self.0.is_faucet()
    }

    #[wasm_bindgen(js_name = "isRegularAccount")]
    /// Returns `true` if the identifier belongs to a regular account.
    pub fn is_regular_account(&self) -> bool {
        self.0.is_regular_account()
    }

    #[wasm_bindgen(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    /// Returns the canonical hex representation of this identifier.
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

        let address: Address = AccountIdAddress::new(self.0, account_interface.into()).into();
        Ok(address.to_bech32(network_id))
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

        let address: Address = AccountIdAddress::new(self.0, account_interface.into()).into();
        Ok(address.to_bech32(network_id))
    }

    /// Returns the high-word prefix of the account identifier.
    pub fn prefix(&self) -> Felt {
        let native_felt: NativeFelt = self.0.prefix().as_felt();
        native_felt.into()
    }

    /// Returns the low-word suffix of the account identifier.
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
            AccountInterface::Unspecified => NativeAccountInterface::Unspecified,
        }
    }
}
