use miden_client::rpc::Endpoint as NativeEndpoint;
use wasm_bindgen::prelude::*;

/// Represents a network endpoint for connecting to Miden nodes.
///
/// An endpoint consists of a protocol (http/https), host, and optional port.
/// Provides convenient constructors for common network configurations.
#[derive(Clone)]
#[wasm_bindgen]
pub struct Endpoint(NativeEndpoint);

#[wasm_bindgen]
impl Endpoint {
    /// Creates an endpoint from a URL string.
    ///
    /// @param url - The URL string (e.g., <https://localhost:57291>)
    /// @throws throws an error if the URL is invalid
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Result<Endpoint, JsValue> {
        NativeEndpoint::try_from(url)
            .map(Endpoint)
            .map_err(|err| JsValue::from_str(&err))
    }

    /// Returns the endpoint for the Miden testnet.
    #[wasm_bindgen]
    pub fn testnet() -> Endpoint {
        Endpoint(NativeEndpoint::testnet())
    }

    /// Returns the endpoint for the Miden devnet.
    #[wasm_bindgen]
    pub fn devnet() -> Endpoint {
        Endpoint(NativeEndpoint::devnet())
    }

    /// Returns the endpoint for a local Miden node.
    ///
    /// Uses <http://localhost:57291>
    #[wasm_bindgen]
    pub fn localhost() -> Endpoint {
        Endpoint(NativeEndpoint::localhost())
    }

    /// Returns the protocol of the endpoint.
    #[wasm_bindgen(getter)]
    pub fn protocol(&self) -> String {
        self.0.protocol().to_string()
    }

    /// Returns the host of the endpoint.
    #[wasm_bindgen(getter)]
    pub fn host(&self) -> String {
        self.0.host().to_string()
    }

    /// Returns the port of the endpoint.
    #[wasm_bindgen(getter)]
    pub fn port(&self) -> Option<u16> {
        self.0.port()
    }

    /// Returns the string representation of the endpoint.
    #[wasm_bindgen(js_name = "toString")]
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
