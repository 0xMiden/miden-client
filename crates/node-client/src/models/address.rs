use miden_client::address::{
    Address as NativeAddress,
    AddressId,
    AddressInterface as NativeAddressInterface,
    RoutingParameters,
};
use miden_client::utils::Deserializable;
use napi::bindgen_prelude::*;

use super::account_id::AccountId;
use super::napi_wrap;

napi_wrap!(clone Address wraps NativeAddress);

#[napi]
impl Address {
    /// Deserializes an Address from bytes.
    #[napi]
    pub fn deserialize(bytes: Buffer) -> Result<Address> {
        let native = NativeAddress::read_from_bytes(&bytes).map_err(|err| {
            napi::Error::from_reason(format!("Failed to deserialize Address: {err}"))
        })?;
        Ok(Address(native))
    }

    /// Builds an address from an account ID and optional interface.
    #[napi(js_name = "fromAccountId")]
    pub fn from_account_id(account_id: &AccountId, interface: Option<String>) -> Result<Address> {
        let native_address = match interface {
            None => NativeAddress::new(account_id.0),
            Some(ref iface) if iface == "BasicWallet" => {
                let routing_params = RoutingParameters::new(NativeAddressInterface::BasicWallet);
                NativeAddress::new(account_id.0)
                    .with_routing_parameters(routing_params)
                    .map_err(|err| {
                        napi::Error::from_reason(format!("failed to set routing params: {err}"))
                    })?
            },
            Some(other) => {
                return Err(napi::Error::from_reason(format!("Unknown interface: {other}")));
            },
        };
        Ok(Address(native_address))
    }

    /// Builds an address from a bech32-encoded string.
    #[napi(js_name = "fromBech32")]
    pub fn from_bech32(bech32: String) -> Result<Address> {
        let (_, address) = NativeAddress::decode(&bech32).map_err(|err| {
            napi::Error::from_reason(format!("could not convert bech32 into an address: {err}"))
        })?;
        Ok(Address(address))
    }

    /// Returns the account ID embedded in the address.
    #[napi(js_name = "accountId")]
    pub fn account_id(&self) -> Result<AccountId> {
        match self.0.id() {
            AddressId::AccountId(account_id) => Ok(AccountId(account_id)),
            _ => Err(napi::Error::from_reason("Unsupported Account address type")),
        }
    }
}
