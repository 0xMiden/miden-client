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

#[derive(Debug)]
pub struct AddressInterface(NativeAddressInterface);

#[wasm_bindgen]
impl AddressInterface {
    #[wasm_bindgen(constructor)]
    pub fn new(interface: u16) -> Self {
        AddressInterface(
            // FIXME: Error handling
            interface.try_into().unwrap(),
        )
    }
    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        match self.0 {
            NativeAddressInterface::Unspecified => format!("Unspecified"),
            NativeAddressInterface::BasicWallet => format!("BasicWallet"),
        }
    }
}

#[wasm_bindgen]
impl Address {
    #[wasm_bindgen(constructor)]
    pub fn new(account_id: AccountId, interface: AddressInterface) -> Self {
        let address = NativeAccountIdAddress::new(account_id.into(), interface.0);
        Address(NativeAddress::AccountId(address))
    }
    pub fn interface(&self) -> AddressInterface {
        AddressInterface(self.0.interface())
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
