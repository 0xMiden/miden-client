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
#[wasm_bindgen(inspectable)]
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
}

#[wasm_bindgen]
impl Address {
    #[wasm_bindgen(constructor)]
    pub fn new(account_id: AccountId, tag_len: u8, interface: AddressInterface) -> Self {
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
}
