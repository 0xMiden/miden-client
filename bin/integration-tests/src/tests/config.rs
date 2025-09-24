use std::env::temp_dir;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::rpc::{Endpoint, TonicRpcClient};
use miden_client::testing::common::{TestClient, TestClientKeyStore, create_test_store_path};
use miden_client::{DebugMode, Felt};
use miden_client_sqlite_store::SqliteStore;
use rand::Rng;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub rpc_endpoint: Endpoint,
    pub rpc_timeout_ms: u64,
    pub store_config: PathBuf,
    pub auth_path: PathBuf,
}

impl ClientConfig {
    pub fn new(rpc_endpoint: Endpoint, rpc_timeout_ms: u64) -> Self {
        Self {
            rpc_endpoint,
            rpc_timeout_ms,
            auth_path: create_test_auth_path(),
            store_config: create_test_store_path(),
        }
    }

    pub fn as_parts(&self) -> (Endpoint, u64, PathBuf, PathBuf) {
        (
            self.rpc_endpoint.clone(),
            self.rpc_timeout_ms,
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

    /// Creates a `TestClient` builder and keystore.
    ///
    /// Creates the client builder using the provided `ClientConfig`. The store uses a `SQLite`
    /// database at a temporary location determined by the store config.
    pub async fn into_client_builder(
        self,
    ) -> Result<(ClientBuilder<TestClientKeyStore>, TestClientKeyStore)> {
        let (rpc_endpoint, rpc_timeout, store_config, auth_path) = self.as_parts();

        let mut rng = rand::rng();
        let coin_seed: [u64; 4] = rng.random();

        let rng = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

        let store = {
            let sqlite_store = SqliteStore::new(store_config)
                .await
                .with_context(|| "failed to create SQLite store")?;
            Arc::new(sqlite_store)
        };

        let keystore = FilesystemKeyStore::new(auth_path.clone()).with_context(|| {
            format!("failed to create keystore at path: {}", auth_path.to_string_lossy())
        })?;

        let builder = ClientBuilder::new()
            .rpc(Arc::new(TonicRpcClient::new(&rpc_endpoint, rpc_timeout)))
            .rng(Box::new(rng))
            .store(store)
            .filesystem_keystore(auth_path.to_str().with_context(|| {
                format!("failed to convert auth path to string: {}", auth_path.to_string_lossy())
            })?)
            .in_debug_mode(DebugMode::Enabled)
            .tx_graceful_blocks(None);

        Ok((builder, keystore))
    }

    /// Creates a `TestClient`.
    ///
    /// Creates the client using the provided [`ClientConfig`]. The store uses a `SQLite` database
    /// at a temporary location determined by the store config. The client is synced to the
    /// current state before being returned.
    pub async fn into_client(self) -> Result<(TestClient, TestClientKeyStore)> {
        let (builder, keystore) = self.into_client_builder().await?;

        let mut client = builder.build().await.with_context(|| "failed to build test client")?;

        client.sync_state().await.with_context(|| "failed to sync client state")?;

        Ok((client, keystore))
    }
}

impl Default for ClientConfig {
    /// Creates a default client config.
    ///
    /// The RPC endpoint is read from the `TEST_MIDEN_RPC_ENDPOINT` environment variable, or
    /// defaults to `localhost` if the environment variable is not set.
    ///
    /// The timeout is set to 10 seconds.
    ///
    /// The store and auth paths are a temporary directory.
    fn default() -> Self {
        // Try to read from env first or default to localhost
        let endpoint = match std::env::var("TEST_MIDEN_RPC_ENDPOINT") {
            Ok(endpoint) => Endpoint::try_from(endpoint.as_str()).unwrap(),
            Err(_) => Endpoint::localhost(),
        };

        let timeout_ms = 10000;

        Self::new(endpoint, timeout_ms)
    }
}

pub(crate) fn create_test_auth_path() -> PathBuf {
    let auth_path = temp_dir().join(format!("keystore-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&auth_path).unwrap();
    auth_path
}
