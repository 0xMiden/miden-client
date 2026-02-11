#![allow(clippy::cast_possible_truncation, clippy::cast_lossless)]

use std::fmt::Write;
use std::sync::Arc;
use std::time::Instant;

use miden_client::account::{AccountId, StorageMap, StorageSlot, StorageSlotName};
use miden_client::assembly::CodeBuilder;
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::rpc::{Endpoint, GrpcClient};
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::{Client, DebugMode, Felt, Serializable};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_protocol::account::auth::AuthSecretKey;
use miden_protocol::account::{
    Account,
    AccountBuilder,
    AccountComponent,
    AccountStorageMode,
    AccountType,
};
use miden_standards::account::auth::AuthFalcon512Rpo;
use miden_standards::account::components::basic_wallet_library;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use crate::generators::{SlotDescriptor, generate_reader_component_code};
use crate::report::format_size;

/// Waits for the chain height to advance, ensuring transaction is in a block
pub(crate) async fn wait_for_block_advancement(
    client: &mut Client<FilesystemKeyStore>,
) -> anyhow::Result<()> {
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

/// Generates MASM code for an account component that can set items in multiple storage maps.
/// Creates a procedure `set_item_slot_N` for each slot that receives key/value from the stack.
pub(crate) fn generate_expansion_component_code(num_slots: usize) -> String {
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

/// Creates an account with empty storage maps, expansion procedures, and reader procedures.
fn create_account_with_empty_maps(
    num_maps: usize,
    seed: [u8; 32],
) -> anyhow::Result<(Account, AuthSecretKey)> {
    let sk = AuthSecretKey::new_falcon512_rpo_with_rng(&mut ChaCha20Rng::from_seed(seed));

    // Create empty storage map slots
    let storage_slots: Vec<StorageSlot> = (0..num_maps)
        .map(|i| {
            let slot_name = format!("miden::bench::map_slot_{i}");
            StorageSlot::with_map(
                StorageSlotName::new(slot_name.as_str()).expect("slot name should be valid"),
                StorageMap::new(),
            )
        })
        .collect();

    // Expansion component: provides set_item_slot_N procedures (needed for expand command)
    let expansion_code = generate_expansion_component_code(num_maps);
    let expansion_component_code = CodeBuilder::default()
        .compile_component_code("miden::bench::storage_expander", &expansion_code)
        .map_err(|e| anyhow::anyhow!("Failed to compile expansion component: {e}"))?;
    let expansion_component = AccountComponent::new(expansion_component_code, storage_slots)
        .map_err(|e| anyhow::anyhow!("Failed to create expansion component: {e}"))?
        .with_supports_all_types();

    // Reader component: provides get_map_item_slot_N procedures (needed for transaction benchmarks)
    let descriptors: Vec<SlotDescriptor> = (0..num_maps)
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

    let account = AccountBuilder::new(seed)
        .with_auth_component(AuthFalcon512Rpo::new(sk.public_key().to_commitment()))
        .account_type(AccountType::RegularAccountUpdatableCode)
        .with_component(wallet_component)
        .with_component(expansion_component)
        .with_component(reader_component)
        .storage_mode(AccountStorageMode::Public)
        .build()?;

    Ok((account, sk))
}

/// Creates and deploys a public wallet with empty storage maps to the network.
/// Returns the account ID and the seed used for key generation (needed for signing transactions).
///
/// The deployed account includes expansion and reader procedures so that storage can be
/// filled later via the `expand` command and read during transaction benchmarks.
pub async fn deploy_account(
    endpoint: &Endpoint,
    maps: usize,
    store_path: Option<&str>,
) -> anyhow::Result<(AccountId, [u8; 32])> {
    println!("Network: {endpoint}");
    println!("Storage maps: {maps} (empty)");
    println!();

    let total_t = Instant::now();

    // Create directory for client data (persistent or temporary)
    let persistent = store_path.is_some();
    let data_dir = if let Some(path) = store_path {
        let p = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&p)?;
        p
    } else {
        let p = std::env::temp_dir().join(format!("miden-bench-deploy-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&p)?;
        p
    };
    let sqlite_path = data_dir.join("store.sqlite3");
    let keystore_path = data_dir.join("keystore");
    std::fs::create_dir_all(&keystore_path)?;

    if persistent {
        println!("Store directory: {}", data_dir.display());
    }

    // Generate a random seed for the account
    let mut rng = rand::rng();
    let mut account_seed = [0u8; 32];
    rng.fill(&mut account_seed);

    // Create client
    let coin_seed: [u64; 4] = rng.random();
    let rng_coin = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

    let keystore =
        FilesystemKeyStore::new(keystore_path.clone()).expect("Failed to create keystore");

    let mut client: Client<FilesystemKeyStore> = ClientBuilder::new()
        .rpc(Arc::new(GrpcClient::new(endpoint, 30_000)))
        .rng(Box::new(rng_coin))
        .sqlite_store(sqlite_path)
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

    // Create account with empty maps
    let t = Instant::now();
    println!("Creating account with {maps} empty storage maps...");
    let (account, secret_key) = create_account_with_empty_maps(maps, account_seed)?;
    println!("  Done in {:.2?}", t.elapsed());

    let account_id = account.id();

    // Add key and account to client
    keystore.add_key(&secret_key)?;
    client.add_account(&account, false).await?;

    // Deploy the account by submitting an empty transaction
    let t = Instant::now();
    println!("Deploying account to network...");
    let tx_request = TransactionRequestBuilder::new().build()?;
    let tx_result = client.execute_transaction(account_id, tx_request).await?;
    let proven_tx = client.prove_transaction(&tx_result).await?;
    let tx_size = proven_tx.to_bytes().len();
    let submission_height = client.submit_proven_transaction(proven_tx, &tx_result).await?;
    client.apply_transaction(&tx_result, submission_height).await?;
    println!("  Done in {:.2?} (tx size: {})", t.elapsed(), format_size(tx_size));

    // Wait for blocks to ensure deployment is finalized
    let t = Instant::now();
    println!("Waiting for chain block height to advance...");
    for _ in 0..4 {
        wait_for_block_advancement(&mut client).await?;
    }
    println!("  Done in {:.2?}", t.elapsed());

    let seed_hex = hex::encode(account_seed);
    println!();
    println!("Total deploy time: {:.2?}", total_t.elapsed());
    println!();
    println!("Account ID: {account_id}");
    println!("Seed: {seed_hex}");

    // Only cleanup when using a temporary directory
    if !persistent {
        let _ = std::fs::remove_dir_all(&data_dir);
    }

    Ok((account_id, account_seed))
}
