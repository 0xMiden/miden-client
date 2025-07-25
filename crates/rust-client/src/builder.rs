use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use std::boxed::Box;

use miden_objects::{
    Felt, MAX_TX_EXECUTION_CYCLES, MIN_TX_EXECUTION_CYCLES,
    crypto::rand::{FeltRng, RpoRandomCoin},
};
use miden_tx::{ExecutionOptions, auth::TransactionAuthenticator};
use rand::Rng;

#[cfg(feature = "tonic")]
use crate::rpc::{Endpoint, TonicRpcClient};
#[cfg(feature = "sqlite")]
use crate::store::sqlite_store::SqliteStore;
use crate::{
    Client, ClientError, DebugMode, keystore::FilesystemKeyStore, rpc::NodeRpcClient, store::Store,
};

// CONSTANTS
// ================================================================================================

/// The number of blocks that are considered old enough to discard pending transactions.
const TX_GRACEFUL_BLOCKS: u32 = 20;

// AUTHENTICATOR CONFIGURATION
// ================================================================================================

/// Represents the configuration for an authenticator.
///
/// This enum defers authenticator instantiation until the build phase. The builder can accept
/// either:
///
/// - A direct instance of an authenticator, or
/// - A keystore path as a string which is then used as an authenticator.
enum AuthenticatorConfig<AUTH> {
    Path(String),
    Instance(Arc<AUTH>),
}

// CLIENT BUILDER
// ================================================================================================

/// A builder for constructing a Miden client.
///
/// This builder allows you to configure the various components required by the client, such as the
/// RPC endpoint, store, RNG, and keystore. It is generic over the keystore type. By default, it
/// uses `FilesystemKeyStore<rand::rngs::StdRng>`.
pub struct ClientBuilder<AUTH> {
    /// An optional custom RPC client. If provided, this takes precedence over `rpc_endpoint`.
    rpc_api: Option<Arc<dyn NodeRpcClient + Send>>,
    /// An optional store provided by the user.
    store: Option<Arc<dyn Store>>,
    /// An optional RNG provided by the user.
    rng: Option<Box<dyn FeltRng>>,
    /// The store path to use when no store is directly provided via `store()`.
    #[cfg(feature = "sqlite")]
    store_path: String,
    /// The keystore configuration provided by the user.
    keystore: Option<AuthenticatorConfig<AUTH>>,
    /// A flag to enable debug mode.
    in_debug_mode: DebugMode,
    /// The number of blocks that are considered old enough to discard pending transactions. If
    /// `None`, there is no limit and transactions will be kept indefinitely.
    tx_graceful_blocks: Option<u32>,
    /// Maximum number of blocks the client can be behind the network for transactions and account
    /// proofs to be considered valid.
    max_block_number_delta: Option<u32>,
}

impl<AUTH> Default for ClientBuilder<AUTH> {
    fn default() -> Self {
        Self {
            rpc_api: None,
            store: None,
            rng: None,
            #[cfg(feature = "sqlite")]
            store_path: "store.sqlite3".to_string(),
            keystore: None,
            in_debug_mode: DebugMode::Disabled,
            tx_graceful_blocks: Some(TX_GRACEFUL_BLOCKS),
            max_block_number_delta: None,
        }
    }
}

impl<AUTH> ClientBuilder<AUTH>
where
    AUTH: TransactionAuthenticator + From<FilesystemKeyStore<rand::rngs::StdRng>> + 'static,
{
    /// Create a new `ClientBuilder` with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable debug mode.
    #[must_use]
    pub fn in_debug_mode(mut self, debug: DebugMode) -> Self {
        self.in_debug_mode = debug;
        self
    }

    /// Sets a custom RPC client directly.
    #[must_use]
    pub fn rpc(mut self, client: Arc<dyn NodeRpcClient + Send>) -> Self {
        self.rpc_api = Some(client);
        self
    }

    /// Sets the a tonic RPC client from the endpoint and optional timeout.
    #[cfg(feature = "tonic")]
    #[must_use]
    pub fn tonic_rpc_client(mut self, endpoint: &Endpoint, timeout_ms: Option<u64>) -> Self {
        self.rpc_api = Some(Arc::new(TonicRpcClient::new(endpoint, timeout_ms.unwrap_or(10_000))));
        self
    }

    /// Optionally set a custom store path.
    #[cfg(feature = "sqlite")]
    #[must_use]
    pub fn sqlite_store(mut self, path: &str) -> Self {
        self.store_path = path.to_string();
        self
    }

    /// Optionally provide a store directly.
    #[must_use]
    pub fn store(mut self, store: Arc<dyn Store>) -> Self {
        self.store = Some(store);
        self
    }

    /// Optionally provide a custom RNG.
    #[must_use]
    pub fn rng(mut self, rng: Box<dyn FeltRng>) -> Self {
        self.rng = Some(rng);
        self
    }

    /// Optionally provide a custom authenticator instance.
    #[must_use]
    pub fn authenticator(mut self, authenticator: Arc<AUTH>) -> Self {
        self.keystore = Some(AuthenticatorConfig::Instance(authenticator));
        self
    }

    /// Optionally set a maximum number of blocks that the client can be behind the network.
    /// By default, there's no maximum.
    #[must_use]
    pub fn max_block_number_delta(mut self, delta: u32) -> Self {
        self.max_block_number_delta = Some(delta);
        self
    }

    /// Optionally set a maximum number of blocks to wait for a transaction to be confirmed. If
    /// `None`, there is no limit and transactions will be kept indefinitely.
    /// By default, the maximum is set to `TX_GRACEFUL_BLOCKS`.
    #[must_use]
    pub fn tx_graceful_blocks(mut self, delta: Option<u32>) -> Self {
        self.tx_graceful_blocks = delta;
        self
    }

    /// **Required:** Provide the keystore path as a string.
    ///
    /// This stores the keystore path as a configuration option so that actual keystore
    /// initialization is deferred until `build()`. This avoids panicking during builder chaining.
    #[must_use]
    pub fn filesystem_keystore(mut self, keystore_path: &str) -> Self {
        self.keystore = Some(AuthenticatorConfig::Path(keystore_path.to_string()));
        self
    }

    /// Build and return the `Client`.
    ///
    /// # Errors
    ///
    /// - Returns an error if no RPC client or endpoint was provided.
    /// - Returns an error if the store cannot be instantiated.
    /// - Returns an error if the keystore is not specified or fails to initialize.
    #[allow(clippy::unused_async, unused_mut)]
    pub async fn build(mut self) -> Result<Client<AUTH>, ClientError> {
        // Determine the RPC client to use.
        let rpc_api: Arc<dyn NodeRpcClient + Send> = if let Some(client) = self.rpc_api {
            client
        } else {
            return Err(ClientError::ClientInitializationError(
                "RPC client or endpoint is required. Call `.rpc(...)` or `.tonic_rpc_client(...)` if `tonic` is enabled."
                    .into(),
            ));
        };

        #[cfg(feature = "sqlite")]
        if self.store.is_none() {
            let store = SqliteStore::new(self.store_path.into())
                .await
                .map_err(ClientError::StoreError)?;
            self.store = Some(Arc::new(store));
        }

        // If no store was provided, create a SQLite store from the given path.
        let arc_store: Arc<dyn Store> = if let Some(store) = self.store {
            store
        } else {
            return Err(ClientError::ClientInitializationError(
                "Store must be specified. Call `.store(...)` or `.sqlite_store(...)` with a store path if `sqlite` is enabled."
                    .into(),
            ));
        };

        // Use the provided RNG, or create a default one.
        let rng = if let Some(user_rng) = self.rng {
            user_rng
        } else {
            let mut seed_rng = rand::rng();
            let coin_seed: [u64; 4] = seed_rng.random();
            Box::new(RpoRandomCoin::new(coin_seed.map(Felt::new).into()))
        };

        // Initialize the authenticator.
        let authenticator = match self.keystore {
            Some(AuthenticatorConfig::Instance(authenticator)) => authenticator,
            Some(AuthenticatorConfig::Path(ref path)) => {
                let keystore = FilesystemKeyStore::new(path.into())
                    .map_err(|err| ClientError::ClientInitializationError(err.to_string()))?;
                Arc::new(AUTH::from(keystore))
            },
            None => {
                return Err(ClientError::ClientInitializationError(
                    "Keystore must be specified. Call `.keystore(...)` or `.filesystem_keystore(...)` with a keystore path."
                        .into(),
                ))
            }
        };

        Ok(Client::new(
            rpc_api,
            rng,
            arc_store,
            authenticator,
            ExecutionOptions::new(
                Some(MAX_TX_EXECUTION_CYCLES),
                MIN_TX_EXECUTION_CYCLES,
                false,
                self.in_debug_mode.into(),
            )
            .expect("Default executor's options should always be valid"),
            self.tx_graceful_blocks,
            self.max_block_number_delta,
        ))
    }
}
