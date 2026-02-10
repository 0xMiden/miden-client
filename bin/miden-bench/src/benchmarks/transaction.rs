use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use miden_client::account::{AccountId, StorageSlotContent};
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::rpc::GrpcClient;
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::{Client, DebugMode, Felt, Serializable, Word};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_protocol::account::auth::AuthSecretKey;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use crate::config::BenchConfig;
use crate::generators::{SlotDescriptor, generate_reader_component_code};
use crate::metrics::{BenchmarkResult, measure_time_async};
use crate::report::format_size;

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

// DATA MODEL
// ================================================================================================

/// Information about a bench map storage slot extracted from the account.
#[derive(Clone, Debug)]
struct StorageSlotInfo {
    name: String,
    keys: Vec<Word>,
}

impl StorageSlotInfo {
    fn num_reads(&self) -> usize {
        self.keys.len()
    }
}

/// A single map entry read operation to be performed in a transaction.
#[derive(Clone, Debug)]
struct ReadOp {
    /// Index into the `slot_infos` array (matches the reader procedure index).
    slot_idx: usize,
    key: Word,
}

// ORCHESTRATOR
// ================================================================================================

/// Runs transaction benchmarks (requires a running node).
///
/// The benchmark uses the specified account as the native account executing transactions.
/// Each transaction reads storage entries (both value and map slots) from the account's
/// own storage. Slot types and entries are auto-detected from the imported account storage.
///
/// When `max_reads_per_tx` is provided and total reads exceed that limit, reads are
/// split across multiple transactions per benchmark iteration. Each iteration's reported
/// time is the sum across all transactions.
pub async fn run_transaction_benchmarks(
    config: &BenchConfig,
    account_id_str: String,
    seed: Option<[u8; 32]>,
    max_reads_per_tx: Option<usize>,
) -> anyhow::Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    // Parse the account ID
    let account_id = AccountId::from_hex(&account_id_str)?;

    // First, try to connect to the node and fetch the account
    println!("Connecting to node at {}...", config.network);

    let (mut client, keystore, _temp_dir) = match create_benchmark_client(config, "tx-init").await {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to connect to node: {e}");
            println!("Skipping transaction benchmarks (requires a running Miden node).");
            results
                .push(BenchmarkResult::new("transaction/connection_failed").with_metadata(
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

    // Detect storage slots and build slot info.
    let storage = client.get_account_storage(account_id).await?;
    let slot_infos: Vec<StorageSlotInfo> = build_slot_infos_from_storage(&storage);

    let total_reads: usize = slot_infos.iter().map(StorageSlotInfo::num_reads).sum();
    if total_reads == 0 {
        anyhow::bail!("Account has no non-empty storage slots to benchmark.");
    }

    let entries_per_map: Vec<usize> = slot_infos.iter().map(|s| s.keys.len()).collect();

    // Flatten into individual read operations and chunk
    let all_ops = flatten_read_ops(&slot_infos);
    let chunks = chunk_read_ops(&all_ops, max_reads_per_tx.unwrap_or(total_reads));
    let num_chunks = chunks.len();

    // Build a slot summary that shows per-map entry counts
    let num_maps = entries_per_map.len();
    let slot_summary = if entries_per_map.windows(2).all(|w| w[0] == w[1]) {
        format!("{num_maps} map ({} entries each)", entries_per_map[0])
    } else {
        let counts = entries_per_map.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
        format!("{num_maps} map (entries: [{counts}])")
    };

    if num_chunks > 1 {
        let reads_per_tx = max_reads_per_tx.unwrap_or(total_reads);
        println!(
            "Slots: {slot_summary}. \
             Total reads: {total_reads} ({num_chunks} txs, up to {reads_per_tx} each)"
        );
    } else {
        println!("Slots: {slot_summary}. Total reads: {total_reads}");
    }

    // Regenerate the signing key from the deployment seed (if provided)
    let secret_key =
        seed.map(|s| AuthSecretKey::new_falcon512_rpo_with_rng(&mut ChaCha20Rng::from_seed(s)));

    // Measure proven transaction size upfront (execute + prove one tx)
    if let Some(ref sk) = secret_key {
        keystore.add_key(sk)?;
        let tx_request = build_chunk_tx_request(&client, &chunks[0], &slot_infos)?;
        let tx_result = client.execute_transaction(account_id, tx_request).await?;
        let proven_tx = client.prove_transaction(&tx_result).await?;
        let tx_size = proven_tx.to_bytes().len();
        println!("Proven transaction size: {}", format_size(tx_size));
    }
    println!();

    // Benchmark 1: Transaction execution time (without proving)
    println!("Benchmarking transaction execution...");
    let execution_result =
        benchmark_tx_execution(config, account_id, &chunks, &slot_infos, secret_key.as_ref())
            .await?;
    results.push(execution_result);

    // Benchmarks 2 & 3 require the signing key for proving and submission
    if let Some(ref sk) = secret_key {
        // Benchmark 2: Transaction proving time
        println!("Benchmarking transaction proving...");
        let proving_result =
            benchmark_tx_proving(config, account_id, &chunks, &slot_infos, sk).await?;
        results.push(proving_result);

        // Benchmark 3: Full transaction (execute + prove + submit)
        println!("Benchmarking full transaction...");
        let full_result =
            Box::pin(benchmark_tx_full(config, account_id, &chunks, &slot_infos, sk)).await?;
        results.push(full_result);
    } else {
        println!("Skipping proving and submission benchmarks (no seed provided).");
    }

    Ok(results)
}

// SLOT DETECTION
// ================================================================================================

/// Builds slot infos from the imported account storage.
///
/// Only includes bench map slots (`miden::bench::map_slot_N`), returned in canonical
/// index order (0, 1, 2, ...) to match the account's reader component. This ensures
/// the dynamically-linked reader procedures have matching MAST roots.
fn build_slot_infos_from_storage(
    storage: &miden_client::account::AccountStorage,
) -> Vec<StorageSlotInfo> {
    // Collect bench map slots with their parsed indices
    let mut indexed: Vec<(usize, Vec<Word>)> = storage
        .slots()
        .iter()
        .filter_map(|slot| {
            let name = slot.name().to_string();
            let idx = name.strip_prefix("miden::bench::map_slot_")?.parse::<usize>().ok()?;
            if let StorageSlotContent::Map(map) = slot.content() {
                let keys: Vec<Word> = map.entries().map(|(k, _v)| *k).collect();
                Some((idx, keys))
            } else {
                None
            }
        })
        .collect();

    if indexed.is_empty() {
        return Vec::new();
    }

    indexed.sort_by_key(|(idx, _)| *idx);
    let max_idx = indexed.last().unwrap().0;

    // Build contiguous slot_infos [0..=max_idx] so procedure indices match the account's
    // reader component. Slots missing from storage get empty key lists (no reads generated).
    let mut keys_by_idx = vec![Vec::new(); max_idx + 1];
    for (idx, keys) in indexed {
        keys_by_idx[idx] = keys;
    }

    keys_by_idx
        .into_iter()
        .enumerate()
        .map(|(i, keys)| StorageSlotInfo {
            name: format!("miden::bench::map_slot_{i}"),
            keys,
        })
        .collect()
}

// READ OPS & CHUNKING
// ================================================================================================

/// Flattens slot infos into individual read operations.
fn flatten_read_ops(slot_infos: &[StorageSlotInfo]) -> Vec<ReadOp> {
    slot_infos
        .iter()
        .enumerate()
        .flat_map(|(idx, info)| info.keys.iter().map(move |k| ReadOp { slot_idx: idx, key: *k }))
        .collect()
}

/// Splits read operations into chunks of at most `max_reads` each.
fn chunk_read_ops(all_ops: &[ReadOp], max_reads: usize) -> Vec<Vec<ReadOp>> {
    if all_ops.len() <= max_reads {
        return vec![all_ops.to_vec()];
    }
    all_ops.chunks(max_reads).map(<[ReadOp]>::to_vec).collect()
}

// MASM GENERATION
// ================================================================================================

/// Maximum read ops per code block to stay within the Miden parser's `u16::MAX` instruction
/// limit. Map entries generate 6 ops each (push key + call + 4 dropw). Using 7000 as the
/// conservative limit.
const MAX_OPS_PER_BLOCK: usize = 7_000;

/// Writes the MASM instructions for a single map entry read (push key, call reader, drop result).
fn write_read_op_instructions(script: &mut String, op: &ReadOp) {
    // Push key (4 felts)
    writeln!(
        script,
        "    push.{}.{}.{}.{}",
        op.key[3].as_int(),
        op.key[2].as_int(),
        op.key[1].as_int(),
        op.key[0].as_int()
    )
    .expect("write to string should not fail");

    // Call the account's reader procedure for this storage map slot.
    // Stack input: [KEY]
    // Stack output via call frame: [VALUE, pad(12)] = 16 elements
    writeln!(script, "    call.storage_reader::get_map_item_slot_{}", op.slot_idx)
        .expect("write to string should not fail");

    // Drop the result (we just want to measure read time)
    script.push_str("    dropw dropw dropw dropw\n");
}

/// Generates a MASM script that reads storage entries from the active account.
///
/// Uses `call` to invoke account reader procedures rather than directly `exec`-ing
/// kernel syscalls. The kernel's `authenticate_account_origin` requires the caller
/// to be an account procedure.
///
/// When the total number of read ops exceeds [`MAX_OPS_PER_BLOCK`], the script is
/// split into `repeat.1 ... end` blocks to stay within the Miden parser's per-block
/// instruction limit.
fn generate_storage_read_script(read_ops: &[ReadOp]) -> String {
    let mut script = String::from(
        "use bench_reader::storage_reader
begin
",
    );

    if read_ops.len() <= MAX_OPS_PER_BLOCK {
        for op in read_ops {
            write_read_op_instructions(&mut script, op);
        }
    } else {
        // Split into repeat.1 blocks to create new block scopes, each with its own
        // independent instruction limit. repeat.1 compiles to a single pass (no overhead).
        for chunk in read_ops.chunks(MAX_OPS_PER_BLOCK) {
            script.push_str("    repeat.1\n");
            for op in chunk {
                write_read_op_instructions(&mut script, op);
            }
            script.push_str("    end\n");
        }
    }

    script.push_str("end\n");
    script
}

/// Compiles and builds a transaction request for a chunk of read operations.
fn build_chunk_tx_request(
    client: &Client<FilesystemKeyStore>,
    chunk: &[ReadOp],
    slot_infos: &[StorageSlotInfo],
) -> anyhow::Result<miden_client::transaction::TransactionRequest> {
    let script_code = generate_storage_read_script(chunk);

    let descriptors: Vec<SlotDescriptor> = slot_infos
        .iter()
        .map(|info| SlotDescriptor { name: info.name.clone(), is_map: true })
        .collect();
    let reader_code = generate_reader_component_code(&descriptors);

    let tx_script = client
        .code_builder()
        .with_linked_module("bench_reader::storage_reader", reader_code.as_str())?
        .compile_tx_script(&script_code)?;
    Ok(TransactionRequestBuilder::new().custom_script(tx_script).build()?)
}

// HELPERS
// ================================================================================================

/// Waits for the chain height to advance by one block.
async fn wait_for_block_advancement(client: &mut Client<FilesystemKeyStore>) -> anyhow::Result<()> {
    let initial_height = client.get_sync_height().await?;
    let target_height = initial_height.as_u32() + 1;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        client.sync_state().await?;
        let current_height = client.get_sync_height().await?;
        if current_height.as_u32() >= target_height {
            break;
        }
    }

    Ok(())
}

/// Formats the benchmark name with optional chunk count.
fn bench_name(phase: &str, total_reads: usize, num_chunks: usize) -> String {
    if num_chunks > 1 {
        format!("{phase} ({total_reads} storage reads, {num_chunks} txs)")
    } else {
        format!("{phase} ({total_reads} storage reads)")
    }
}

// BENCHMARKS
// ================================================================================================

/// Benchmarks transaction execution time (reading storage from active account).
///
/// When multiple chunks are provided, each iteration executes all chunks sequentially
/// and reports the total execution time.
async fn benchmark_tx_execution(
    config: &BenchConfig,
    account_id: AccountId,
    chunks: &[Vec<ReadOp>],
    slot_infos: &[StorageSlotInfo],
    secret_key: Option<&AuthSecretKey>,
) -> anyhow::Result<BenchmarkResult> {
    let total_reads: usize = chunks.iter().map(Vec::len).sum();
    let num_chunks = chunks.len();

    let mut result = BenchmarkResult::new(bench_name("execute", total_reads, num_chunks));

    for i in 0..config.iterations {
        let iter_t = Instant::now();

        let (mut client, keystore, _) =
            create_benchmark_client(config, &format!("tx-exec-iter-{i}")).await?;
        client.sync_state().await?;

        // Import the account and add the signing key (if available)
        client.import_account_by_id(account_id).await?;
        if let Some(sk) = secret_key {
            keystore.add_key(sk)?;
        }

        let mut total_duration = Duration::ZERO;

        for chunk in chunks {
            let tx_request = build_chunk_tx_request(&client, chunk, slot_infos)?;

            let (_, duration) = measure_time_async(|| async {
                client.execute_transaction(account_id, tx_request).await
            })
            .await;

            total_duration += duration;
        }

        result.add_iteration(total_duration);
        println!(
            "  Iteration {}/{}: {:.2?} (total: {:.2?})",
            i + 1,
            config.iterations,
            total_duration,
            iter_t.elapsed()
        );
    }

    result = result.with_metadata(format!(
        "Transaction execution (no proving), {total_reads} storage reads from active account{}",
        if num_chunks > 1 {
            format!(" across {num_chunks} transactions")
        } else {
            String::new()
        }
    ));

    Ok(result)
}

/// Benchmarks transaction proving time.
///
/// When multiple chunks are provided, each iteration executes and proves all chunks
/// sequentially, reporting the total proving time (execution time is excluded).
async fn benchmark_tx_proving(
    config: &BenchConfig,
    account_id: AccountId,
    chunks: &[Vec<ReadOp>],
    slot_infos: &[StorageSlotInfo],
    secret_key: &AuthSecretKey,
) -> anyhow::Result<BenchmarkResult> {
    let total_reads: usize = chunks.iter().map(Vec::len).sum();
    let num_chunks = chunks.len();

    let mut result = BenchmarkResult::new(bench_name("prove", total_reads, num_chunks));

    for i in 0..config.iterations {
        let iter_t = Instant::now();

        let (mut client, keystore, _) =
            create_benchmark_client(config, &format!("tx-prove-iter-{i}")).await?;
        client.sync_state().await?;

        // Import the account and add the signing key
        client.import_account_by_id(account_id).await?;
        keystore.add_key(secret_key)?;

        let mut total_duration = Duration::ZERO;

        for chunk in chunks {
            let tx_request = build_chunk_tx_request(&client, chunk, slot_infos)?;

            // Execute first (not measured)
            let tx_result = client.execute_transaction(account_id, tx_request).await?;

            // Measure proving time only
            let (proven_tx, duration) =
                measure_time_async(|| async { client.prove_transaction(&tx_result).await }).await;

            total_duration += duration;

            if let Ok(proven) = proven_tx {
                let proof_bytes = proven.proof().to_bytes();
                result = result.with_output_size(proof_bytes.len());
            }
        }

        result.add_iteration(total_duration);
        println!(
            "  Iteration {}/{}: {:.2?} (total: {:.2?})",
            i + 1,
            config.iterations,
            total_duration,
            iter_t.elapsed()
        );
    }

    result = result.with_metadata(format!(
        "Transaction proving, {total_reads} storage reads from active account{}",
        if num_chunks > 1 {
            format!(" across {num_chunks} transactions")
        } else {
            String::new()
        }
    ));

    Ok(result)
}

/// Benchmarks full transaction (execute + prove + submit).
///
/// When multiple chunks are provided, each iteration submits all chunks sequentially
/// with block advancement waits between submissions (needed for nonce updates).
/// Reports the total time including waits.
async fn benchmark_tx_full(
    config: &BenchConfig,
    account_id: AccountId,
    chunks: &[Vec<ReadOp>],
    slot_infos: &[StorageSlotInfo],
    secret_key: &AuthSecretKey,
) -> anyhow::Result<BenchmarkResult> {
    let total_reads: usize = chunks.iter().map(Vec::len).sum();
    let num_chunks = chunks.len();

    let mut result = BenchmarkResult::new(bench_name("full", total_reads, num_chunks));

    for i in 0..config.iterations {
        let iter_t = Instant::now();
        let mut total_duration = Duration::ZERO;

        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            // Each chunk submission needs a fresh client with up-to-date state,
            // because the previous submission changes the account nonce on the network.
            let (mut client, keystore, _) =
                create_benchmark_client(config, &format!("tx-full-iter-{i}-chunk-{chunk_idx}"))
                    .await?;
            client.sync_state().await?;
            client.import_account_by_id(account_id).await?;
            keystore.add_key(secret_key)?;

            let tx_request = build_chunk_tx_request(&client, chunk, slot_infos)?;

            // Measure full transaction time (execute + prove + submit)
            let (_, duration) = measure_time_async(|| async {
                client.submit_new_transaction(account_id, tx_request).await
            })
            .await;

            total_duration += duration;

            // Wait for the block to advance before the next chunk so the
            // node has the updated nonce when we submit the next transaction.
            if chunk_idx < num_chunks - 1 {
                wait_for_block_advancement(&mut client).await?;
            }
        }

        result.add_iteration(total_duration);
        println!(
            "  Iteration {}/{}: {:.2?} (total: {:.2?})",
            i + 1,
            config.iterations,
            total_duration,
            iter_t.elapsed()
        );
    }

    result = result.with_metadata(format!(
        "Full transaction (execute + prove + submit), {total_reads} storage reads{}",
        if num_chunks > 1 {
            format!(" across {num_chunks} transactions")
        } else {
            String::new()
        }
    ));

    Ok(result)
}
