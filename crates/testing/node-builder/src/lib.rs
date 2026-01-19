#![recursion_limit = "256"]

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ::rand::{Rng, random};
use anyhow::{Context, Error, Result};
use miden_node_block_producer::{
    BlockProducer,
    DEFAULT_MAX_BATCHES_PER_BLOCK,
    DEFAULT_MAX_TXS_PER_BATCH,
    DEFAULT_MEMPOOL_TX_CAPACITY,
};
use miden_node_ntx_builder::NetworkTransactionBuilder;
use miden_node_rpc::Rpc;
use miden_node_store::{GenesisState, Store};
use miden_node_utils::crypto::get_rpo_random_coin;
use miden_node_validator::Validator;
use miden_protocol::account::auth::AuthSecretKey;
use miden_protocol::account::{Account, AccountBuilder, AccountComponent, AccountFile, StorageMap};
use miden_protocol::asset::{Asset, FungibleAsset, TokenSymbol};
use miden_protocol::block::FeeParameters;
use miden_protocol::crypto::dsa::ecdsa_k256_keccak;
use miden_protocol::testing::account_id::ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET;
use miden_protocol::utils::Serializable;
use miden_protocol::{Felt, ONE, Word};
use miden_standards::AuthScheme;
use miden_standards::account::components::basic_wallet_library;
use miden_standards::account::faucets::create_basic_fungible_faucet;
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
const DEFAULT_TIMEOUT_DURATION: Duration = Duration::from_secs(10);

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

        let test_faucets_and_account = build_test_faucets_and_account()?;

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
        let validator_signer = ecdsa_k256_keccak::SecretKey::new();

        let genesis_state = GenesisState::new(
            [&[account_file.account][..], &test_faucets_and_account[..]].concat(),
            FeeParameters::new(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET.try_into().unwrap(), 0u32)
                .unwrap(),
            version,
            timestamp,
            validator_signer.clone(),
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

        let validator_address = available_socket_addr()
            .await
            .context("failed to bind to validator gRPC endpoint")?;

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
            validator_address,
            checkpoint,
            &mut join_set,
        );

        let validator_id = join_set
            .spawn({
                async move {
                    Validator {
                        address: validator_address,
                        grpc_timeout: DEFAULT_TIMEOUT_DURATION,
                        signer: validator_signer,
                    }
                    .serve()
                    .await
                    .context("failed while serving validator component")
                }
            })
            .id();

        let rpc_id = join_set
            .spawn(async move {
                let store_url = Url::parse(&format!("http://{store_rpc_address}"))
                    .context("Failed to parse URL")?;
                let block_producer_url = Some(
                    Url::parse(&format!("http://{block_producer_address}"))
                        .context("Failed to parse URL")?,
                );
                let validator_url = Url::parse(&format!("http://{validator_address}"))
                    .context("Failed to parse URL")?;

                Rpc {
                    listener: grpc_rpc,
                    store_url,
                    block_producer_url,
                    validator_url,
                    grpc_timeout: DEFAULT_TIMEOUT_DURATION,
                }
                .serve()
                .await
                .context("failed while serving RPC component")
            })
            .id();

        let component_ids = HashMap::from([
            (store_id, "store"),
            (block_producer_id, "block-producer"),
            (validator_id, "validator"),
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
                        grpc_timeout: DEFAULT_TIMEOUT_DURATION,
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
        validator_address: SocketAddr,
        checkpoint: Arc<Barrier>,
        join_set: &mut JoinSet<Result<()>>,
    ) -> Id {
        let batch_interval = self.batch_interval;
        let block_interval = self.block_interval;
        join_set
            .spawn(async move {
                let store_url = Url::parse(&format!("http://{store_address}"))
                    .context("Failed to parse URL")?;
                let validator_url = Url::parse(&format!("http://{validator_address}"))
                    .context("Failed to parse URL")?;
                BlockProducer {
                    block_producer_address,
                    store_url,
                    grpc_timeout: DEFAULT_TIMEOUT_DURATION,
                    batch_prover_url: None,
                    block_prover_url: None,
                    validator_url,
                    batch_interval,
                    block_interval,
                    max_txs_per_batch: DEFAULT_MAX_TXS_PER_BATCH,
                    max_batches_per_block: DEFAULT_MAX_BATCHES_PER_BLOCK,
                    production_checkpoint: checkpoint,
                    mempool_tx_capacity: DEFAULT_MEMPOOL_TX_CAPACITY,
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
        let block_producer_url = Url::parse(&format!(
            "http://{}:{}/",
            block_producer_address.ip(),
            block_producer_address.port()
        ))
        .unwrap();

        join_set
            .spawn(async move {
                NetworkTransactionBuilder::new(
                    store_url,
                    block_producer_url,
                    None,
                    Duration::from_millis(200),
                    production_checkpoint,
                )
                .run()
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

/// Expected account ID for the test account. Used to verify deterministic generation.
const TEST_ACCOUNT_ID: &str = "0x0a0a0a0a0a0a0a100a0a0a0a0a0a0a";

/// Deterministic seed used for the test account to ensure reproducible account IDs.
const TEST_ACCOUNT_SEED: [u8; 32] = [0xa; 32];

/// Number of faucets to create. This value is chosen to exceed typical limits
/// and trigger the "too_many_assets" flag during testing.
const NUM_TEST_FAUCETS: u128 = 1501;

const NUM_STORAGE_MAP_ENTRIES: u32 = 2001;

const FAUCET_DECIMALS: u8 = 12;
const FAUCET_MAX_SUPPLY: u32 = 1 << 30;
const ASSET_AMOUNT_PER_FAUCET: u64 = 100;

/// Builds test faucets and an account that triggers the "too_many_assets" flag
/// when requested from the node. This is used to test edge cases in account
/// retrieval and asset handling.
fn build_test_faucets_and_account() -> anyhow::Result<Vec<Account>> {
    let mut rng = ChaCha20Rng::from_seed(random());
    let secret = AuthSecretKey::new_falcon512_rpo_with_rng(&mut get_rpo_random_coin(&mut rng));

    let faucets = create_test_faucets(&secret)?;
    let account = create_test_account_with_many_assets(&faucets)?;

    assert_eq!(
        account.id().to_hex(),
        TEST_ACCOUNT_ID,
        "test account was generated with a different id than expected; \
         this may indicate a change in account generation logic"
    );

    Ok([&faucets[..], &[account][..]].concat())
}

/// Creates multiple fungible faucets for testing purposes.
/// Each faucet is created with a deterministic seed derived from its index,
/// ensuring reproducible test scenarios.
fn create_test_faucets(secret: &AuthSecretKey) -> anyhow::Result<Vec<Account>> {
    (0..NUM_TEST_FAUCETS)
        .map(|i| create_single_test_faucet(i, secret))
        .collect::<Result<Vec<_>>>()
        .map_err(|err| Error::msg(format!("failed to create test faucets: {err}")))
}

fn create_single_test_faucet(index: u128, secret: &AuthSecretKey) -> anyhow::Result<Account> {
    let init_seed: [u8; 32] = [index.to_be_bytes(), index.to_be_bytes()]
        .concat()
        .try_into()
        .expect("concatenating two 16-byte arrays yields exactly 32 bytes");

    let auth_scheme = AuthScheme::RpoFalcon512 {
        pub_key: secret.public_key().to_commitment(),
    };

    let faucet = create_basic_fungible_faucet(
        init_seed,
        TokenSymbol::new("TKN")?,
        FAUCET_DECIMALS,
        Felt::from(FAUCET_MAX_SUPPLY),
        miden_protocol::account::AccountStorageMode::Public,
        auth_scheme,
    )?;

    // Set nonce to ONE to indicate the account is deployed (see generate_genesis_account)
    let (id, vault, storage, code, ..) = faucet.into_parts();
    Ok(Account::new_unchecked(id, vault, storage, code, ONE, None))
}

/// Creates a test account holding assets from all provided faucets.
/// The account also includes a large storage map to test storage capacity limits.
fn create_test_account_with_many_assets(faucets: &[Account]) -> anyhow::Result<Account> {
    let sk =
        AuthSecretKey::new_falcon512_rpo_with_rng(&mut ChaCha20Rng::from_seed(TEST_ACCOUNT_SEED));

    let storage_map = create_large_storage_map();
    let acc_component = AccountComponent::new(basic_wallet_library(), vec![storage_map])
        .expect("basic wallet component should satisfy account component requirements")
        .with_supports_all_types();

    let assets = faucets.iter().map(|faucet| {
        Asset::Fungible(
            FungibleAsset::new(faucet.id(), ASSET_AMOUNT_PER_FAUCET)
                .expect("faucet id should be valid for asset creation"),
        )
    });

    let account = AccountBuilder::new(TEST_ACCOUNT_SEED)
        .with_auth_component(miden_standards::account::auth::AuthRpoFalcon512::new(
            sk.public_key().to_commitment(),
        ))
        .account_type(miden_protocol::account::AccountType::RegularAccountUpdatableCode)
        .with_component(acc_component)
        .storage_mode(miden_protocol::account::AccountStorageMode::Public)
        .with_assets(assets)
        .build_existing()?;

    Ok(account)
}

/// Creates a storage map with many entries for stress-testing storage handling.
fn create_large_storage_map() -> miden_protocol::account::StorageSlot {
    let map_entries = (0..NUM_STORAGE_MAP_ENTRIES).map(|i| (Word::from([i; 4]), Word::from([i; 4])));

    miden_protocol::account::StorageSlot::with_map(
        miden_protocol::account::StorageSlotName::new("miden::test_account::map::too_many_entries")
            .expect("slot name should be valid"),
        StorageMap::with_entries(map_entries).expect("map entries should be valid"),
    )
}

fn generate_genesis_account() -> anyhow::Result<AccountFile> {
    let mut rng = ChaCha20Rng::from_seed(random());
    let secret = AuthSecretKey::new_falcon512_rpo_with_rng(&mut get_rpo_random_coin(&mut rng));

    let account = create_basic_fungible_faucet(
        rng.random(),
        TokenSymbol::try_from("TST").expect("TST should be a valid token symbol"),
        12,
        Felt::from(1_000_000u32),
        miden_protocol::account::AccountStorageMode::Public,
        AuthScheme::RpoFalcon512 {
            pub_key: secret.public_key().to_commitment(),
        },
    )?;

    // Force the account nonce to 1.
    //
    // By convention, a nonce of zero indicates a freshly generated local account that has yet
    // to be deployed. An account is deployed onchain along within its first transaction which
    // results in a non-zero nonce onchain.
    //
    // The genesis block is special in that accounts are "deployed" without transactions and
    // therefore we need bump the nonce manually to uphold this invariant.
    let (id, vault, storage, code, ..) = account.into_parts();
    let updated_account = Account::new_unchecked(id, vault, storage, code, ONE, None);

    Ok(AccountFile::new(updated_account, vec![secret]))
}

async fn available_socket_addr() -> Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0").await.context("failed to bind to endpoint")?;
    listener.local_addr().context("failed to retrieve the address")
}
