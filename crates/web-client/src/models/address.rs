use miden_client::account::AccountId as NativeAccountId;
use miden_client::address::{
    Address as NativeAddress,
    AddressId,
    AddressInterface as NativeAddressInterface,
    NetworkId as NativeNetworkId,
    RoutingParameters,
};
use js_export_macro::js_export;
#[cfg(feature = "browser")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "nodejs")]
use napi_derive::napi;

use super::NoteTag;
use super::account_id::{AccountId, NetworkId};
use crate::js_error_with_context;
use crate::platform::{JsBytes, JsErr, from_str_err};
use crate::utils::deserialize_from_bytes;

/// Representation of a Miden address (account ID plus routing parameters).
#[cfg_attr(feature = "browser", wasm_bindgen(inspectable))]
#[cfg_attr(feature = "nodejs", napi)]
#[derive(Clone, Debug)]
pub struct Address(NativeAddress);

/// Specifies which procedures an account accepts, and by extension which notes it can consume.
#[cfg(feature = "browser")]
#[wasm_bindgen]
pub enum AddressInterface {
    BasicWallet = "BasicWallet",
}

/// Specifies which procedures an account accepts, and by extension which notes it can consume.
#[cfg(feature = "nodejs")]
#[napi]
#[repr(u8)]
pub enum AddressInterface {
    BasicWallet = 0,
}

#[js_export]
impl Address {
    /// Deserializes a byte array into an `Address`.
    pub fn deserialize(bytes: JsBytes) -> Result<Address, JsErr> {
        let native_address: NativeAddress = deserialize_from_bytes(&bytes)?;
        Ok(Self(native_address))
    }

    #[js_export(js_name = "fromAccountId")]
    // Can't pass the proper AddressInterface enum here since wasm_bindgen does not derive the ref
    // trait for enum types. But we can still leave its definition since it gets exported as a
    // constant for the JS SDK.
    /// Builds an address from an account ID and optional interface.
    pub fn from_account_id(
        account_id: &AccountId,
        interface: Option<String>,
    ) -> Result<Self, JsErr> {
        let native_account_id: NativeAccountId = account_id.into();
        let native_address = match interface {
            None => NativeAddress::new(native_account_id),
            Some(interface) if &interface == "BasicWallet" => {
                let routing_params = RoutingParameters::new(NativeAddressInterface::BasicWallet);
                NativeAddress::new(native_account_id)
                    .with_routing_parameters(routing_params)
                    .map_err(|err| js_error_with_context(err, "failed to set routing params"))?
            },
            Some(other_interface) => {
                return Err(from_str_err(&format!(
                    "Failed to build address from account id, wrong interface value given: {other_interface}"
                )));
            },
        };

        Ok(Self(native_address))
    }

    /// Builds an address from a bech32-encoded string.
    #[js_export(js_name = "fromBech32")]
    pub fn from_bech32(bech32: String) -> Result<Self, JsErr> {
        let (_net_id, address) = NativeAddress::decode(&bech32).map_err(|err| {
            js_error_with_context(err, "could not convert bech32 into an address")
        })?;
        Ok(Self(address))
    }

    /// Returns the address interface.
    pub fn interface(&self) -> Result<AddressInterface, JsErr> {
        match self.0.interface() {
            Some(interface) => interface.try_into(),
            None => Err(from_str_err("Address has no specified interface")),
        }
    }

    /// Returns the account ID embedded in the address.
    #[js_export(js_name = "accountId")]
    pub fn account_id(&self) -> Result<AccountId, JsErr> {
        match &self.0.id() {
            AddressId::AccountId(account_id_address) => Ok(account_id_address.into()),
            _other => Err(from_str_err("Unsupported Account address type")),
        }
    }

    /// Converts the address into a note tag.
    #[js_export(js_name = "toNoteTag")]
    pub fn to_note_tag(&self) -> NoteTag {
        self.0.to_note_tag().into()
    }

    /// Encodes the address using the provided network prefix.
    #[js_export(js_name = "toBech32")]
    pub fn to_bech32(&self, network_id: NetworkId) -> Result<String, JsErr> {
        let net_id: NativeNetworkId = network_id.into();
        Ok(self.0.encode(net_id))
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAddress> for Address {
    fn from(native_address: NativeAddress) -> Self {
        Address(native_address)
    }
}

impl From<&NativeAddress> for Address {
    fn from(native_address: &NativeAddress) -> Self {
        Address(native_address.clone())
    }
}

impl From<Address> for NativeAddress {
    fn from(address: Address) -> Self {
        address.0
    }
}

impl From<&Address> for NativeAddress {
    fn from(address: &Address) -> Self {
        address.0.clone()
    }
}

impl TryFrom<AddressInterface> for NativeAddressInterface {
    type Error = &'static str;
    fn try_from(value: AddressInterface) -> Result<Self, &'static str> {
        match value {
            AddressInterface::BasicWallet => Ok(NativeAddressInterface::BasicWallet),
            #[cfg(feature = "browser")]
            AddressInterface::__Invalid => Err("Non-valid address interface given"),
        }
    }
}

impl TryFrom<NativeAddressInterface> for AddressInterface {
    type Error = JsErr;
    fn try_from(value: NativeAddressInterface) -> Result<Self, Self::Error> {
        match value {
            NativeAddressInterface::BasicWallet => Ok(AddressInterface::BasicWallet),
            _other => {
                Err(from_str_err("AddressInterface from miden-protocol crate was instantiated with an unsupported value"))
            },
        }
    }
}

impl_napi_from_value!(Address);
