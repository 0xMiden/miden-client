use alloc::string::String;
use core::ops::{Deref, DerefMut};

use api_client_wrapper::{ApiClient, InnerClient};
use miden_objects::Word;
use tonic::{
    metadata::{AsciiMetadataValue, errors::InvalidMetadataValue},
    service::Interceptor,
};

// WEB CLIENT
// ================================================================================================

#[cfg(all(not(target_arch = "wasm32"), feature = "web-tonic"))]
compile_error!("The `web-tonic` feature is only supported when targeting wasm32.");

#[cfg(feature = "web-tonic")]
pub(crate) mod api_client_wrapper {
    use alloc::string::String;

    use miden_objects::Word;
    use tonic::service::interceptor::InterceptedService;

    use super::{MetadataInterceptor, header_interceptor};
    use crate::rpc::{RpcError, generated::rpc::api_client::ApiClient as ProtoClient};

    pub type WasmClient = tonic_web_wasm_client::Client;
    pub type InnerClient = ProtoClient<InterceptedService<WasmClient, MetadataInterceptor>>;
    #[derive(Clone)]
    pub struct ApiClient(pub(crate) InnerClient);

    impl ApiClient {
        /// Connects to the Miden node API using the provided URL and genesis commitment.
        ///
        /// The client is configured with an interceptor that sets all requisite request metadata.
        #[allow(clippy::unused_async)]
        pub async fn new_client(
            endpoint: String,
            _timeout_ms: u64,
            genesis_commitment: Option<Word>,
        ) -> Result<ApiClient, RpcError> {
            let wasm_client = WasmClient::new(endpoint);
            let interceptor = header_interceptor(genesis_commitment);
            Ok(ApiClient(ProtoClient::with_interceptor(wasm_client, interceptor)))
        }
    }
}

// CLIENT
// ================================================================================================

#[cfg(feature = "tonic")]
pub(crate) mod api_client_wrapper {
    use alloc::{boxed::Box, string::String};
    use core::time::Duration;

    use miden_objects::Word;
    use tonic::{service::interceptor::InterceptedService, transport::Channel};

    use super::{MetadataInterceptor, header_interceptor};
    use crate::rpc::{RpcError, generated::rpc::api_client::ApiClient as ProtoClient};

    pub type InnerClient = ProtoClient<InterceptedService<Channel, MetadataInterceptor>>;
    #[derive(Clone)]
    pub struct ApiClient(pub(crate) InnerClient);

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
            let interceptor = header_interceptor(genesis_commitment);

            // Return the connected client.
            Ok(ApiClient(ProtoClient::with_interceptor(channel, interceptor)))
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
fn header_interceptor(genesis_digest: Option<Word>) -> MetadataInterceptor {
    let version = env!("CARGO_PKG_VERSION");
    let mut accept_value = format!("application/vnd.miden; version={version}");
    if let Some(commitment) = genesis_digest {
        accept_value.push_str(&format!("; genesis={}", commitment.to_hex()));
    }

    MetadataInterceptor::default()
        .with_metadata("accept", accept_value)
        .expect("valid key/value metadata for interceptor")
}
