use miden_client::account::AccountId as NativeAccountId;
use miden_client::address::{
    Address as NativeAddress, AddressId, AddressInterface as NativeAddressInterface,
    NetworkId as NativeNetworkId, RoutingParameters,
};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use super::account_id::{AccountId, NetworkId};
use super::note_tag::NoteTag;
use crate::js_error_with_context;

#[wasm_bindgen(inspectable)]
#[derive(Clone, Debug)]
pub struct Address(NativeAddress);

#[wasm_bindgen]
/// Specifies which procedures an account accepts, and by extension which notes it can consume.
pub enum AddressInterface {
    BasicWallet = "BasicWallet",
}

#[wasm_bindgen]
impl Address {
    #[wasm_bindgen(js_name = "fromAccountId")]
    // Can't pass the proper AddressInterface enum here since wasm_bindgen does not derive the ref
    // trait for enum types. But we can still leave its definition since it gets exported as a
    // constant for the JS SDK.
    pub fn from_account_id(
        account_id: &AccountId,
        interface: Option<String>,
    ) -> Result<Self, JsValue> {
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
                return Err(JsValue::from_str(&format!(
                    "failed to build address from account id, wrong interface value given: {other_interface}"
                )));
            },
        };

        Ok(Self(native_address))
    }

    #[wasm_bindgen(js_name = fromBech32)]
    pub fn from_bech32(bech32: &str) -> Result<Self, JsValue> {
        let (_net_id, address) = NativeAddress::decode(bech32).map_err(|err| {
            js_error_with_context(err, "could not convert bech32 into an address")
        })?;
        Ok(Self(address))
    }

    pub fn interface(&self) -> Result<AddressInterface, JsValue> {
        match self.0.interface() {
            Some(interface) => interface.try_into(),
            None => Err(JsValue::from_str("address has no specified interface")),
        }
    }

    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> Result<AccountId, JsValue> {
        match &self.0.id() {
            AddressId::AccountId(account_id_address) => Ok(account_id_address.into()),
            _other => Err("Unsupported Account address type".into()),
        }
    }

    #[wasm_bindgen(js_name = "toNoteTag")]
    pub fn to_note_tag(&self) -> NoteTag {
        self.0.to_note_tag().into()
    }

    #[wasm_bindgen(js_name = "toBech32")]
    pub fn to_bech32(&self, network_id: NetworkId) -> Result<String, JsValue> {
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
            AddressInterface::__Invalid => Err("Non-valid address interface given"),
        }
    }
}

impl TryFrom<NativeAddressInterface> for AddressInterface {
    type Error = JsValue;
    fn try_from(value: NativeAddressInterface) -> Result<Self, Self::Error> {
        match value {
            NativeAddressInterface::BasicWallet => Ok(AddressInterface::BasicWallet),
            _other => {
                Err("AddressInterface from miden-objects crate was instantiated with an unsupported value"
                    .into())
            },
        }
    }
}
