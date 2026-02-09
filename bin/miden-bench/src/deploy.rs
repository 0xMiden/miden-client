#![allow(clippy::cast_possible_truncation, clippy::cast_lossless)]

use std::fmt::Write;
use std::sync::Arc;
use std::time::Instant;

use miden_client::account::{AccountId, StorageMap, StorageSlot, StorageSlotName};
use miden_client::assembly::{CodeBuilder, DefaultSourceManager, Module, ModuleKind, Path};
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::rpc::{Endpoint, GrpcClient};
use miden_client::transaction::{TransactionKernel, TransactionRequestBuilder};
use miden_client::{Client, DebugMode, Felt, Serializable};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_protocol::account::auth::AuthSecretKey;
use miden_protocol::account::{AccountBuilder, AccountComponent, AccountStorageMode, AccountType};
use miden_standards::account::auth::AuthFalcon512Rpo;
use miden_standards::account::components::basic_wallet_library;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use crate::generators::{
    LargeAccountConfig,
    SlotDescriptor,
    generate_reader_component_code,
    random_word,
    slot_rng,
};
use crate::report::format_size;
use crate::spinner::with_spinner;

/// Maximum storage entries for a single-transaction deployment.
/// Proven transactions with more entries exceed the gRPC message size limit (~4MB).
/// Accounts above this threshold use two-phase deployment (empty maps + expansion).
const MAX_ENTRIES_SINGLE_DEPLOY: usize = 200;

/// Maximum entries to set per expansion transaction.
const ENTRIES_PER_EXPANSION_TX: usize = 1000;

/// Waits for the chain height to advance, ensuring transaction is in a block
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

/// Map entries to be added after initial deployment via expansion transactions
struct MapEntries {
    entries: Vec<([Felt; 4], [Felt; 4])>,
}

/// Creates an account with empty storage maps and expansion procedures (for two-phase deployment)
fn create_minimal_account(
    config: &LargeAccountConfig,
) -> anyhow::Result<(
    miden_protocol::account::Account,
    miden_protocol::account::auth::AuthSecretKey,
    Vec<MapEntries>,
)> {
    let sk = AuthSecretKey::new_falcon512_rpo_with_rng(&mut ChaCha20Rng::from_seed(config.seed));

    // Create empty storage map slots and collect deferred entries
    let mut storage_slots = Vec::new();
    let mut deferred_entries = Vec::new();

    for i in 0..config.num_map_slots {
        let slot_name = format!("miden::bench::map_slot_{i}");

        // Use the same key scheme as the single-tx path: key_val = slot_index * 1000 + j
        let seed = i as u32;
        let mut rng = slot_rng(seed);

        // Generate the initial entry (j=0) first so the RNG stays in sync with
        // the single-tx path in `create_large_storage_slot`.
        let initial_key_val = seed.wrapping_mul(1000); // j=0
        let initial_key = [Felt::new(initial_key_val as u64); 4];
        let initial_value = random_word(&mut rng);

        // Generate the entries that will be added later via expansion transactions (j=1..N-1)
        let entries: Vec<([Felt; 4], [Felt; 4])> = (1..config.num_storage_map_entries as u32)
            .map(|j| {
                let key_val = seed.wrapping_mul(1000).wrapping_add(j);
                let key = [Felt::new(key_val as u64); 4];
                let value = random_word(&mut rng);
                (key, value)
            })
            .collect();

        deferred_entries.push(MapEntries { entries });

        // Create storage map with the initial entry (j=0).
        // Value is random (non-zero with overwhelming probability) — the SMT treats
        // zero values as deletions, but random_word guarantees non-zero.
        let mut initial_map = StorageMap::new();
        initial_map
            .insert(initial_key.into(), initial_value.into())
            .expect("inserting initial entry should succeed");

        storage_slots.push(StorageSlot::with_map(
            StorageSlotName::new(slot_name.as_str()).expect("slot name should be valid"),
            initial_map,
        ));
    }

    // Create expansion component code that has procedures to set map items for each slot
    let expansion_code = generate_expansion_component_code(config.num_map_slots);

    // Compile the expansion component
    let expansion_component_code = CodeBuilder::default()
        .compile_component_code("miden::bench::storage_expander", &expansion_code)
        .map_err(|e| anyhow::anyhow!("Failed to compile expansion component: {e}"))?;

    // Create the expansion component with storage slots (provides set_item_slot_N procedures)
    let expansion_component = AccountComponent::new(expansion_component_code, storage_slots)
        .map_err(|e| anyhow::anyhow!("Failed to create expansion component: {e}"))?
        .with_supports_all_types();

    // Reader component: provides get_map_item_slot_N procedures for transaction benchmarks.
    // No storage slots needed — the procedures access slots by name from any component.
    let descriptors: Vec<SlotDescriptor> = (0..config.num_map_slots)
        .map(|i| SlotDescriptor {
            name: format!("miden::bench::map_slot_{i}"),
            is_map: true,
        })
        .collect();
    let reader_code = generate_reader_component_code(&descriptors);
    let reader_component_code = CodeBuilder::default()
        .compile_component_code("miden::bench::storage_reader", &reader_code)
        .map_err(|e| anyhow::anyhow!("Failed to compile reader component: {e}"))?;
    let reader_component = AccountComponent::new(reader_component_code, vec![])
        .map_err(|e| anyhow::anyhow!("Failed to create reader component: {e}"))?
        .with_supports_all_types();

    // Basic wallet for normal operations
    let wallet_component = AccountComponent::new(basic_wallet_library(), vec![])
        .expect("basic wallet component should satisfy account component requirements")
        .with_supports_all_types();

    let account = AccountBuilder::new(config.seed)
        .with_auth_component(AuthFalcon512Rpo::new(sk.public_key().to_commitment()))
        .account_type(AccountType::RegularAccountUpdatableCode)
        .with_component(wallet_component)
        .with_component(expansion_component)
        .with_component(reader_component)
        .storage_mode(AccountStorageMode::Public)
        .build()?;

    Ok((account, sk, deferred_entries))
}

/// Generates MASM code for an account component that can set items in multiple storage maps.
/// Creates a procedure `set_item_slot_N` for each slot that receives key/value from the stack.
fn generate_expansion_component_code(num_slots: usize) -> String {
    let mut code = String::new();

    for i in 0..num_slots {
        let slot_name = format!("miden::bench::map_slot_{i}");
        write!(
            code,
            r#"const MAP_SLOT_{i} = word("{slot_name}")

# Sets an item in storage slot {i}.
# Stack input:  [KEY, VALUE, ...]
# Stack output: [...]
pub proc set_item_slot_{i}
    push.MAP_SLOT_{i}[0..2]
    # Stack: [slot_suffix, slot_prefix, KEY, VALUE, ...]

    exec.::miden::protocol::native_account::set_map_item
    # Stack: [OLD_VALUE, ...]

    dropw
end

"#
        )
        .expect("writing to String should not fail");
    }

    code
}

/// Expands storage maps by submitting batched transactions.
/// Processes one map at a time, filling entries sequentially before moving to the next map.
async fn expand_storage_maps(
    client: &mut Client<FilesystemKeyStore>,
    account_id: AccountId,
    map_entries: Vec<MapEntries>,
    num_slots: usize,
) -> anyhow::Result<()> {
    let total_entries: usize = map_entries.iter().map(|m| m.entries.len()).sum();
    if total_entries == 0 {
        return Ok(());
    }

    let total_batches: usize = map_entries
        .iter()
        .map(|m| m.entries.len().div_ceil(ENTRIES_PER_EXPANSION_TX))
        .sum();

    let mut processed = 0;
    let mut batch_num = 0;

    // Generate expansion component code (same as what's in the account)
    let expansion_code = generate_expansion_component_code(num_slots);

    // Create the library for dynamic linking (once, reuse for all transactions)
    let assembler = TransactionKernel::assembler();
    let source_manager = Arc::new(DefaultSourceManager::default());
    let module = Module::parser(ModuleKind::Library)
        .parse_str(Path::new("expander::storage_expander"), &expansion_code, source_manager.clone())
        .map_err(|e| anyhow::anyhow!("Failed to parse expansion module: {e}"))?;
    let library = assembler
        .assemble_library([module])
        .map_err(|e| anyhow::anyhow!("Failed to assemble library: {e}"))?;

    // Process each map sequentially, batching entries within each map
    for (slot_idx, map) in map_entries.iter().enumerate() {
        let entries = &map.entries;
        let mut start = 0;

        while start < entries.len() {
            let batch_t = Instant::now();
            let count = ENTRIES_PER_EXPANSION_TX.min(entries.len() - start);
            let batch = &entries[start..start + count];

            // Generate transaction script targeting a single map
            let script_code = generate_single_map_expansion_tx_script(slot_idx, batch);

            // Compile transaction script with dynamic linking
            let tx_script = CodeBuilder::new()
                .with_dynamically_linked_library(library.clone())
                .map_err(|e| anyhow::anyhow!("Failed to link library: {e}"))?
                .compile_tx_script(&script_code)
                .map_err(|e| anyhow::anyhow!("Failed to compile tx script: {e}"))?;

            // Build, prove, and submit transaction
            let tx_request = TransactionRequestBuilder::new().custom_script(tx_script).build()?;

            let tx_result = client.execute_transaction(account_id, tx_request).await?;
            let proven_tx = client.prove_transaction(&tx_result).await?;
            let tx_size = proven_tx.to_bytes().len();
            let submission_height =
                client.submit_proven_transaction(proven_tx, &tx_result).await?;
            client.apply_transaction(&tx_result, submission_height).await?;

            // Wait for 3 blocks to ensure storage is properly indexed
            for _ in 0..3 {
                wait_for_block_advancement(&mut *client).await?;
            }

            batch_num += 1;
            processed += count;
            start += count;
            println!(
                "  Batch {batch_num}/{total_batches}: map {slot_idx} entries [{start_off}..{end}] \
                 ({processed}/{total_entries} total) in {:.2?} (tx size: {})",
                batch_t.elapsed(),
                format_size(tx_size),
                start_off = start - count,
                end = start,
            );
        }
    }

    Ok(())
}

/// Generates a transaction script that sets map items in a single storage slot.
/// Key and value are passed via the stack (not memory), because `call` switches
/// to the account's memory context which is separate from the tx script's memory.
fn generate_single_map_expansion_tx_script(
    slot_idx: usize,
    entries: &[([Felt; 4], [Felt; 4])],
) -> String {
    let mut script = String::from("use expander::storage_expander\n\nbegin\n");
    let procedure_name = format!("set_item_slot_{slot_idx}");

    for (key, value) in entries {
        write!(
            script,
            r"    # Push value then key onto stack (key on top for the procedure)
    push.{val3}.{val2}.{val1}.{val0}
    push.{key3}.{key2}.{key1}.{key0}
    call.storage_expander::{procedure_name}
    dropw dropw dropw dropw

",
            key3 = key[3].as_int(),
            key2 = key[2].as_int(),
            key1 = key[1].as_int(),
            key0 = key[0].as_int(),
            val3 = value[3].as_int(),
            val2 = value[2].as_int(),
            val1 = value[1].as_int(),
            val0 = value[0].as_int(),
        )
        .expect("writing to String should not fail");
    }

    script.push_str("end\n");
    script
}

/// Creates and deploys a public wallet with configurable storage to the network.
/// Returns the account ID and the seed used for key generation (needed for signing transactions).
///
/// For accounts with more than `MAX_ENTRIES_SINGLE_DEPLOY` entries, uses a two-phase deployment:
/// 1. Deploy account with empty storage maps
/// 2. Expand storage through batched transactions
#[allow(clippy::too_many_lines)]
pub async fn deploy_account(
    endpoint: &Endpoint,
    maps: usize,
    entries_per_map: usize,
) -> anyhow::Result<(AccountId, [u8; 32])> {
    let account_config = LargeAccountConfig::new(maps, entries_per_map);
    let total_entries = account_config.total_entries();
    let needs_expansion = total_entries > MAX_ENTRIES_SINGLE_DEPLOY;

    println!("Network: {endpoint}");
    println!("Storage maps: {maps}");
    println!("Entries per map: {entries_per_map}");
    println!("Total storage entries: {total_entries}");

    if needs_expansion {
        println!("Deployment mode: two-phase (entries exceed {MAX_ENTRIES_SINGLE_DEPLOY} limit)");
    } else {
        println!("Deployment mode: single transaction");
    }

    println!();

    let total_t = Instant::now();

    // Create temp directory for client data
    let temp_dir =
        std::env::temp_dir().join(format!("miden-bench-deploy-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    let store_path = temp_dir.join("store.sqlite3");
    let keystore_path = temp_dir.join("keystore");
    std::fs::create_dir_all(&keystore_path)?;

    // Create client
    let mut rng = rand::rng();
    let coin_seed: [u64; 4] = rng.random();
    let rng_coin = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

    let keystore =
        FilesystemKeyStore::new(keystore_path.clone()).expect("Failed to create keystore");

    let mut client: Client<FilesystemKeyStore> = ClientBuilder::new()
        .rpc(Arc::new(GrpcClient::new(endpoint, 30_000)))
        .rng(Box::new(rng_coin))
        .sqlite_store(store_path)
        .filesystem_keystore(keystore_path.to_str().expect("keystore path should be valid UTF-8"))?
        .in_debug_mode(DebugMode::Disabled)
        .tx_discard_delta(None)
        .build()
        .await?;

    // Initial sync
    println!("Connecting to node at {endpoint}...");
    client.sync_state().await?;
    let chain_height = client.get_sync_height().await?;
    println!("Connected successfully. Chain height: {chain_height}");

    let account_id = if needs_expansion {
        // Two-phase deployment for large accounts

        // Phase 1: Create account with empty storage maps
        let t = Instant::now();
        let (account, secret_key, deferred_entries) =
            with_spinner("Creating account with empty storage maps", || async {
                create_minimal_account(&account_config)
            })
            .await?;
        println!("  Done in {:.2?}", t.elapsed());

        let account_id = account.id();

        // Add key and account to client
        keystore.add_key(&secret_key)?;
        client.add_account(&account, false).await?;

        // Deploy the minimal account
        let t = Instant::now();
        let tx_size = with_spinner("Deploying minimal account to network", || async {
            let tx_request = TransactionRequestBuilder::new().build()?;
            let tx_result = client.execute_transaction(account_id, tx_request).await?;
            let proven_tx = client.prove_transaction(&tx_result).await?;
            let tx_size = proven_tx.to_bytes().len();
            let submission_height = client.submit_proven_transaction(proven_tx, &tx_result).await?;
            client.apply_transaction(&tx_result, submission_height).await?;
            Ok::<_, anyhow::Error>(tx_size)
        })
        .await?;
        println!("  Done in {:.2?} (tx size: {})", t.elapsed(), format_size(tx_size));

        println!();
        println!("Account ID: {account_id}");
        println!("Seed: 0x{}", hex::encode(account_config.seed));
        println!();
        let t = Instant::now();
        println!("Waiting for chain block height to advance...");
        for _ in 0..4 {
            wait_for_block_advancement(&mut client).await?;
        }
        println!("  Done in {:.2?}", t.elapsed());

        // Phase 2: Expand storage maps through batched transactions
        println!();
        let t = Instant::now();
        println!(
            "Expanding storage maps ({total_entries} entries in batches of {ENTRIES_PER_EXPANSION_TX})..."
        );
        expand_storage_maps(&mut client, account_id, deferred_entries, maps).await?;
        println!("  Done in {:.2?}", t.elapsed());

        // Wait for the node to fully index all storage changes
        let t = Instant::now();
        println!("Waiting for storage indexing to complete...");
        for _ in 0..5 {
            wait_for_block_advancement(&mut client).await?;
        }
        println!("  Done in {:.2?}", t.elapsed());

        account_id
    } else {
        // Single-transaction deployment for small accounts
        use crate::generators::create_large_account;

        let t = Instant::now();
        let (account, secret_key) =
            with_spinner("Creating account", || async { create_large_account(&account_config) })
                .await?;
        println!("  Done in {:.2?}", t.elapsed());

        let account_id = account.id();

        // Add key and account to client
        keystore.add_key(&secret_key)?;
        client.add_account(&account, false).await?;

        // Deploy the account by submitting an empty transaction
        let t = Instant::now();
        let tx_size = with_spinner("Deploying account to network", || async {
            let tx_request = TransactionRequestBuilder::new().build()?;
            let tx_result = client.execute_transaction(account_id, tx_request).await?;
            let proven_tx = client.prove_transaction(&tx_result).await?;
            let tx_size = proven_tx.to_bytes().len();
            let submission_height = client.submit_proven_transaction(proven_tx, &tx_result).await?;
            client.apply_transaction(&tx_result, submission_height).await?;
            Ok::<_, anyhow::Error>(tx_size)
        })
        .await?;
        println!("  Done in {:.2?} (tx size: {})", t.elapsed(), format_size(tx_size));

        account_id
    };

    let seed = account_config.seed;

    println!();
    println!("Total deploy time: {:.2?}", total_t.elapsed());

    // Cleanup temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok((account_id, seed))
}
