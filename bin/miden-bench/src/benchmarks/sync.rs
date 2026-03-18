use std::path::Path;
use std::time::Instant;

use miden_client::account::AccountId;
use miden_client::rpc::Endpoint;

use crate::config;

/// Runs the sync benchmark: creates a fresh client, imports an account, and syncs from genesis.
pub async fn run_sync_benchmark(
    endpoint: &Endpoint,
    base_store_path: &Path,
    account_id_hex: &str,
    iterations: usize,
) -> anyhow::Result<()> {
    let account_id = AccountId::from_hex(account_id_hex)?;

    for i in 1..=iterations {
        println!("\n--- Iteration {i}/{iterations} ---");

        // Create a fresh store directory for each iteration so we sync from genesis
        let iter_store_path = base_store_path.join(format!("sync-bench-{i}"));
        std::fs::create_dir_all(&iter_store_path)?;

        let mut client = config::create_client(endpoint, &iter_store_path).await?;

        // Import the account so the sync tracks it
        println!("Importing account {account_id_hex}...");
        let import_start = Instant::now();
        client.import_account_by_id(account_id).await?;
        let import_duration = import_start.elapsed();
        println!("Account imported in {import_duration:.2?}");

        // Sync from genesis to chain tip
        println!("Syncing...");
        let sync_start = Instant::now();
        let summary = client.sync_state().await?;
        let sync_duration = sync_start.elapsed();

        println!(
            "Synced to block {} in {sync_duration:.2?} (committed: {}, consumed: {})",
            summary.block_num,
            summary.committed_notes.len(),
            summary.consumed_notes.len(),
        );

        // Clean up iteration store
        let _ = std::fs::remove_dir_all(&iter_store_path);
    }

    Ok(())
}
