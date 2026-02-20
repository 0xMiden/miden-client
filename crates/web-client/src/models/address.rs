use miden_client::account::AccountId as NativeAccountId;
use miden_client::address::{
    Address as NativeAddress,
    AddressId,
    AddressInterface as NativeAddressInterface,
    NetworkId as NativeNetworkId,
    RoutingParameters,
};

#[cfg(feature = "napi")]
use miden_client::Deserializable;
#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;

#[cfg(feature = "wasm")]
use super::account_id::{AccountId, NetworkId};
use super::note_tag::NoteTag;
use crate::prelude::*;

#[cfg(feature = "wasm")]
use wasm_bindgen_futures::js_sys::Uint8Array;
#[cfg(feature = "wasm")]
use crate::utils::deserialize_from_uint8array;

/// Representation of a Miden address (account ID plus routing parameters).
#[bindings(inspectable)]
#[derive(Clone, Debug)]
pub struct Address(NativeAddress);

#[cfg(feature = "wasm")]
mod wasm_def {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    /// Specifies which procedures an account accepts, and by extension which notes it can consume.
    pub enum AddressInterface {
        BasicWallet = "BasicWallet",
    }
}

#[cfg(feature = "napi")]
mod napi_def {
    use napi_derive::napi;

    /// Specifies which procedures an account accepts, and by extension which notes it can consume.
    #[napi(string_enum)]
    pub enum AddressInterface {
        BasicWallet,
    }
}

#[cfg(feature = "wasm")]
pub use wasm_def::AddressInterface;

#[cfg(feature = "napi")]
pub use napi_def::AddressInterface;

// Shared methods (identical across wasm and napi)
#[bindings]
impl Address {
    /// Returns the account ID embedded in the address.
    #[bindings(getter)]
    pub fn account_id(&self) -> JsResult<AccountId> {
        match &self.0.id() {
            AddressId::AccountId(account_id_address) => Ok(account_id_address.into()),
            _other => Err(platform::error_from_string("Unsupported Account address type")),
        }
    }

    /// Converts the address into a note tag.
    #[bindings]
    pub fn to_note_tag(&self) -> NoteTag {
        self.0.to_note_tag().into()
    }

    /// Builds an address from an account ID and optional interface.
    #[bindings(factory)]
    pub fn from_account_id(
        account_id: &AccountId,
        interface: Option<String>,
    ) -> JsResult<Address> {
        let native_account_id: NativeAccountId = account_id.into();
        let native_address = match interface {
            None => NativeAddress::new(native_account_id),
            Some(interface) if &interface == "BasicWallet" => {
                let routing_params = RoutingParameters::new(NativeAddressInterface::BasicWallet);
                NativeAddress::new(native_account_id)
                    .with_routing_parameters(routing_params)
                    .map_err(|err| platform::error_with_context(err, "failed to set routing params"))?
            },
            Some(other_interface) => {
                return Err(platform::error_from_string(&format!(
                    "Failed to build address from account id, wrong interface value given: {other_interface}"
                )));
            },
        };

        Ok(Self(native_address))
    }

    /// Builds an address from a bech32-encoded string.
    #[bindings(factory, js_name = "fromBech32")]
    pub fn from_bech32(bech32: String) -> JsResult<Address> {
        let (_net_id, address) = NativeAddress::decode(&bech32).map_err(|err| {
            platform::error_with_context(err, "could not convert bech32 into an address")
        })?;
        Ok(Self(address))
    }
}

// wasm-specific methods
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Address {
    /// Deserializes a byte array into an `Address`.
    pub fn deserialize(bytes: &Uint8Array) -> JsResult<Address> {
        let native_address: NativeAddress = deserialize_from_uint8array(bytes)?;
        Ok(Self(native_address))
    }

    /// Returns the address interface.
    pub fn interface(&self) -> JsResult<AddressInterface> {
        match self.0.interface() {
            Some(interface) => interface.try_into().map_err(|e: String| platform::error_from_string(&e)),
            None => Err(platform::error_from_string("Address has no specified interface")),
        }
    }

    /// Encodes the address using the provided network prefix.
    #[wasm_bindgen(js_name = "toBech32")]
    pub fn to_bech32(&self, network_id: NetworkId) -> JsResult<String> {
        let net_id: NativeNetworkId = network_id.into();
        Ok(self.0.encode(net_id))
    }
}

// napi-specific methods
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl Address {
    /// Deserializes a byte array into an `Address`.
    #[napi(factory)]
    pub fn deserialize(bytes: Buffer) -> JsResult<Address> {
        let native_address = NativeAddress::read_from_bytes(&bytes)
            .map_err(|err| platform::error_with_context(err, "failed to deserialize Address"))?;
        Ok(Self(native_address))
    }

    /// Encodes the address using the provided network prefix.
    pub fn to_bech32(&self, network_id: &NetworkId) -> String {
        let net_id: NativeNetworkId = network_id.into();
        self.0.encode(net_id)
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

#[cfg(feature = "wasm")]
impl TryFrom<AddressInterface> for NativeAddressInterface {
    type Error = &'static str;
    fn try_from(value: AddressInterface) -> Result<Self, &'static str> {
        match value {
            AddressInterface::BasicWallet => Ok(NativeAddressInterface::BasicWallet),
            AddressInterface::__Invalid => Err("Non-valid address interface given"),
        }
    }
}

#[cfg(feature = "wasm")]
impl TryFrom<NativeAddressInterface> for AddressInterface {
    type Error = String;
    fn try_from(value: NativeAddressInterface) -> Result<Self, Self::Error> {
        match value {
            NativeAddressInterface::BasicWallet => Ok(AddressInterface::BasicWallet),
            _other => {
                Err("AddressInterface from miden-protocol crate was instantiated with an unsupported value"
                    .into())
            },
        }
    }
}

#[cfg(feature = "napi")]
impl From<AddressInterface> for NativeAddressInterface {
    fn from(value: AddressInterface) -> Self {
        match value {
            AddressInterface::BasicWallet => NativeAddressInterface::BasicWallet,
        }
    }
}

#[cfg(feature = "napi")]
impl TryFrom<NativeAddressInterface> for AddressInterface {
    type Error = String;
    fn try_from(value: NativeAddressInterface) -> std::result::Result<Self, Self::Error> {
        match value {
            NativeAddressInterface::BasicWallet => Ok(AddressInterface::BasicWallet),
            _other => Err(
                "AddressInterface from miden-protocol crate was instantiated with an unsupported value".to_string(),
            ),
        }
    }
}
