use js_export_macro::js_export;
use miden_client::rpc::Endpoint as NativeEndpoint;

use crate::platform::{JsErr, from_str_err};

/// The `Endpoint` struct represents a network endpoint, consisting of a protocol, a host, and a
/// port.
///
/// This struct is used to define the address of a Miden node that the client will connect to.
#[derive(Clone)]
#[js_export]
pub struct Endpoint(NativeEndpoint);

#[js_export]
impl Endpoint {
    /// Creates an endpoint from a URL string.
    ///
    /// @param url - The URL string (e.g., <https://localhost:57291>)
    /// @throws throws an error if the URL is invalid
    #[js_export(constructor)]
    pub fn new(url: String) -> Result<Endpoint, JsErr> {
        NativeEndpoint::try_from(url.as_str())
            .map(Endpoint)
            .map_err(|err| from_str_err(&err))
    }

    /// Returns the endpoint for the Miden testnet.
    pub fn testnet() -> Endpoint {
        Endpoint(NativeEndpoint::testnet())
    }

    /// Returns the endpoint for the Miden devnet.
    pub fn devnet() -> Endpoint {
        Endpoint(NativeEndpoint::devnet())
    }

    /// Returns the endpoint for a local Miden node.
    ///
    /// Uses <http://localhost:57291>
    pub fn localhost() -> Endpoint {
        Endpoint(NativeEndpoint::localhost())
    }

    /// Returns the protocol of the endpoint.
    #[js_export(getter)]
    pub fn protocol(&self) -> String {
        self.0.protocol().to_string()
    }

    /// Returns the host of the endpoint.
    #[js_export(getter)]
    pub fn host(&self) -> String {
        self.0.host().to_string()
    }

    /// Returns the port of the endpoint.
    #[js_export(getter)]
    pub fn port(&self) -> Option<u16> {
        self.0.port()
    }

    /// Returns the string representation of the endpoint.
    #[js_export(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeEndpoint> for Endpoint {
    fn from(native_endpoint: NativeEndpoint) -> Self {
        Endpoint(native_endpoint)
    }
}

impl From<Endpoint> for NativeEndpoint {
    fn from(endpoint: Endpoint) -> Self {
        endpoint.0
    }
}

impl From<&Endpoint> for NativeEndpoint {
    fn from(endpoint: &Endpoint) -> Self {
        endpoint.0.clone()
    }
}

impl_napi_from_value!(Endpoint);
