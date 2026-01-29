use std::sync::Arc;

use miden_client::account::AccountId;
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::rpc::{Endpoint, GrpcClient};
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::{Client, DebugMode, Felt};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use rand::Rng;

use crate::AccountSize;
use crate::generators::{LargeAccountConfig, create_large_account};
use crate::spinner::with_spinner;

/// Creates and deploys a public wallet with configurable storage to the network.
/// Returns the account ID of the deployed account.
pub async fn deploy_account(endpoint: &Endpoint, size: AccountSize) -> anyhow::Result<AccountId> {
    // Use a random seed to create unique accounts each time
    let mut rng = rand::rng();
    let mut random_seed = [0u8; 32];
    rng.fill(&mut random_seed);

    let mut account_config = LargeAccountConfig::from_size(size);
    account_config.seed = random_seed;

    let total_entries = account_config.num_map_slots * account_config.num_storage_map_entries;

    // Warn about memory requirements for large accounts
    if total_entries > 500 {
        eprintln!(
            "Warning: Deploying accounts with {total_entries} storage entries requires significant memory."
        );
        eprintln!(
            "         Transaction proving may fail or be killed by the OS on machines with <64GB RAM."
        );
        eprintln!("         Consider using --size small or --size medium for deployment.");
        eprintln!();
    }

    println!("Network: {endpoint}");
    println!("Account size: {size:?}");
    println!("Storage entries: {total_entries}");
    println!();

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
        .filesystem_keystore(keystore_path.to_str().expect("keystore path should be valid UTF-8"))
        .in_debug_mode(DebugMode::Disabled)
        .tx_graceful_blocks(None)
        .build()
        .await?;

    // Initial sync
    println!("Connecting to node at {endpoint}...");
    client.sync_state().await?;
    let chain_height = client.get_sync_height().await?;
    println!("Connected successfully. Chain height: {chain_height}");

    // Create the account with large storage
    let (account, secret_key) =
        with_spinner("Creating account", || async { create_large_account(&account_config) })
            .await?;

    let account_id = account.id();

    // Add key and account to client
    keystore.add_key(&secret_key)?;
    client.add_account(&account, false).await?;

    // Deploy the account by submitting an empty transaction
    // This increments the nonce from 0 to 1, deploying it on-chain
    // Note: This can be memory-intensive for large accounts
    with_spinner(
        "Deploying account to network (this may take a while for large accounts)",
        || async {
            let tx_request = TransactionRequestBuilder::new().build()?;
            client.submit_new_transaction(account_id, tx_request).await?;
            Ok::<_, anyhow::Error>(())
        },
    )
    .await?;

    println!();
    println!("Deployed public wallet with {total_entries} storage entries.");

    // Cleanup temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(account_id)
}
