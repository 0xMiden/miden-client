use std::env::temp_dir;
use std::path::PathBuf;

use uuid::Uuid;

use crate::rpc::Endpoint;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub rpc_endpoint: Endpoint,
    pub rpc_timeout: u64,
    pub store_config: PathBuf,
    pub auth_path: PathBuf,
}

impl ClientConfig {
    pub fn new(
        rpc_endpoint: Endpoint,
        rpc_timeout: u64,
        store_config: PathBuf,
        auth_path: PathBuf,
    ) -> Self {
        Self {
            rpc_endpoint,
            rpc_timeout,
            store_config,
            auth_path,
        }
    }

    pub fn into_parts(&self) -> (Endpoint, u64, PathBuf, PathBuf) {
        (
            self.rpc_endpoint.clone(),
            self.rpc_timeout,
            self.store_config.clone(),
            self.auth_path.clone(),
        )
    }

    #[allow(clippy::return_self_not_must_use)]
    pub fn with_rpc_endpoint(mut self, rpc_endpoint: Endpoint) -> Self {
        self.rpc_endpoint = rpc_endpoint;
        self
    }

    pub fn rpc_endpoint(&self) -> Endpoint {
        self.rpc_endpoint.clone()
    }
}

impl Default for ClientConfig {
    /// Creates a default client config.
    ///
    /// The RPC endpoint is read from the `MIDEN_RPC_ENDPOINT` environment variable, or defaults to
    /// `localhost` if the environment variable is not set.
    ///
    /// The timeout is set to 10 seconds.
    ///
    /// The store and auth paths are a temporary directory.
    fn default() -> Self {
        // Try to read from env first or default to localhost
        let endpoint = match std::env::var("MIDEN_RPC_ENDPOINT") {
            Ok(endpoint) => Endpoint::try_from(endpoint.as_str()).unwrap(),
            Err(_) => Endpoint::localhost(),
        };

        let timeout_ms = 10000;

        let auth_path = temp_dir().join(format!("keystore-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&auth_path).unwrap();

        Self::new(endpoint, timeout_ms, create_test_store_path(), auth_path)
    }
}

pub fn create_test_store_path() -> PathBuf {
    let mut temp_file = temp_dir();
    temp_file.push(format!("{}.sqlite3", Uuid::new_v4()));
    temp_file
}

pub fn create_test_auth_path() -> PathBuf {
    let auth_path = temp_dir().join(format!("keystore-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&auth_path).unwrap();
    auth_path
}
