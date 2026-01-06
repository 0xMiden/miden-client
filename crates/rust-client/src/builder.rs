use alloc::string::{String, ToString};
use alloc::sync::Arc;
use std::boxed::Box;

use miden_protocol::crypto::rand::{FeltRng, RpoRandomCoin};
use miden_protocol::{Felt, MAX_TX_EXECUTION_CYCLES, MIN_TX_EXECUTION_CYCLES};
use miden_tx::ExecutionOptions;
use miden_tx::auth::TransactionAuthenticator;
use rand::Rng;

use crate::keystore::FilesystemKeyStore;
use crate::note_transport::NoteTransportClient;
use crate::rpc::NodeRpcClient;
use crate::store::{Store, StoreError};
use crate::transaction::TransactionProver;
use crate::{Client, ClientError, DebugMode};

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

// STORE BUILDER
// ================================================================================================

/// Allows the [`ClientBuilder`] to accept either an already built store instance or a factory for
/// deferring the store instantiation.
pub enum StoreBuilder {
    Store(Arc<dyn Store>),
    Factory(Box<dyn StoreFactory>),
}

/// Trait for building a store instance.
#[async_trait::async_trait]
pub trait StoreFactory {
    /// Returns a new store instance.
    async fn build(&self) -> Result<Arc<dyn Store>, StoreError>;
}

// CLIENT BUILDER
// ================================================================================================

/// A builder for constructing a Miden client.
///
/// This builder allows you to configure the various components required by the client, such as the
/// RPC endpoint, store, RNG, and keystore. It is generic over the keystore type. By default, it
/// uses [`FilesystemKeyStore`].
pub struct ClientBuilder<AUTH> {
    /// An optional custom RPC client. If provided, this takes precedence over `rpc_endpoint`.
    rpc_api: Option<Arc<dyn NodeRpcClient>>,
    /// An optional store provided by the user.
    pub store: Option<StoreBuilder>,
    /// An optional RNG provided by the user.
    rng: Option<Box<dyn FeltRng>>,
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
    /// An optional custom note transport client.
    note_transport_api: Option<Arc<dyn NoteTransportClient>>,
    /// An optional custom transaction prover.
    tx_prover: Option<Arc<dyn TransactionProver + Send + Sync>>,
}

impl<AUTH> Default for ClientBuilder<AUTH> {
    fn default() -> Self {
        Self {
            rpc_api: None,
            store: None,
            rng: None,
            keystore: None,
            in_debug_mode: DebugMode::Disabled,
            tx_graceful_blocks: Some(TX_GRACEFUL_BLOCKS),
            max_block_number_delta: None,
            note_transport_api: None,
            tx_prover: None,
        }
    }
}

impl<AUTH> ClientBuilder<AUTH>
where
    AUTH: BuilderAuthenticator,
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
    pub fn rpc(mut self, client: Arc<dyn NodeRpcClient>) -> Self {
        self.rpc_api = Some(client);
        self
    }

    /// Sets a gRPC client from the endpoint and optional timeout.
    #[must_use]
    #[cfg(feature = "tonic")]
    pub fn grpc_client(mut self, endpoint: &crate::rpc::Endpoint, timeout_ms: Option<u64>) -> Self {
        self.rpc_api =
            Some(Arc::new(crate::rpc::GrpcClient::new(endpoint, timeout_ms.unwrap_or(10_000))));
        self
    }

    /// Provide a store to be used by the client.
    #[must_use]
    pub fn store(mut self, store: Arc<dyn Store>) -> Self {
        self.store = Some(StoreBuilder::Store(store));
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

    /// Sets a custom note transport client directly.
    #[must_use]
    pub fn note_transport(mut self, client: Arc<dyn NoteTransportClient>) -> Self {
        self.note_transport_api = Some(client);
        self
    }

    /// Sets a custom transaction prover.
    #[must_use]
    pub fn prover(mut self, prover: Arc<dyn TransactionProver + Send + Sync>) -> Self {
        self.tx_prover = Some(prover);
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
        let rpc_api: Arc<dyn NodeRpcClient> = if let Some(client) = self.rpc_api {
            client
        } else {
            return Err(ClientError::ClientInitializationError(
                "RPC client or endpoint is required. Call `.rpc(...)` or `.tonic_rpc_client(...)`."
                    .into(),
            ));
        };

        // Ensure a store was provided.
        let store = if let Some(store_builder) = self.store {
            match store_builder {
                StoreBuilder::Store(store) => store,
                StoreBuilder::Factory(factory) => factory.build().await?,
            }
        } else {
            return Err(ClientError::ClientInitializationError(
                "Store must be specified. Call `.store(...)`.".into(),
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
            Some(AuthenticatorConfig::Instance(authenticator)) => Some(authenticator),
            Some(AuthenticatorConfig::Path(ref path)) => {
                let keystore = FilesystemKeyStore::new(path.into())
                    .map_err(|err| ClientError::ClientInitializationError(err.to_string()))?;
                Some(Arc::new(AUTH::from(keystore)))
            },
            None => None,
        };

        Client::new(
            rpc_api,
            rng,
            store,
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
            self.note_transport_api,
            self.tx_prover,
        )
        .await
    }
}

// AUTH TRAIT MARKER
// ================================================================================================

/// Marker trait to capture the bounds the builder requires for the authenticator type
/// parameter
pub trait BuilderAuthenticator:
    TransactionAuthenticator + From<FilesystemKeyStore> + 'static
{
}
impl<T> BuilderAuthenticator for T where
    T: TransactionAuthenticator + From<FilesystemKeyStore> + 'static
{
}
