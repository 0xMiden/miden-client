#![recursion_limit = "256"]

use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    net::SocketAddr,
    num::NonZeroUsize,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ::rand::{Rng, random};
use anyhow::{Context, Result};
use miden_lib::{AuthScheme, account::faucets::create_basic_fungible_faucet, utils::Serializable};
use miden_node_block_producer::BlockProducer;
use miden_node_ntx_builder::NetworkTransactionBuilder;
use miden_node_rpc::Rpc;
use miden_node_store::{GenesisState, Store};
use miden_node_utils::{crypto::get_rpo_random_coin, grpc::UrlExt};
use miden_objects::{
    Felt, ONE,
    account::{AccountFile, AccountIdAnchor, AuthSecretKey},
    asset::TokenSymbol,
    crypto::dsa::rpo_falcon512::SecretKey,
};
use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};
use tokio::{
    net::TcpListener,
    task::{Id, JoinSet},
};
use url::Url;

// CONSTANTS
// --------------------------------------------------------------------------------------------

pub const DEFAULT_BLOCK_INTERVAL: u64 = 5000;
pub const DEFAULT_BATCH_INTERVAL: u64 = 2000;
pub const DEFAULT_RPC_PORT: u16 = 57291;
pub const DEFAULT_STORE_PORT: u16 = 50051;
pub const DEFAULT_BLOCK_PRODUCER_PORT: u16 = 50052;
pub const LOCALHOST: &str = "127.0.0.1";
pub const GENESIS_ACCOUNT_FILE: &str = "account.mac";

// NODE BUILDER
// ================================================================================================

/// Builder for configuring and starting a Miden node with all components.
pub struct NodeBuilder {
    data_directory: PathBuf,
    block_interval: Duration,
    batch_interval: Duration,
    rpc_port: u16,
}

impl NodeBuilder {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------

    /// Creates a new [`NodeBuilder`] with default settings.
    pub fn new(data_directory: PathBuf) -> Self {
        Self {
            data_directory,
            block_interval: Duration::from_millis(DEFAULT_BLOCK_INTERVAL),
            batch_interval: Duration::from_millis(DEFAULT_BATCH_INTERVAL),
            rpc_port: DEFAULT_RPC_PORT,
        }
    }

    // CONFIGURATION
    // --------------------------------------------------------------------------------------------

    /// Sets the block production interval.
    #[must_use]
    pub fn with_block_interval(mut self, interval: Duration) -> Self {
        self.block_interval = interval;
        self
    }

    /// Sets the batch production interval.
    #[must_use]
    pub fn with_batch_interval(mut self, interval: Duration) -> Self {
        self.batch_interval = interval;
        self
    }

    /// Sets the RPC port.
    #[must_use]
    pub fn with_rpc_port(mut self, port: u16) -> Self {
        self.rpc_port = port;
        self
    }

    // START
    // --------------------------------------------------------------------------------------------

    /// Starts all node components and returns a handle to manage them.
    pub async fn start(self) -> Result<NodeHandle> {
        miden_node_utils::logging::setup_tracing(
            miden_node_utils::logging::OpenTelemetry::Disabled,
        )?;

        // Generate the genesis accounts.
        let account_file =
            generate_genesis_account().context("failed to create genesis account")?;

        // Write account data to disk (including secrets).
        //
        // Without this the accounts would be inaccessible by the user.
        // This is not used directly by the node, but rather by the owner / operator of the node.
        let filepath = self.data_directory.join(GENESIS_ACCOUNT_FILE);
        File::create_new(&filepath)
            .and_then(|mut file| file.write_all(&account_file.to_bytes()))
            .with_context(|| {
                format!("failed to write data for genesis account to file {}", filepath.display())
            })?;

        // Bootstrap the store database.
        let version = 1;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current timestamp should be greater than unix epoch")
            .as_secs()
            .try_into()
            .expect("timestamp should fit into u32");
        let genesis_state = GenesisState::new(vec![account_file.account], version, timestamp);

        Store::bootstrap(genesis_state, &self.data_directory)
            .context("failed to bootstrap store")?;

        // Start listening on all gRPC urls so that inter-component connections can be created
        // before each component is fully started up.
        //
        // This is required because `tonic` does not handle retries nor reconnections and our
        // services expect to be able to connect on startup.
        let rpc_url = Url::parse(&format!("http://{LOCALHOST}:{}", self.rpc_port)).unwrap();
        let grpc_rpc = rpc_url.to_socket().context("Failed to to RPC gRPC socket")?;
        let grpc_rpc = TcpListener::bind(grpc_rpc)
            .await
            .context("Failed to bind to RPC gRPC endpoint")?;

        // Define the block producer and ntx builder addresses.
        let block_producer_address = available_socket_addr()
            .await
            .context("Failed to bind to block-producer gRPC endpoint")?;

        let ntx_builder_address = available_socket_addr()
            .await
            .context("Failed to bind to ntx-builder gRPC endpoint")?;

        let mut join_set = JoinSet::new();

        let (store_id, store_address) =
            self.start_store(&mut join_set).await.context("failed to start store")?;

        let ntx_builder_id = NodeBuilder::start_ntx_builder(
            ntx_builder_address,
            block_producer_address,
            store_address,
            &mut join_set,
        );

        let block_producer_id = self.start_block_producer(
            block_producer_address,
            store_address,
            ntx_builder_address,
            &mut join_set,
        );

        // Start RPC component.
        let rpc_id = join_set
            .spawn(async move {
                Rpc {
                    listener: grpc_rpc,
                    store: store_address,
                    block_producer: Some(block_producer_address),
                }
                .serve()
                .await
                .context("failed while serving RPC component")
            })
            .id();

        // Lookup table so we can identify the failed component.
        let component_ids = HashMap::from([
            (store_id, "store"),
            (block_producer_id, "block-producer"),
            (rpc_id, "rpc"),
            (ntx_builder_id, "ntx-builder"),
        ]);

        // SAFETY: The joinset is definitely not empty.
        let component_result = join_set.join_next_with_id().await.unwrap();

        // We expect components to run indefinitely, so we treat any return as fatal.
        //
        // Map all outcomes to an error, and provide component context.
        let (id, err) = match component_result {
            Ok((id, Ok(_))) => (id, Err(anyhow::anyhow!("Component completed unexpectedly"))),
            Ok((id, Err(err))) => (id, Err(err)),
            Err(join_err) => (join_err.id(), Err(join_err).context("Joining component task")),
        };
        let component = component_ids.get(&id).unwrap_or(&"unknown");

        // We could abort and gracefully shutdown the other components, but since we're crashing the
        // node there is no point.

        err.context(format!("Component {component} failed"))
    }

    // Start store and return the tokio task ID plus the store's gRPC address. The store endpoint is
    // available after loading completes.
    async fn start_store(&self, join_set: &mut JoinSet<Result<()>>) -> Result<(Id, SocketAddr)> {
        let grpc_store = TcpListener::bind("127.0.0.1:0")
            .await
            .context("Failed to bind to store gRPC endpoint")?;
        let store_address =
            grpc_store.local_addr().context("Failed to retrieve the store's gRPC address")?;

        let data_directory = self.data_directory.clone();
        Ok((
            join_set
                .spawn(async move {
                    Store { listener: grpc_store, data_directory }
                        .serve()
                        .await
                        .context("failed while serving store component")
                })
                .id(),
            store_address,
        ))
    }

    // Start block-producer and return the tokio task ID. The block-producer's endpoint is available
    // after loading completes.
    fn start_block_producer(
        &self,
        block_producer_address: SocketAddr,
        store_address: SocketAddr,
        ntx_builder_address: SocketAddr,
        join_set: &mut JoinSet<Result<()>>,
    ) -> Id {
        let batch_interval = self.batch_interval;
        let block_interval = self.block_interval;
        join_set
            .spawn(async move {
                BlockProducer {
                    block_producer_address,
                    store_address,
                    ntx_builder_address: Some(ntx_builder_address),
                    batch_prover_url: None,
                    block_prover_url: None,
                    batch_interval,
                    block_interval,
                }
                .serve()
                .await
                .context("failed while serving block-producer component")
            })
            .id()
    }

    // Start ntx-builder and return the tokio task ID. The ntx-builder's endpoint is available after
    // loading completes.
    fn start_ntx_builder(
        ntx_builder_address: SocketAddr,
        block_producer_address: SocketAddr,
        store_address: SocketAddr,
        join_set: &mut JoinSet<Result<()>>,
    ) -> Id {
        // SAFETY: socket addr yields valid URLs
        let store_url =
            Url::parse(&format!("http://{}:{}/", store_address.ip(), store_address.port()))
                .unwrap();

        join_set
            .spawn(async move {
                NetworkTransactionBuilder {
                    ntx_builder_address,
                    store_url,
                    block_producer_address,
                    tx_prover_url: None,
                    ticker_interval: Duration::from_millis(200),
                    account_cache_capacity: NonZeroUsize::new(128).expect("non-zero"),
                }
                .serve_resilient()
                .await
                .context("failed while serving ntx builder component")
            })
            .id()
    }
}

// NODE HANDLE
// ================================================================================================

/// Handle to manage running node components.
pub struct NodeHandle {
    rpc_url: String,
    rpc_handle: tokio::task::JoinHandle<()>,
    block_producer_handle: tokio::task::JoinHandle<()>,
    store_handle: tokio::task::JoinHandle<()>,
}

impl NodeHandle {
    /// Returns the URL where the RPC server is listening.
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    /// Stops all node components.
    pub async fn stop(self) -> Result<()> {
        self.rpc_handle.abort();
        self.block_producer_handle.abort();
        self.store_handle.abort();

        // Wait for the tasks to complete
        let _ = self.rpc_handle.await;
        let _ = self.block_producer_handle.await;
        let _ = self.store_handle.await;

        Ok(())
    }
}

// UTILS
// ================================================================================================

fn generate_genesis_account() -> anyhow::Result<AccountFile> {
    let mut rng = ChaCha20Rng::from_seed(random());
    let secret = SecretKey::with_rng(&mut get_rpo_random_coin(&mut rng));

    let (mut account, account_seed) = create_basic_fungible_faucet(
        rng.random(),
        AccountIdAnchor::PRE_GENESIS,
        TokenSymbol::try_from("TST").expect("TST should be a valid token symbol"),
        12,
        Felt::from(1_000_000u32),
        miden_objects::account::AccountStorageMode::Public,
        AuthScheme::RpoFalcon512 { pub_key: secret.public_key() },
    )?;

    // Force the account nonce to 1.
    //
    // By convention, a nonce of zero indicates a freshly generated local account that has yet
    // to be deployed. An account is deployed onchain along with its first transaction which
    // results in a non-zero nonce onchain.
    //
    // The genesis block is special in that accounts are "deplyed" without transactions and
    // therefore we need bump the nonce manually to uphold this invariant.
    account.set_nonce(ONE).context("failed to set account nonce to 1")?;

    Ok(AccountFile::new(
        account,
        Some(account_seed),
        AuthSecretKey::RpoFalcon512(secret),
    ))
}

async fn available_socket_addr() -> Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0").await.context("Failed to bind to endpoint")?;
    listener.local_addr().context("Failed to retrieve the address")
}
