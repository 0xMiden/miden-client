use alloc::string::String;
use core::fmt::Write;
use core::ops::{Deref, DerefMut};

use api_client_wrapper::{ApiClient, InnerClient};
use miden_objects::Word;
// (no additional tonic metadata imports needed here)

// WEB CLIENT
// ================================================================================================

#[cfg(all(feature = "web-tonic", target_arch = "wasm32"))]
pub(crate) mod api_client_wrapper {
    use alloc::string::String;

    use miden_objects::Word;
    use super::accept_header_value;
    use crate::rpc::RpcError;
    use crate::rpc::generated::rpc::api_client::ApiClient as ProtoClient;

    pub type WasmClient = tonic_web_wasm_client::Client;
    pub type InnerClient = ProtoClient<WasmClient>;
    #[derive(Clone)]
    pub struct ApiClient(pub(crate) InnerClient);

    impl ApiClient {
        /// Connects to the Miden node API using the provided URL and genesis commitment.
        ///
        /// The client is configured with required request metadata on each call.
        #[allow(clippy::unused_async)]
        pub async fn new_client(
            endpoint: String,
            _timeout_ms: u64,
            _genesis_commitment: Option<Word>,
        ) -> Result<ApiClient, RpcError> {
            let wasm_client = WasmClient::new(endpoint);
            Ok(ApiClient(ProtoClient::new(wasm_client)))
        }
    }
}

// CLIENT
// ================================================================================================

#[cfg(all(feature = "tonic", not(target_arch = "wasm32")))]
pub(crate) mod api_client_wrapper {
    use alloc::boxed::Box;
    use alloc::string::String;
    use core::time::Duration;

    use miden_objects::Word;
    use tonic::transport::Channel;

    use super::accept_header_value;
    use crate::rpc::RpcError;
    use crate::rpc::generated::rpc::api_client::ApiClient as ProtoClient;

    pub type InnerClient = ProtoClient<Channel>;
    #[derive(Clone)]
    pub struct ApiClient(pub(crate) InnerClient);

    impl ApiClient {
        /// Connects to the Miden node API using the provided URL, timeout and genesis commitment.
        ///
        /// The client is configured with required request metadata on each call.
        pub async fn new_client(
            endpoint: String,
            timeout_ms: u64,
            genesis_commitment: Option<Word>,
        ) -> Result<ApiClient, RpcError> {
            // Setup connection channel.
            let endpoint = tonic::transport::Endpoint::try_from(endpoint)
                .map_err(|err| RpcError::ConnectionError(Box::new(err)))?
                .timeout(Duration::from_millis(timeout_ms));
            let channel = endpoint
                .tls_config(tonic::transport::ClientTlsConfig::new().with_native_roots())
                .map_err(|err| RpcError::ConnectionError(Box::new(err)))?
                .connect()
                .await
                .map_err(|err| RpcError::ConnectionError(Box::new(err)))?;

            // Return the connected client.
            let client = ProtoClient::new(channel).max_decoding_message_size(usize::MAX);
            Ok(ApiClient(client))
        }
    }
}

impl Deref for ApiClient {
    type Target = InnerClient;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ApiClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// HEADER VALUE
// ================================================================================================

/// Returns the value to be used for the HTTP `accept` header expected by Miden RPC.
/// The value sets the Miden API version and optionally includes the genesis commitment.
pub(crate) fn accept_header_value(genesis_digest: Option<Word>) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let mut accept_value = format!("application/vnd.miden; version={version}");
    if let Some(commitment) = genesis_digest {
        write!(accept_value, "; genesis={}", commitment.to_hex())
            .expect("valid hex representation of Word");
    }
    accept_value
}
