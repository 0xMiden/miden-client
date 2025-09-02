use miden_objects::address::{
    AccountIdAddress as NativeAccountIdAddress, Address as NativeAddress,
    AddressInterface as NativeAddressInterface,
};
use wasm_bindgen::prelude::wasm_bindgen;

use super::{
    account_id::{AccountId, NetworkId},
    note_tag::NoteTag,
};

#[wasm_bindgen(inspectable)]
#[derive(Debug)]
pub struct Address(NativeAddress);

#[wasm_bindgen]
/// Specifies which procedures an account accepts, and by extension which notes it can consume.
pub enum AddressInterface {
    Unspecified = "Unspecified",
    BasicWallet = "BasicWallet",
}

#[wasm_bindgen]
impl Address {
    #[wasm_bindgen(constructor)]
    pub fn new(account_id: AccountId, interface: AddressInterface) -> Self {
        // FIXME: Handle error
        let address = NativeAccountIdAddress::new(account_id.into(), interface.try_into().unwrap());
        Address(NativeAddress::AccountId(address))
    }
    pub fn interface(&self) -> AddressInterface {
        // FIXME: Handle error
        self.0.interface().try_into().unwrap()
    }

    #[wasm_bindgen(js_name = toNoteTag)]
    pub fn to_note_tag(&self) -> NoteTag {
        self.0.to_note_tag().into()
    }

    // FIXME: Handle errors
    pub fn to_bech32(&self, network_id: NetworkId) -> String {
        let net_id = network_id.try_into().unwrap();
        self.0.to_bech32(net_id)
    }

    // FIXME: Handle errors
    pub fn from_bech32(bech32: &str) -> Self {
        let (_net_id, address) = NativeAddress::from_bech32(bech32).unwrap();
        Self(address)
    }
}

impl TryFrom<AddressInterface> for NativeAddressInterface {
    type Error = &'static str;
    fn try_from(value: AddressInterface) -> Result<Self, &'static str> {
        match value {
            AddressInterface::BasicWallet => Ok(NativeAddressInterface::BasicWallet),
            AddressInterface::Unspecified => Ok(NativeAddressInterface::Unspecified),
            AddressInterface::__Invalid => Err("Non-valid address interface given"),
        }
    }
}

impl TryFrom<NativeAddressInterface> for AddressInterface {
    type Error = &'static str;
    fn try_from(value: NativeAddressInterface) -> Result<Self, Self::Error> {
        match value {
            NativeAddressInterface::BasicWallet => Ok(AddressInterface::BasicWallet),
            NativeAddressInterface::Unspecified => Ok(AddressInterface::Unspecified),
            _other => {
                Err("AddressInterface from miden-objects crate was instanced with a wrong value")
            },
        }
    }
}
