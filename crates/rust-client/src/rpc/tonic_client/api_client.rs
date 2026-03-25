use alloc::string::String;
use core::fmt::Write;
use core::ops::{Deref, DerefMut};

use api_client_wrapper::{ApiClient, InnerClient};
use miden_protocol::Word;
use tonic::metadata::AsciiMetadataValue;
use tonic::metadata::errors::InvalidMetadataValue;
use tonic::service::Interceptor;

// WEB CLIENT
// ================================================================================================

#[cfg(target_arch = "wasm32")]
pub(crate) mod api_client_wrapper {
    use alloc::string::String;

    use miden_protocol::Word;
    use tonic::service::interceptor::InterceptedService;

    use super::{MetadataInterceptor, accept_header_interceptor};
    use crate::rpc::RpcError;
    use crate::rpc::generated::rpc::api_client::ApiClient as ProtoClient;

    pub type WasmClient = tonic_web_wasm_client::Client;
    pub type InnerClient = ProtoClient<InterceptedService<WasmClient, MetadataInterceptor>>;
    #[derive(Clone)]
    pub struct ApiClient {
        pub(crate) client: InnerClient,
        wasm_client: WasmClient,
    }

    impl ApiClient {
        /// Connects to the Miden node API using the provided URL and genesis commitment.
        ///
        /// The client is configured with an interceptor that sets all requisite request metadata.
        // Kept async for API parity with the native client; in WASM this is synchronous.
        #[allow(clippy::unused_async)]
        pub async fn new_client(
            endpoint: String,
            _timeout_ms: u64,
            genesis_commitment: Option<Word>,
        ) -> Result<ApiClient, RpcError> {
            let wasm_client = WasmClient::new(endpoint);
            let interceptor = accept_header_interceptor(genesis_commitment);
            let client = ProtoClient::with_interceptor(wasm_client.clone(), interceptor);
            Ok(ApiClient { client, wasm_client })
        }

        /// Connects to the Miden node API without injecting an Accept header.
        // Kept async for API parity with the native client; in WASM this is synchronous.
        #[allow(clippy::unused_async)]
        pub async fn new_client_without_accept_header(
            endpoint: String,
            _timeout_ms: u64,
        ) -> Result<ApiClient, RpcError> {
            let wasm_client = WasmClient::new(endpoint);
            let interceptor = MetadataInterceptor::default();
            let client = ProtoClient::with_interceptor(wasm_client.clone(), interceptor);
            Ok(ApiClient { client, wasm_client })
        }

        /// Returns a new `ApiClient` with an updated genesis commitment.
        /// This creates a new client that shares the same underlying channel.
        pub fn set_genesis_commitment(&mut self, genesis_commitment: Word) -> &mut Self {
            let interceptor = accept_header_interceptor(Some(genesis_commitment));
            self.client = ProtoClient::with_interceptor(self.wasm_client.clone(), interceptor);
            self
        }
    }
}

// CLIENT
// ================================================================================================

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod api_client_wrapper {
    use alloc::boxed::Box;
    use alloc::string::String;
    use core::time::Duration;

    use miden_protocol::Word;
    use tonic::service::interceptor::InterceptedService;
    use tonic::transport::Channel;

    use super::{MetadataInterceptor, accept_header_interceptor};
    use crate::rpc::RpcError;
    use crate::rpc::generated::rpc::api_client::ApiClient as ProtoClient;

    pub type InnerClient = ProtoClient<InterceptedService<Channel, MetadataInterceptor>>;
    #[derive(Clone)]
    pub struct ApiClient {
        pub(crate) client: InnerClient,
        channel: Channel,
    }

    impl ApiClient {
        /// Connects to the Miden node API using the provided URL, timeout and genesis commitment.
        ///
        /// The client is configured with an interceptor that sets all requisite request metadata.
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

            // Set up the accept metadata interceptor.
            let interceptor = accept_header_interceptor(genesis_commitment);

            // Return the connected client.
            let client = ProtoClient::with_interceptor(channel.clone(), interceptor);
            Ok(ApiClient { client, channel })
        }

        /// Connects to the Miden node API without injecting an Accept header.
        pub async fn new_client_without_accept_header(
            endpoint: String,
            timeout_ms: u64,
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

            let interceptor = MetadataInterceptor::default();
            let client = ProtoClient::with_interceptor(channel.clone(), interceptor);
            Ok(ApiClient { client, channel })
        }

        /// Returns a new `ApiClient` with an updated genesis commitment.
        /// This creates a new client that shares the same underlying channel.
        pub fn set_genesis_commitment(&mut self, genesis_commitment: Word) -> &mut Self {
            let interceptor = accept_header_interceptor(Some(genesis_commitment));
            self.client = ProtoClient::with_interceptor(self.channel.clone(), interceptor);
            self
        }
    }
}

impl Deref for ApiClient {
    type Target = InnerClient;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for ApiClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

// INTERCEPTOR
// ================================================================================================

/// Interceptor designed to inject required metadata into all [`ApiClient`] requests.
#[derive(Default, Clone)]
pub struct MetadataInterceptor {
    metadata: alloc::collections::BTreeMap<&'static str, AsciiMetadataValue>,
}

impl MetadataInterceptor {
    /// Adds or overwrites metadata to the interceptor.
    pub fn with_metadata(
        mut self,
        key: &'static str,
        value: String,
    ) -> Result<Self, InvalidMetadataValue> {
        self.metadata.insert(key, AsciiMetadataValue::try_from(value)?);
        Ok(self)
    }
}

impl Interceptor for MetadataInterceptor {
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let mut request = request;
        for (key, value) in &self.metadata {
            request.metadata_mut().insert(*key, value.clone());
        }
        Ok(request)
    }
}

/// Returns the HTTP header [`MetadataInterceptor`] that is expected by Miden RPC.
/// The interceptor sets the `accept` header to the Miden API version and optionally includes the
/// genesis commitment.
fn accept_header_interceptor(genesis_digest: Option<Word>) -> MetadataInterceptor {
    let version = env!("CARGO_PKG_VERSION");
    let mut accept_value = format!("application/vnd.miden; version={version}");
    if let Some(commitment) = genesis_digest {
        write!(accept_value, "; genesis={}", commitment.to_hex())
            .expect("valid hex representation of Word");
    }

    MetadataInterceptor::default()
        .with_metadata("accept", accept_value)
        .expect("valid key/value metadata for interceptor")
}
