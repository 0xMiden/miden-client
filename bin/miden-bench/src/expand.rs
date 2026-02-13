#![allow(clippy::cast_possible_truncation, clippy::cast_lossless)]

use std::sync::Arc;
use std::time::Instant;

use miden_client::account::AccountId;
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::rpc::{Endpoint, GrpcClient};
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::{Client, DebugMode, Felt, Serializable};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use rand::Rng;

use crate::deploy::wait_for_block_advancement;
use crate::generators::{random_word, slot_rng};
use crate::masm::{generate_expansion_component_code, generate_expansion_tx_script};
use crate::report::format_size;

/// Maximum entries per expansion transaction. Determined empirically to stay
/// within Miden VM instruction limits per transaction.
const MAX_ENTRIES_PER_EXPANSION_TX: usize = 280;

/// Generates deterministic storage map entries for the given map index and range.
///
/// The key/value generation scheme matches `create_large_storage_slot` in generators:
/// - Keys are derived from `seed.wrapping_mul(1000).wrapping_add(entry_index)`
/// - Values are drawn from a `ChaCha20` RNG seeded by `map_idx`
/// - The RNG is advanced past `offset` entries so values are position-stable
fn generate_entries(map_idx: usize, offset: usize, count: usize) -> Vec<([Felt; 4], [Felt; 4])> {
    let seed = map_idx as u32;
    let mut rng = slot_rng(seed);

    // Advance the RNG past entries [0..offset) so we produce the same values
    // regardless of which offset we start from.
    for _ in 0..offset {
        random_word(&mut rng);
    }

    (0..count)
        .map(|i| {
            let key_val = seed.wrapping_mul(1000).wrapping_add((offset + i) as u32);
            let key = [Felt::new(key_val as u64); 4];
            let value = random_word(&mut rng);
            (key, value)
        })
        .collect()
}

/// Detects the number of bench map slots in an imported account by counting
/// storage slots whose names match `miden::bench::map_slot_*`.
async fn detect_num_maps(
    client: &Client<FilesystemKeyStore>,
    account_id: AccountId,
) -> anyhow::Result<usize> {
    let storage = client.get_account_storage(account_id).await?;
    let count = storage
        .slots()
        .iter()
        .filter(|slot| slot.name().to_string().starts_with("miden::bench::map_slot_"))
        .count();
    Ok(count)
}

/// Creates a client for the expand command using the persistent store directory.
async fn create_expand_client(
    endpoint: &Endpoint,
    store_path: &str,
) -> anyhow::Result<(Client<FilesystemKeyStore>, std::path::PathBuf)> {
    let data_dir = std::path::PathBuf::from(store_path);
    std::fs::create_dir_all(&data_dir)?;
    let sqlite_path = data_dir.join("store.sqlite3");
    let keystore_path = data_dir.join("keystore");
    std::fs::create_dir_all(&keystore_path)?;

    println!("Store directory: {}", data_dir.display());

    let mut rng = rand::rng();
    let coin_seed: [u64; 4] = rng.random();
    let rng_coin = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

    let client: Client<FilesystemKeyStore> = ClientBuilder::new()
        .rpc(Arc::new(GrpcClient::new(endpoint, 30_000)))
        .rng(Box::new(rng_coin))
        .sqlite_store(sqlite_path)
        .filesystem_keystore(keystore_path.to_str().expect("keystore path should be valid UTF-8"))?
        .in_debug_mode(DebugMode::Disabled)
        .tx_discard_delta(None)
        .build()
        .await?;

    Ok((client, data_dir))
}

/// Submits expansion transactions in batches, waiting for blocks between each batch.
async fn submit_expansion_batches(
    client: &mut Client<FilesystemKeyStore>,
    account_id: AccountId,
    map_idx: usize,
    offset: usize,
    entries: &[([Felt; 4], [Felt; 4])],
    expansion_code: &str,
) -> anyhow::Result<()> {
    let total_batches = entries.len().div_ceil(MAX_ENTRIES_PER_EXPANSION_TX);

    for (batch_idx, batch_entries) in entries.chunks(MAX_ENTRIES_PER_EXPANSION_TX).enumerate() {
        let batch_offset = offset + batch_idx * MAX_ENTRIES_PER_EXPANSION_TX;
        let batch_end = batch_offset + batch_entries.len();
        let t = Instant::now();

        let script_code = generate_expansion_tx_script(map_idx, batch_entries);

        let tx_script = client
            .code_builder()
            .with_linked_module("expander::storage_expander", expansion_code)?
            .compile_tx_script(&script_code)?;

        let tx_request = TransactionRequestBuilder::new().custom_script(tx_script).build()?;

        let tx_result = client.execute_transaction(account_id, tx_request).await?;
        let proven_tx = client.prove_transaction(&tx_result).await?;
        let tx_size = proven_tx.to_bytes().len();
        let submission_height = client.submit_proven_transaction(proven_tx, &tx_result).await?;
        client.apply_transaction(&tx_result, submission_height).await?;

        println!(
            "  Batch {}/{total_batches}: entries [{batch_offset}..{batch_end}] in {:.2?} (tx size: {})",
            batch_idx + 1,
            t.elapsed(),
            format_size(tx_size)
        );

        // Wait for blocks between batches so the node processes each transaction
        if batch_idx < total_batches - 1 {
            for _ in 0..3 {
                wait_for_block_advancement(client).await?;
            }
            client.sync_state().await?;
        }
    }

    Ok(())
}

/// Fills entries into a specific storage map of a deployed account.
///
/// The account must have been deployed via the `deploy` command, which creates
/// empty storage maps with expansion procedures already installed. This function
/// submits transactions that call those procedures to insert entries.
///
/// The signing key is expected to be present in the persistent keystore
/// (written by the `deploy` command).
pub async fn expand_storage(
    endpoint: &Endpoint,
    account_id_str: &str,
    map_idx: usize,
    offset: usize,
    count: usize,
    store_path: &str,
) -> anyhow::Result<()> {
    let account_id = AccountId::from_hex(account_id_str)?;

    println!("Network: {endpoint}");
    println!(
        "Expanding map {map_idx} of account {account_id}: entries [{offset}..{}] ({count} entries)",
        offset + count
    );
    println!();

    let total_t = Instant::now();

    let (mut client, _data_dir) = create_expand_client(endpoint, store_path).await?;

    // Sync and import account
    println!("Connecting to node at {endpoint}...");
    client.sync_state().await?;
    let chain_height = client.get_sync_height().await?;
    println!("Connected successfully. Chain height: {chain_height}");

    let has_account = client.get_account_storage(account_id).await.is_ok();
    if has_account {
        println!("Using account {account_id} from persistent store");
    } else {
        println!("Importing account {account_id}...");
        client.import_account_by_id(account_id).await?;
    }

    // Detect number of maps from the imported account
    let num_maps = detect_num_maps(&client, account_id).await?;
    if num_maps == 0 {
        anyhow::bail!("Account has no bench storage map slots");
    }
    if map_idx >= num_maps {
        anyhow::bail!(
            "Map index {map_idx} out of range (account has {num_maps} maps, indices 0..{})",
            num_maps - 1
        );
    }
    println!("Detected {num_maps} storage map(s) in account");
    println!();

    let entries = generate_entries(map_idx, offset, count);
    let expansion_code = generate_expansion_component_code(num_maps);

    submit_expansion_batches(&mut client, account_id, map_idx, offset, &entries, &expansion_code)
        .await?;

    println!();
    println!("Total expand time: {:.2?}", total_t.elapsed());

    Ok(())
}
