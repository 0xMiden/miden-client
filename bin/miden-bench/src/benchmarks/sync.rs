use std::path::PathBuf;
use std::sync::Arc;

use miden_client::account::AccountId;
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::rpc::GrpcClient;
use miden_client::{DebugMode, Felt};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use rand::Rng;

use crate::config::BenchConfig;
use crate::metrics::{BenchmarkResult, measure_time_async};
use crate::spinner::with_spinner;

// Helper to create a unique temp directory for each benchmark run
fn create_temp_dir(config: &BenchConfig, suffix: &str) -> PathBuf {
    let base = config.temp_dir();
    let unique_id = uuid::Uuid::new_v4();
    let path = base.join(format!("miden-bench-{suffix}-{unique_id}"));
    std::fs::create_dir_all(&path).expect("Failed to create temp directory");
    path
}

// Helper to create a client for benchmarking
async fn create_benchmark_client(
    config: &BenchConfig,
    suffix: &str,
) -> anyhow::Result<miden_client::Client<FilesystemKeyStore>> {
    let temp_dir = create_temp_dir(config, suffix);
    let store_path = temp_dir.join("store.sqlite3");
    let keystore_path = temp_dir.join("keystore");
    std::fs::create_dir_all(&keystore_path)?;

    let mut rng = rand::rng();
    let coin_seed: [u64; 4] = rng.random();
    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

    let client = ClientBuilder::new()
        .rpc(Arc::new(GrpcClient::new(&config.network, 30_000)))
        .rng(Box::new(rng))
        .sqlite_store(store_path)
        .filesystem_keystore(keystore_path.to_str().expect("keystore path should be valid UTF-8"))
        .in_debug_mode(DebugMode::Disabled)
        .tx_graceful_blocks(None)
        .build()
        .await?;

    Ok(client)
}

/// Runs sync operation benchmarks (requires a running node)
pub async fn run_sync_benchmarks(
    config: &BenchConfig,
    account_id: Option<String>,
) -> anyhow::Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    // Validate account ID upfront if provided
    let parsed_account_id = if let Some(ref account_id_str) = account_id {
        Some(
            AccountId::from_hex(account_id_str)
                .map_err(|e| anyhow::anyhow!("Invalid account ID '{account_id_str}': {e}"))?,
        )
    } else {
        None
    };

    // First, try to connect to the node
    println!("Connecting to node at {}...", config.network);

    let mut client = match create_benchmark_client(config, "sync").await {
        Ok(client) => client,
        Err(e) => {
            println!("Failed to connect to node: {e}");
            println!("Skipping sync benchmarks (requires a running Miden node).");
            results.push(
                BenchmarkResult::new("connection_failed")
                    .with_metadata(format!("Could not connect to node at {}: {e}", config.network)),
            );
            return Ok(results);
        },
    };

    // Sync to get chain height
    client.sync_state().await?;
    let chain_height = client.get_sync_height().await?;
    println!("Connected successfully. Chain height: {chain_height}");

    // Benchmark 1: Sync with no account tracking
    let no_account_result = with_spinner("Benchmarking sync with no account tracking", || {
        benchmark_no_account_tracking(&mut client, config)
    })
    .await?;
    results.push(no_account_result);

    // Benchmark 2: Import public account (if account_id is provided)
    if let Some(account_id) = parsed_account_id {
        let import_result = with_spinner("Benchmarking import public account", || {
            benchmark_import_public_account(config, account_id)
        })
        .await?;
        results.push(import_result);
    }

    Ok(results)
}

/// Benchmarks sync operation with no accounts being tracked
async fn benchmark_no_account_tracking(
    client: &mut miden_client::Client<FilesystemKeyStore>,
    config: &BenchConfig,
) -> anyhow::Result<BenchmarkResult> {
    let mut result = BenchmarkResult::new("no_account_tracking");

    // For each iteration, we need a fresh client with no accounts
    for i in 0..config.iterations {
        let mut fresh_client = create_benchmark_client(config, &format!("sync-iter-{i}")).await?;

        let (_, duration) = measure_time_async(|| async { fresh_client.sync_state().await }).await;

        result.add_iteration(duration);
    }

    // Get the synced block number for metadata
    let block_num = client.get_sync_height().await?;
    result = result.with_metadata(format!("Synced to block {block_num}"));

    Ok(result)
}

/// Benchmarks importing a public account from the network
async fn benchmark_import_public_account(
    config: &BenchConfig,
    account_id: AccountId,
) -> anyhow::Result<BenchmarkResult> {
    let bench_name = "import_public_account".to_string();

    let mut result = BenchmarkResult::new(&bench_name);

    for i in 0..config.iterations {
        let mut client = create_benchmark_client(config, &format!("sync-import-iter-{i}")).await?;

        // Initial sync first
        client.sync_state().await?;

        // Measure the time to import the public account from the network
        let (import_result, duration) =
            measure_time_async(|| async { client.import_account_by_id(account_id).await }).await;

        if let Err(e) = import_result {
            return Err(anyhow::anyhow!(
                "Failed to import account {account_id}: {e}. Make sure the account exists and is public."
            ));
        }

        result.add_iteration(duration);
    }

    result = result.with_metadata(format!("Account {account_id}"));

    Ok(result)
}
