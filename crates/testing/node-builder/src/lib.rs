#![recursion_limit = "256"]

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ::rand::{Rng, random};
use anyhow::{Context, Result};
use miden_lib::AuthScheme;
use miden_lib::account::faucets::create_basic_fungible_faucet;
use miden_lib::utils::Serializable;
use miden_node_block_producer::{
    BlockProducer,
    DEFAULT_MAX_BATCHES_PER_BLOCK,
    DEFAULT_MAX_TXS_PER_BATCH,
};
use miden_node_ntx_builder::NetworkTransactionBuilder;
use miden_node_rpc::Rpc;
use miden_node_store::{GenesisState, Store};
use miden_node_utils::crypto::get_rpo_random_coin;
use miden_objects::account::{Account, AccountFile, AuthSecretKey};
use miden_objects::asset::TokenSymbol;
use miden_objects::block::FeeParameters;
use miden_objects::crypto::dsa::rpo_falcon512::SecretKey;
use miden_objects::testing::account_id::ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET;
use miden_objects::{Felt, ONE};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use tokio::net::TcpListener;
use tokio::sync::Barrier;
use tokio::task::{Id, JoinSet};
use url::Url;

pub const DEFAULT_BLOCK_INTERVAL: u64 = 5_000;
pub const DEFAULT_BATCH_INTERVAL: u64 = 2_000;
pub const DEFAULT_RPC_PORT: u16 = 57_291;
pub const GENESIS_ACCOUNT_FILE: &str = "account.mac";

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
    #[allow(clippy::too_many_lines)]
    pub async fn start(self) -> Result<NodeHandle> {
        miden_node_utils::logging::setup_tracing(
            miden_node_utils::logging::OpenTelemetry::Disabled,
        )?;

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

        let version = 1;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current timestamp should be greater than unix epoch")
            .as_secs()
            .try_into()
            .expect("timestamp should fit into u32");
        let genesis_state = GenesisState::new(
            vec![account_file.account],
            FeeParameters::new(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET.try_into().unwrap(), 0u32)
                .unwrap(),
            version,
            timestamp,
        );

        // Bootstrap the store database
        Store::bootstrap(genesis_state, &self.data_directory)
            .context("failed to bootstrap store")?;

        // Start listening on all gRPC urls so that inter-component connections can be created
        // before each component is fully started up.
        let grpc_rpc = TcpListener::bind(format!("127.0.0.1:{}", self.rpc_port))
            .await
            .context("failed to bind to RPC gRPC endpoint")?;
        let store_rpc_listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("failed to bind to store RPC gRPC endpoint")?;
        let store_ntx_builder_listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("failed to bind to store ntx-builder gRPC endpoint")?;
        let store_block_producer_listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("failed to bind to store block-producer gRPC endpoint")?;

        let store_rpc_address = store_rpc_listener
            .local_addr()
            .context("failed to retrieve the store's RPC gRPC address")?;
        let store_block_producer_address = store_block_producer_listener
            .local_addr()
            .context("failed to retrieve the store's block-producer gRPC address")?;
        let store_ntx_builder_address = store_ntx_builder_listener
            .local_addr()
            .context("failed to retrieve the store's ntx-builder gRPC address")?;

        let block_producer_address = available_socket_addr()
            .await
            .context("failed to bind to block-producer gRPC endpoint")?;

        // Start components

        let mut join_set = JoinSet::new();
        let (store_id, _) = Self::start_store(
            self.data_directory.clone(),
            &mut join_set,
            store_rpc_listener,
            store_ntx_builder_listener,
            store_block_producer_listener,
        )
        .context("failed to start store")?;

        let checkpoint = Arc::new(Barrier::new(2));

        let ntx_builder_id = Self::start_ntx_builder(
            block_producer_address,
            store_ntx_builder_address,
            checkpoint.clone(),
            &mut join_set,
        );

        let block_producer_id = self.start_block_producer(
            block_producer_address,
            store_block_producer_address,
            checkpoint,
            &mut join_set,
        );

        let rpc_id = join_set
            .spawn(async move {
                Rpc {
                    listener: grpc_rpc,
                    store: store_rpc_address,
                    block_producer: Some(block_producer_address),
                }
                .serve()
                .await
                .context("failed while serving RPC component")
            })
            .id();

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
    fn start_store(
        data_directory: PathBuf,
        join_set: &mut JoinSet<Result<()>>,
        rpc_listener: TcpListener,
        ntx_builder_listener: TcpListener,
        block_producer_listener: TcpListener,
    ) -> Result<(Id, SocketAddr)> {
        let store_address = rpc_listener
            .local_addr()
            .context("failed to retrieve the store's gRPC address")?;
        Ok((
            join_set
                .spawn(async move {
                    Store {
                        data_directory,
                        rpc_listener,
                        block_producer_listener,
                        ntx_builder_listener,
                    }
                    .serve()
                    .await
                    .context("failed while serving store component")
                })
                .id(),
            store_address,
        ))
    }

    /// Start block-producer and return the tokio task ID. The block-producer's endpoint is
    /// available after loading completes.
    fn start_block_producer(
        &self,
        block_producer_address: SocketAddr,
        store_address: SocketAddr,
        checkpoint: Arc<Barrier>,
        join_set: &mut JoinSet<Result<()>>,
    ) -> Id {
        let batch_interval = self.batch_interval;
        let block_interval = self.block_interval;
        join_set
            .spawn(async move {
                BlockProducer {
                    block_producer_address,
                    store_address,
                    batch_prover_url: None,
                    block_prover_url: None,
                    batch_interval,
                    block_interval,
                    max_txs_per_batch: DEFAULT_MAX_TXS_PER_BATCH,
                    max_batches_per_block: DEFAULT_MAX_BATCHES_PER_BLOCK,
                    production_checkpoint: checkpoint,
                }
                .serve()
                .await
                .context("failed while serving block-producer component")
            })
            .id()
    }

    /// Start ntx-builder and return the tokio task ID.
    fn start_ntx_builder(
        block_producer_address: SocketAddr,
        store_address: SocketAddr,
        production_checkpoint: Arc<Barrier>,
        join_set: &mut JoinSet<Result<()>>,
    ) -> Id {
        let store_url =
            Url::parse(&format!("http://{}:{}/", store_address.ip(), store_address.port()))
                .unwrap();

        join_set
            .spawn(async move {
                NetworkTransactionBuilder {
                    store_url,
                    block_producer_address,
                    tx_prover_url: None,
                    ticker_interval: Duration::from_millis(200),
                    bp_checkpoint: production_checkpoint,
                }
                .serve_new()
                .await
                .context("failed while serving ntx builder component")
            })
            .id()
    }
}

// NODE HANDLE
// ================================================================================================

pub struct NodeHandle {
    pub rpc_url: String,
    pub rpc_handle: tokio::task::JoinHandle<()>,
    pub block_producer_handle: tokio::task::JoinHandle<()>,
    pub store_handle: tokio::task::JoinHandle<()>,
}

impl NodeHandle {
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

    let (account, account_seed) = create_basic_fungible_faucet(
        rng.random(),
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
    let (id, vault, storage, code, _) = account.into_parts();
    let updated_account = Account::from_parts(id, vault, storage, code, ONE);

    Ok(AccountFile::new(
        updated_account,
        Some(account_seed),
        vec![AuthSecretKey::RpoFalcon512(secret)],
    ))
}

async fn available_socket_addr() -> Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0").await.context("failed to bind to endpoint")?;
    listener.local_addr().context("failed to retrieve the address")
}
