use alloc::string::String;
use tonic::{
    metadata::{AsciiMetadataValue, errors::InvalidMetadataValue},
    service::Interceptor,
};

#[cfg(all(not(target_arch = "wasm32"), feature = "web-tonic"))]
compile_error!("The `web-tonic` feature is only supported when targeting wasm32.");

#[cfg(feature = "web-tonic")]
pub(crate) mod api_client_wrapper {
    use alloc::string::String;

    use crate::rpc::RpcError;

    pub type ApiClient =
        crate::rpc::generated::rpc::api_client::ApiClient<tonic_web_wasm_client::Client>;

    impl ApiClient {
        #[allow(clippy::unused_async)]
        pub async fn new_client(endpoint: String, _timeout_ms: u64) -> Result<ApiClient, RpcError> {
            let wasm_client = tonic_web_wasm_client::Client::new(endpoint);
            Ok(ApiClient::new(wasm_client))
        }
    }
}

#[cfg(feature = "tonic")]
pub(crate) mod api_client_wrapper {
    use super::MetadataInterceptor;
    use crate::rpc::{RpcError, generated::rpc::api_client::ApiClient as ProtoClient};
    use alloc::{boxed::Box, string::String};
    use core::{
        ops::{Deref, DerefMut},
        time::Duration,
    };
    use tonic::{service::interceptor::InterceptedService, transport::Channel};

    type InnerClient = ProtoClient<InterceptedService<Channel, MetadataInterceptor>>;
    #[derive(Clone)]
    pub struct ApiClient(InnerClient);

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

    impl ApiClient {
        /// Connects to the Miden node API using the provided URL and timeout.
        ///
        /// The client is configured with an interceptor that sets all requisite request metadata.
        pub async fn new_client(endpoint: String, timeout_ms: u64) -> Result<ApiClient, RpcError> {
            // Setup connection channel.
            let endpoint = tonic::transport::Endpoint::try_from(endpoint)
                .map_err(|err| RpcError::ConnectionError(Box::new(err)))?
                .timeout(Duration::from_millis(timeout_ms));
            let channel = endpoint
                .connect()
                .await
                .map_err(|err| RpcError::ConnectionError(Box::new(err)))?;

            // Set up the accept metadata interceptor.
            let version = env!("CARGO_PKG_VERSION");
            let accept_value = format!("application/vnd.miden.{version}+grpc");
            let interceptor = MetadataInterceptor::default()
                .with_metadata("accept", accept_value)
                .expect("valid key/value metadata for interceptor");

            // Return the connected client.
            Ok(ApiClient(ProtoClient::with_interceptor(channel, interceptor)))
        }
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
