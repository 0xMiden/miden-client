use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;

use miden_client::account::{AccountId, StorageSlotContent};
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::rpc::GrpcClient;
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::{DebugMode, Felt, Word};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_protocol::account::auth::AuthSecretKey;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use crate::config::BenchConfig;
use crate::generators::generate_reader_component_code;
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
) -> anyhow::Result<(miden_client::Client<FilesystemKeyStore>, FilesystemKeyStore, PathBuf)> {
    let temp_dir = create_temp_dir(config, suffix);
    let store_path = temp_dir.join("store.sqlite3");
    let keystore_path = temp_dir.join("keystore");
    std::fs::create_dir_all(&keystore_path)?;

    let mut rng = rand::rng();
    let coin_seed: [u64; 4] = rng.random();
    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

    let keystore = FilesystemKeyStore::new(keystore_path.clone())
        .expect("Failed to create filesystem keystore");

    let client = ClientBuilder::new()
        .rpc(Arc::new(GrpcClient::new(&config.network, 30_000)))
        .rng(Box::new(rng))
        .sqlite_store(store_path)
        .filesystem_keystore(keystore_path.to_str().expect("keystore path should be valid UTF-8"))?
        .in_debug_mode(DebugMode::Disabled)
        .tx_discard_delta(None)
        .build()
        .await?;

    Ok((client, keystore, temp_dir))
}

/// Information about the account's storage maps extracted from the network
#[derive(Clone, Debug)]
struct StorageMapInfo {
    /// Keys in this storage map (derived from the slot index)
    keys: Vec<Word>,
}

/// Runs transaction benchmarks (requires a running node)
///
/// The benchmark uses the specified account as the native account executing transactions.
/// Each transaction reads all storage map entries from the account's own storage.
///
/// The number of maps is auto-detected from the account storage. When `entries_per_map`
/// is provided, keys are generated deterministically (required for two-phase deployed
/// accounts whose expansion entries aren't visible via the import RPC).
pub async fn run_transaction_benchmarks(
    config: &BenchConfig,
    account_id_str: String,
    seed: Option<[u8; 32]>,
    entries_per_map: Option<usize>,
) -> anyhow::Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    // Parse the account ID
    let account_id = AccountId::from_hex(&account_id_str)?;

    // First, try to connect to the node and fetch the account
    println!("Connecting to node at {}...", config.network);

    let (mut client, _keystore, _temp_dir) =
        match create_benchmark_client(config, "tx-init").await {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to connect to node: {e}");
                println!("Skipping transaction benchmarks (requires a running Miden node).");
                results.push(BenchmarkResult::new("transaction/connection_failed").with_metadata(
                    format!("Could not connect to node at {}: {e}", config.network),
                ));
                return Ok(results);
            },
        };

    // Sync with the network first
    if let Err(e) = client.sync_state().await {
        println!("Failed to sync with node: {e}");
        println!("Skipping transaction benchmarks.");
        results.push(
            BenchmarkResult::new("transaction/sync_failed")
                .with_metadata(format!("Failed to sync: {e}")),
        );
        return Ok(results);
    }

    let chain_height = client.get_sync_height().await?;
    println!("Connected successfully. Chain height: {chain_height}");

    // Import the public account from the network
    println!("Importing account {account_id}...");
    client.import_account_by_id(account_id).await?;

    // Auto-detect the number of map-type storage slots from the imported account.
    let storage = client.get_account_storage(account_id).await?;
    let num_maps = storage
        .slots()
        .iter()
        .filter(|slot| matches!(slot.content(), StorageSlotContent::Map(_)))
        .count();

    if num_maps == 0 {
        anyhow::bail!("Account has no storage map slots to benchmark");
    }

    // Build storage map keys. When entries_per_map is provided, generate keys
    // deterministically using the same scheme as the deploy command. This is
    // required for two-phase deployed accounts: the node only returns initial
    // component entries via the import RPC, not entries added by expansion
    // transactions. When omitted, read keys from the imported account directly
    // (works for single-tx deployed accounts where all entries are in the component).
    let storage_maps: Vec<StorageMapInfo> = if let Some(entries_per_map) = entries_per_map {
        generate_storage_map_keys(num_maps, entries_per_map)
    } else {
        storage
            .slots()
            .iter()
            .filter_map(|slot| match slot.content() {
                StorageSlotContent::Map(map) => {
                    let keys = map.entries().map(|(k, _v)| *k).collect();
                    Some(StorageMapInfo { keys })
                },
                StorageSlotContent::Value(_) => None,
            })
            .collect()
    };

    let total_entries: usize = storage_maps.iter().map(|m| m.keys.len()).sum();
    if total_entries == 0 {
        anyhow::bail!(
            "Account has no visible storage map entries. If this account was deployed \
             with two-phase expansion, pass --entries-per-map to generate keys."
        );
    }

    println!("Storage maps: {num_maps}, total entries: {total_entries}");

    // Regenerate the signing key from the deployment seed (if provided)
    let secret_key =
        seed.map(|s| AuthSecretKey::new_falcon512_rpo_with_rng(&mut ChaCha20Rng::from_seed(s)));

    // Benchmark 1: Transaction execution time (without proving)
    let execution_result = with_spinner("Benchmarking transaction execution", || {
        benchmark_tx_execution(config, account_id, &storage_maps, secret_key.as_ref())
    })
    .await?;
    results.push(execution_result);

    // Benchmarks 2 & 3 require the signing key for proving and submission
    if let Some(ref sk) = secret_key {
        // Benchmark 2: Transaction proving time
        let proving_result = with_spinner("Benchmarking transaction proving", || {
            benchmark_tx_proving(config, account_id, &storage_maps, sk)
        })
        .await?;
        results.push(proving_result);

        // Benchmark 3: Full transaction (execute + prove + submit)
        let full_result = with_spinner("Benchmarking full transaction", || {
            Box::pin(benchmark_tx_full(config, account_id, &storage_maps, sk))
        })
        .await?;
        results.push(full_result);
    } else {
        println!("Skipping proving and submission benchmarks (no seed provided).");
    }

    Ok(results)
}

/// Generates storage map keys deterministically using the same scheme as the deploy command.
///
/// Both deployment paths (single-tx and two-phase) use the formula:
/// `key_val = map_index * 1000 + entry_index` for each entry.
#[allow(clippy::cast_possible_truncation)]
fn generate_storage_map_keys(num_maps: usize, entries_per_map: usize) -> Vec<StorageMapInfo> {
    (0..num_maps)
        .map(|map_idx| {
            let seed = map_idx as u32;
            let keys = (0..entries_per_map as u32)
                .map(|j| {
                    let key_val = seed.wrapping_mul(1000).wrapping_add(j);
                    [Felt::new(u64::from(key_val)); 4].into()
                })
                .collect();
            StorageMapInfo { keys }
        })
        .collect()
}

/// Generates a MASM script that reads all storage map entries from the active account.
///
/// Uses `call` to invoke account reader procedures (`get_map_item_slot_N`) rather than
/// directly `exec`-ing kernel syscalls. The kernel's `authenticate_account_origin` requires
/// the caller to be an account procedure, so the transaction script must `call` into the
/// account's reader component which then `exec`s the syscall.
fn generate_storage_read_script(storage_maps: &[StorageMapInfo]) -> String {
    let mut script = String::from(
        "use bench_reader::storage_reader
begin
",
    );

    for (map_idx, map_info) in storage_maps.iter().enumerate() {
        for key in &map_info.keys {
            // Push key (4 felts) - order: [key0, key1, key2, key3] on stack top
            writeln!(
                script,
                "    push.{}.{}.{}.{}",
                key[3].as_int(),
                key[2].as_int(),
                key[1].as_int(),
                key[0].as_int()
            )
            .expect("write to string should not fail");

            // Call the account's reader procedure for this storage map slot.
            // The reader procedure pushes the slot ID and exec's get_map_item.
            // Stack input: [KEY]
            // Stack output: [VALUE, pad(12)] = 16 elements
            writeln!(script, "    call.storage_reader::get_map_item_slot_{map_idx}")
                .expect("write to string should not fail");

            // Drop the result (we just want to measure read time)
            script.push_str("    dropw dropw dropw dropw\n");
        }
    }

    script.push_str("end\n");
    script
}

/// Benchmarks transaction execution time (reading storage from active account)
async fn benchmark_tx_execution(
    config: &BenchConfig,
    account_id: AccountId,
    storage_maps: &[StorageMapInfo],
    secret_key: Option<&AuthSecretKey>,
) -> anyhow::Result<BenchmarkResult> {
    let total_entries: usize = storage_maps.iter().map(|m| m.keys.len()).sum();
    let bench_name = format!("execute ({total_entries} storage reads)");

    let mut result = BenchmarkResult::new(&bench_name);

    for i in 0..config.iterations {
        let (mut client, keystore, _) =
            create_benchmark_client(config, &format!("tx-exec-iter-{i}")).await?;
        client.sync_state().await?;

        // Import the account and add the signing key (if available)
        client.import_account_by_id(account_id).await?;
        if let Some(sk) = secret_key {
            keystore.add_key(sk)?;
        }

        // Generate the script that reads all storage entries from active account
        let script_code = generate_storage_read_script(storage_maps);
        let reader_code = generate_reader_component_code(storage_maps.len());
        let tx_script = client
            .code_builder()
            .with_linked_module("bench_reader::storage_reader", reader_code.as_str())?
            .compile_tx_script(&script_code)?;

        // Create the transaction request
        let tx_request = TransactionRequestBuilder::new().custom_script(tx_script).build()?;

        // Measure execution time only
        let (_, duration) = measure_time_async(|| async {
            client.execute_transaction(account_id, tx_request).await
        })
        .await;

        result.add_iteration(duration);
    }

    result = result.with_metadata(format!(
        "Transaction execution (no proving), {total_entries} storage reads from active account"
    ));

    Ok(result)
}

/// Benchmarks transaction proving time
async fn benchmark_tx_proving(
    config: &BenchConfig,
    account_id: AccountId,
    storage_maps: &[StorageMapInfo],
    secret_key: &AuthSecretKey,
) -> anyhow::Result<BenchmarkResult> {
    let total_entries: usize = storage_maps.iter().map(|m| m.keys.len()).sum();
    let bench_name = format!("prove ({total_entries} storage reads)");

    let mut result = BenchmarkResult::new(&bench_name);

    for i in 0..config.iterations {
        let (mut client, keystore, _) =
            create_benchmark_client(config, &format!("tx-prove-iter-{i}")).await?;
        client.sync_state().await?;

        // Import the account and add the signing key
        client.import_account_by_id(account_id).await?;
        keystore.add_key(secret_key)?;

        let script_code = generate_storage_read_script(storage_maps);
        let reader_code = generate_reader_component_code(storage_maps.len());
        let tx_script = client
            .code_builder()
            .with_linked_module("bench_reader::storage_reader", reader_code.as_str())?
            .compile_tx_script(&script_code)?;

        let tx_request = TransactionRequestBuilder::new().custom_script(tx_script).build()?;

        // Execute first (not measured)
        let tx_result = client.execute_transaction(account_id, tx_request).await?;

        // Measure proving time only
        let (proven_tx, duration) =
            measure_time_async(|| async { client.prove_transaction(&tx_result).await }).await;

        if let Ok(proven) = proven_tx {
            result.add_iteration(duration);
            // Record proof size
            let proof_bytes = proven.proof().to_bytes();
            result = result.with_output_size(proof_bytes.len());
        } else {
            // If proving fails, still record the time
            result.add_iteration(duration);
        }
    }

    result = result.with_metadata(format!(
        "Transaction proving, {total_entries} storage reads from active account"
    ));

    Ok(result)
}

/// Benchmarks full transaction (execute + prove + submit)
async fn benchmark_tx_full(
    config: &BenchConfig,
    account_id: AccountId,
    storage_maps: &[StorageMapInfo],
    secret_key: &AuthSecretKey,
) -> anyhow::Result<BenchmarkResult> {
    let total_entries: usize = storage_maps.iter().map(|m| m.keys.len()).sum();
    let bench_name = format!("full ({total_entries} storage reads)");

    let mut result = BenchmarkResult::new(&bench_name);

    for i in 0..config.iterations {
        let (mut client, keystore, _) =
            create_benchmark_client(config, &format!("tx-full-iter-{i}")).await?;
        client.sync_state().await?;

        // Import the account and add the signing key
        client.import_account_by_id(account_id).await?;
        keystore.add_key(secret_key)?;

        let script_code = generate_storage_read_script(storage_maps);
        let reader_code = generate_reader_component_code(storage_maps.len());
        let tx_script = client
            .code_builder()
            .with_linked_module("bench_reader::storage_reader", reader_code.as_str())?
            .compile_tx_script(&script_code)?;

        let tx_request = TransactionRequestBuilder::new().custom_script(tx_script).build()?;

        // Measure full transaction time (execute + prove + submit)
        let (_, duration) = measure_time_async(|| async {
            client.submit_new_transaction(account_id, tx_request).await
        })
        .await;

        result.add_iteration(duration);
    }

    result = result.with_metadata(format!(
        "Full transaction (execute + prove + submit), {total_entries} storage reads"
    ));

    Ok(result)
}
