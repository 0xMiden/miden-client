#![recursion_limit = "256"]

use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ::rand::{Rng, random};
use anyhow::{Context, Result};
use miden_node_block_producer::{
    BlockProducerApi,
    DEFAULT_MAX_BATCHES_PER_BLOCK,
    DEFAULT_MAX_CONCURRENT_PROOFS,
    DEFAULT_MAX_TXS_PER_BATCH,
    DEFAULT_MEMPOOL_TX_CAPACITY,
    Sequencer,
};
use miden_node_proto::clients::{
    Builder as GrpcClientBuilder,
    GrpcClient,
    NtxBuilderClient,
    ValidatorClient,
};
use miden_node_rpc::{Rpc, RpcMode};
use miden_node_store::state::State;
use miden_node_store::{DatabaseOptions, GenesisState, default_sqlite_connection_pool_size};
use miden_node_utils::clap::{GrpcOptionsExternal, GrpcOptionsInternal, StorageOptions};
use miden_node_utils::crypto::get_random_coin;
use miden_ntx_builder::NtxBuilderConfig;
use miden_protocol::account::auth::{AuthScheme, AuthSecretKey};
use miden_protocol::account::{
    Account,
    AccountBuilder,
    AccountComponent,
    AccountComponentMetadata,
    AccountFile,
    AccountType,
    StorageMap,
    StorageMapKey,
};
use miden_protocol::asset::{Asset, AssetAmount, FungibleAsset, TokenSymbol};
use miden_protocol::block::FeeParameters;
use miden_protocol::crypto::dsa::ecdsa_k256_keccak;
use miden_protocol::testing::account_id::ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET;
use miden_protocol::utils::serde::Serializable;
use miden_protocol::{ONE, Word};
use miden_standards::AuthMethod;
use miden_standards::account::access::AccessControl;
use miden_standards::account::auth::AuthSingleSig;
use miden_standards::account::faucets::{FungibleFaucet, TokenName, create_fungible_faucet};
use miden_standards::account::policies::{
    BurnPolicyConfig,
    MintPolicyConfig,
    PolicyRegistration,
    TokenPolicyManager,
};
use miden_standards::account::wallets::BasicWallet;
use miden_validator::{Validator, ValidatorSigner};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tonic::metadata::AsciiMetadataValue;
use url::Url;

pub const DEFAULT_BLOCK_INTERVAL: u64 = 5_000;
pub const DEFAULT_BATCH_INTERVAL: u64 = 2_000;
pub const DEFAULT_RPC_PORT: u16 = 57_291;
/// Default remote transaction prover endpoint used by the ntx-builder. Matches the local prover
/// started via `make start-prover` (see `crates/testing/prover`).
pub const DEFAULT_PROVER_URL: &str = "http://127.0.0.1:50051";
pub const GENESIS_ACCOUNT_FILE: &str = "account.mac";
/// Arbitrary shared secret authenticating ntx-builder RPC calls. The value is unconstrained;
/// the ntx-builder (client) and the RPC server (validator) only need to agree on it.
const NTX_AUTH_HEADER_VALUE: &str = "miden-client-testing-node-ntx";

/// Builder for configuring and starting a Miden node with all components.
pub struct NodeBuilder {
    data_directory: PathBuf,
    block_interval: Duration,
    batch_interval: Duration,
    rpc_port: u16,
    prover_url: Url,
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
            prover_url: Url::parse(DEFAULT_PROVER_URL).expect("default prover URL is valid"),
        }
    }

    /// Sets the remote transaction prover endpoint used by the ntx-builder.
    #[must_use]
    pub fn with_prover_url(mut self, prover_url: Url) -> Self {
        self.prover_url = prover_url;
        self
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
        let validator_signing_key = ecdsa_k256_keccak::SigningKey::new();
        let validator_public_key = validator_signing_key.public_key();

        let genesis_state = GenesisState::new(
            [&[account_file.account][..], &test_faucets_and_account[..]].concat(),
            FeeParameters::new(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET.try_into().unwrap(), 0u32),
            version,
            timestamp,
            validator_public_key,
        );

        // Bootstrap the store database
        let genesis_block = genesis_state
            .into_block(&validator_signing_key)
            .with_context(|| "failed to create genesis block")?;

        let signed_genesis_block = genesis_block.inner().clone();
        let genesis_header = signed_genesis_block.header().clone();
        State::bootstrap(genesis_block, &self.data_directory)
            .with_context(|| "failed to bootstrap store")?;

        // Bootstrap the validator database with the genesis block header so that block
        // validation can find the chain tip.
        let validator_db = miden_validator::db::load(self.data_directory.join("validator.sqlite3"))
            .await
            .with_context(|| "failed to initialize validator database")?;
        validator_db
            .transact("bootstrap_validator", move |conn| {
                miden_validator::db::upsert_block_header(conn, &genesis_header)
            })
            .await
            .with_context(|| "failed to bootstrap validator with genesis block header")?;

        let grpc_rpc = TcpListener::bind(format!("127.0.0.1:{}", self.rpc_port))
            .await
            .with_context(|| "failed to bind to RPC gRPC endpoint")?;
        let rpc_address = grpc_rpc
            .local_addr()
            .with_context(|| "failed to retrieve the RPC gRPC address")?;

        let ntx_builder_listener = TcpListener::bind("127.0.0.1:0")
            .await
            .with_context(|| "failed to bind to ntx-builder gRPC endpoint")?;
        let ntx_builder_address = ntx_builder_listener
            .local_addr()
            .with_context(|| "failed to retrieve the ntx-builder gRPC address")?;

        let validator_address = available_socket_addr()
            .await
            .with_context(|| "failed to bind to validator gRPC endpoint")?;

        // Start components

        let sqlite_connection_pool_size = default_sqlite_connection_pool_size();
        let ntx_builder_database_filepath = self.data_directory.join("ntx-builder.sqlite3");
        miden_ntx_builder::bootstrap(ntx_builder_database_filepath.clone(), &signed_genesis_block)
            .await
            .context("failed to bootstrap ntx-builder database")?;

        let store = State::load_with_database_options(
            &self.data_directory,
            StorageOptions::default(),
            DatabaseOptions {
                connection_pool_size: sqlite_connection_pool_size,
            },
        )
        .await
        .context("failed to load store state")?;
        let store = Arc::new(store);
        let ntx_auth_header = AsciiMetadataValue::from_static(NTX_AUTH_HEADER_VALUE);

        let ntx_builder_handle = Self::start_ntx_builder(
            rpc_address,
            self.prover_url.clone(),
            ntx_builder_database_filepath,
            ntx_builder_listener,
            ntx_auth_header.clone(),
        );

        let (block_producer_api, block_producer_handle) =
            self.start_block_producer(Arc::clone(&store), validator_address).await?;

        let validator_handle = tokio::spawn({
            let data_directory = self.data_directory.clone();
            async move {
                Validator {
                    address: validator_address,
                    grpc_options: GrpcOptionsInternal::default(),
                    signer: ValidatorSigner::new_local(validator_signing_key),
                    data_directory,
                    sqlite_connection_pool_size,
                }
                .serve()
                .await
                .context("failed while serving validator component")
            }
        });

        let rpc_handle = tokio::spawn(async move {
            let validator_url = Url::parse(&format!("http://{validator_address}"))
                .context("Failed to parse URL")?;
            let validator: ValidatorClient = grpc_client(validator_url);
            let ntx_builder_url = Url::parse(&format!("http://{ntx_builder_address}"))
                .context("Failed to parse URL")?;
            let ntx_builder: Option<NtxBuilderClient> = Some(grpc_client(ntx_builder_url));

            Rpc {
                listener: grpc_rpc,
                store,
                mode: RpcMode::sequencer(block_producer_api, validator),
                ntx_builder,
                grpc_options: GrpcOptionsExternal {
                    burst_size: NonZeroU32::new(10_000).unwrap(),
                    replenish_n_per_second_per_ip: NonZeroU64::new(10_000).unwrap(),
                    ..GrpcOptionsExternal::default()
                },
                network_tx_auth: Some(ntx_auth_header),
            }
            .serve()
            .await
            .context("failed while serving RPC component")
        });

        Ok(NodeHandle {
            rpc_url: format!("http://{rpc_address}"),
            handles: vec![block_producer_handle, validator_handle, rpc_handle, ntx_builder_handle],
        })
    }

    /// Start block-producer and return its in-process API and runtime task.
    async fn start_block_producer(
        &self,
        store: Arc<State>,
        validator_address: SocketAddr,
    ) -> Result<(BlockProducerApi, JoinHandle<Result<()>>)> {
        let batch_interval = self.batch_interval;
        let block_interval = self.block_interval;
        let validator_url =
            Url::parse(&format!("http://{validator_address}")).context("Failed to parse URL")?;
        let sequencer = Sequencer {
            store,
            validator_url,
            batch_prover_url: None,
            block_prover_url: None,
            batch_interval,
            block_interval,
            max_txs_per_batch: DEFAULT_MAX_TXS_PER_BATCH,
            max_batches_per_block: DEFAULT_MAX_BATCHES_PER_BLOCK,
            max_concurrent_proofs: DEFAULT_MAX_CONCURRENT_PROOFS,
            mempool_tx_capacity: DEFAULT_MEMPOOL_TX_CAPACITY,
        };

        let runtime = sequencer
            .spawn()
            .await
            .context("failed while starting block-producer component")?;
        let api = runtime.api();
        let handle = tokio::spawn(async move {
            runtime.wait().await.context("failed while serving block-producer component")
        });

        Ok((api, handle))
    }

    /// Start ntx-builder and return the task handle.
    fn start_ntx_builder(
        rpc_address: SocketAddr,
        prover_url: Url,
        database_filepath: PathBuf,
        listener: TcpListener,
        rpc_auth_header: AsciiMetadataValue,
    ) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let rpc_url =
                Url::parse(&format!("http://{rpc_address}/")).context("Failed to parse URL")?;

            NtxBuilderConfig::new(rpc_url, prover_url, database_filepath)
                .with_max_cycles(1 << 18)
                .with_rpc_auth_header(rpc_auth_header)
                .build()
                .await
                .context("failed to build ntx builder")?
                .run(listener)
                .await
                .context("failed while serving ntx builder component")
        })
    }
}

// NODE HANDLE
// ================================================================================================

pub struct NodeHandle {
    pub rpc_url: String,
    handles: Vec<JoinHandle<Result<()>>>,
}

impl NodeHandle {
    /// Stops all node components.
    pub async fn stop(mut self) -> Result<()> {
        for handle in &self.handles {
            handle.abort();
        }

        while let Some(handle) = self.handles.pop() {
            let _ = handle.await;
        }

        Ok(())
    }
}

// UTILS
// ================================================================================================

fn generate_genesis_account() -> anyhow::Result<AccountFile> {
    let mut rng = ChaCha20Rng::from_seed(random());
    let secret = AuthSecretKey::new_falcon512_poseidon2_with_rng(&mut get_random_coin(&mut rng));

    let auth_method = AuthMethod::SingleSig {
        approver: (secret.public_key().to_commitment(), AuthScheme::Falcon512Poseidon2),
    };

    let symbol = TokenSymbol::try_from("TST").expect("TST should be a valid token symbol");
    let name = TokenName::new(&symbol.to_string()).expect("token symbol is a valid token name");
    let faucet = FungibleFaucet::builder()
        .name(name)
        .symbol(symbol)
        .decimals(12)
        .max_supply(AssetAmount::new(1_000_000_000_000).unwrap())
        .build()?;
    let account = create_fungible_faucet(
        rng.random(),
        faucet,
        AccountType::Public,
        auth_method,
        AccessControl::AuthControlled,
        allow_all_policy_manager(),
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

fn grpc_client<T: GrpcClient>(url: Url) -> T {
    GrpcClientBuilder::new(url)
        .without_tls()
        .without_timeout()
        .without_metadata_version()
        .without_metadata_genesis()
        .with_otel_context_injection()
        .connect_lazy::<T>()
}

// UTILS (GENESIS ACCOUNTS)
// ================================================================================================

/// Expected account ID produced by [`TEST_ACCOUNT_SEED`] under the current `FungibleFaucet`
/// component layout, policy components, and schema commitments. Used to verify deterministic
/// account generation; update this constant if any input to ID derivation changes.
const TEST_ACCOUNT_ID: &str = "0x0a0a0a0a0a0a0a110a0a0a0a0a0a0a";

/// Deterministic seed used for the test account to ensure reproducible account IDs.
const TEST_ACCOUNT_SEED: [u8; 32] = [0xa; 32];

/// Number of faucets to create. This value is chosen to exceed typical limits
/// and trigger the `too_many_assets` flag during testing.
const NUM_TEST_FAUCETS: u128 = 1501;

const NUM_STORAGE_MAP_ENTRIES: u32 = 200;

const FAUCET_DECIMALS: u8 = 12;
const FAUCET_MAX_SUPPLY: u32 = 1 << 30;
const ASSET_AMOUNT_PER_FAUCET: u64 = 75;

/// Builds test faucets and an account that triggers the `too_many_assets` flag
/// when requested from the node. This is used to test edge cases in account
/// retrieval and asset handling.
fn build_test_faucets_and_account() -> anyhow::Result<Vec<Account>> {
    let mut rng = ChaCha20Rng::from_seed(random());
    let secret = AuthSecretKey::new_falcon512_poseidon2_with_rng(&mut get_random_coin(&mut rng));

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
        .map_err(|err| anyhow::Error::msg(format!("failed to create test faucets: {err}")))
}

fn create_single_test_faucet(index: u128, secret: &AuthSecretKey) -> anyhow::Result<Account> {
    let init_seed: [u8; 32] = [index.to_be_bytes(), index.to_be_bytes()]
        .concat()
        .try_into()
        .expect("concatenating two 16-byte arrays yields exactly 32 bytes");

    let auth_scheme = AuthMethod::SingleSig {
        approver: (secret.public_key().to_commitment(), AuthScheme::Falcon512Poseidon2),
    };

    let symbol = TokenSymbol::new("TKN")?;
    let name = TokenName::new(&symbol.to_string()).expect("token symbol is a valid token name");
    let faucet_component = FungibleFaucet::builder()
        .name(name)
        .symbol(symbol)
        .decimals(FAUCET_DECIMALS)
        .max_supply(AssetAmount::new(u64::from(FAUCET_MAX_SUPPLY)).unwrap())
        .build()?;
    let faucet = create_fungible_faucet(
        init_seed,
        faucet_component,
        AccountType::Public,
        auth_scheme,
        AccessControl::AuthControlled,
        allow_all_policy_manager(),
    )?;

    // Set nonce to ONE to indicate the account is deployed (see generate_genesis_account)
    let (id, vault, storage, code, ..) = faucet.into_parts();
    Ok(Account::new_unchecked(id, vault, storage, code, ONE, None))
}

/// Creates a test account holding assets from all provided faucets.
/// The account also includes a large storage map to test storage capacity limits.
fn create_test_account_with_many_assets(faucets: &[Account]) -> anyhow::Result<Account> {
    let sk = AuthSecretKey::new_falcon512_poseidon2_with_rng(&mut ChaCha20Rng::from_seed(
        TEST_ACCOUNT_SEED,
    ));

    let storage_map = create_large_storage_map();
    let acc_component = AccountComponent::new(
        BasicWallet::code().as_library().clone(),
        vec![storage_map],
        AccountComponentMetadata::new("miden::testing::basic_wallet"),
    )
    .expect("basic wallet component should satisfy account component requirements");

    let assets = faucets.iter().map(|faucet| {
        Asset::Fungible(
            FungibleAsset::new(faucet.id(), ASSET_AMOUNT_PER_FAUCET)
                .expect("faucet id should be valid for asset creation"),
        )
    });

    let account = AccountBuilder::new(TEST_ACCOUNT_SEED)
        .with_auth_component(AuthSingleSig::new(
            sk.public_key().to_commitment(),
            AuthScheme::Falcon512Poseidon2,
        ))
        .account_type(AccountType::Public)
        .with_component(acc_component)
        .with_assets(assets)
        .build_existing()?;

    Ok(account)
}

fn allow_all_policy_manager() -> TokenPolicyManager {
    // Only mint/burn — registering transfer policies installs asset-callback slots on the
    // faucet, which forces minted assets to carry `AssetCallbackFlag::Enabled`. Tests build
    // assets via `FungibleAsset::new`, which defaults to `Disabled`, so adding transfer
    // policies makes `mint_and_send` reject the mint with
    // `ERR_FUNGIBLE_MINT_NOTE_ASSET_NOT_FROM_THIS_FAUCET`.
    TokenPolicyManager::new()
        .with_mint_policy(MintPolicyConfig::AllowAll, PolicyRegistration::Active)
        .expect("allow-all mint policy should register")
        .with_burn_policy(BurnPolicyConfig::AllowAll, PolicyRegistration::Active)
        .expect("allow-all burn policy should register")
}

/// Creates a storage map with many entries for stress-testing storage handling.
fn create_large_storage_map() -> miden_protocol::account::StorageSlot {
    let map_entries = (0..NUM_STORAGE_MAP_ENTRIES)
        .map(|i| (StorageMapKey::new(Word::from([i; 4])), Word::from([i; 4])));

    miden_protocol::account::StorageSlot::with_map(
        miden_protocol::account::StorageSlotName::new("miden::test_account::map::too_many_entries")
            .expect("slot name should be valid"),
        StorageMap::with_entries(map_entries).expect("map entries should be valid"),
    )
}
