use std::env::temp_dir;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::grpc_support::{DEVNET_PROVER_ENDPOINT, TESTNET_PROVER_ENDPOINT};
use miden_client::rpc::{Endpoint, GrpcClient};
use miden_client::testing::common::{FilesystemKeyStore, TestClient, create_test_store_path};
use miden_client::{DebugMode, Felt, RemoteTransactionProver};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use rand::Rng;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub rpc_endpoint: Endpoint,
    pub rpc_timeout_ms: u64,
    pub store_config: PathBuf,
    pub auth_path: PathBuf,
    /// Optional remote prover endpoint. If set, the client will use a remote prover instead of the
    /// default local prover.
    pub prover_endpoint: Option<String>,
}

impl ClientConfig {
    pub fn new(rpc_endpoint: Endpoint, rpc_timeout_ms: u64) -> Self {
        Self {
            rpc_endpoint,
            rpc_timeout_ms,
            auth_path: create_test_auth_path(),
            store_config: create_test_store_path(),
            prover_endpoint: None,
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
    pub fn with_prover_endpoint(mut self, prover_endpoint: Option<String>) -> Self {
        self.prover_endpoint = prover_endpoint;
        self
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
    ) -> Result<(ClientBuilder<FilesystemKeyStore>, FilesystemKeyStore)> {
        let (rpc_endpoint, rpc_timeout, store_config, auth_path) = self.as_parts();

        let mut rng = rand::rng();
        let coin_seed: [u64; 4] = rng.random();

        let rng = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

        let keystore = FilesystemKeyStore::new(auth_path.clone()).with_context(|| {
            format!("failed to create keystore at path: {}", auth_path.to_string_lossy())
        })?;

        let rpc_client = Arc::new(GrpcClient::new(&rpc_endpoint, rpc_timeout));

        let mut builder = ClientBuilder::new()
            .rpc(rpc_client)
            .rng(Box::new(rng))
            .sqlite_store(store_config)
            .authenticator(Arc::new(keystore.clone()))
            .in_debug_mode(DebugMode::Disabled)
            .tx_discard_delta(None);

        if let Some(prover_url) = &self.prover_endpoint {
            builder = builder.prover(Arc::new(RemoteTransactionProver::new(prover_url)));
        }

        Ok((builder, keystore))
    }

    /// Creates a `TestClient`.
    ///
    /// Creates the client using the provided [`ClientConfig`]. The store uses a `SQLite` database
    /// at a temporary location determined by the store config. The client is synced to the
    /// current state before being returned.
    pub async fn into_client(self) -> Result<(TestClient, FilesystemKeyStore)> {
        let (builder, keystore) = self.into_client_builder().await?;

        let mut client = builder.build().await.with_context(|| "failed to build test client")?;

        client.sync_state().await.with_context(|| "failed to sync client state")?;

        Ok((client, keystore))
    }
}

impl Default for ClientConfig {
    /// Creates a default client config.
    ///
    /// The network is read from the `TEST_MIDEN_NETWORK` environment variable, or
    /// defaults to `localhost` if the environment variable is not set.
    /// Accepted values: "devnet", "testnet", "localhost", or a custom RPC endpoint string.
    ///
    /// The remote prover is read from the `TEST_MIDEN_PROVER_URL` environment variable.
    /// Accepted values: "devnet", "testnet", or a custom prover endpoint URL.
    /// If unset, the local prover is used.
    ///
    /// The timeout is set to 10 seconds.
    ///
    /// The store and auth paths are a temporary directory.
    fn default() -> Self {
        // Use TEST_MIDEN_NETWORK to determine the endpoint.
        // Accepts "devnet", "testnet", "localhost", or a custom RPC endpoint string.
        let network =
            std::env::var("TEST_MIDEN_NETWORK").unwrap_or_else(|_| "localhost".to_string());
        let network_lower = network.to_lowercase();
        let endpoint = if network_lower == "devnet" {
            Endpoint::devnet()
        } else if network_lower == "testnet" {
            Endpoint::testnet()
        } else if network_lower == "localhost" {
            Endpoint::localhost()
        } else {
            Endpoint::try_from(network_lower.as_str()).unwrap()
        };

        // Use TEST_MIDEN_PROVER_URL to optionally configure a remote prover.
        // Accepts "devnet", "testnet", or a custom prover endpoint URL.
        let prover_endpoint = std::env::var("TEST_MIDEN_PROVER_URL").ok().map(|url| {
            match url.to_lowercase().as_str() {
                "devnet" => DEVNET_PROVER_ENDPOINT.to_string(),
                "testnet" => TESTNET_PROVER_ENDPOINT.to_string(),
                _ => url,
            }
        });

        let timeout_ms = 10000;

        Self::new(endpoint, timeout_ms).with_prover_endpoint(prover_endpoint)
    }
}

pub(crate) fn create_test_auth_path() -> PathBuf {
    let auth_path = temp_dir().join(format!("keystore-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&auth_path).unwrap();
    auth_path
}
