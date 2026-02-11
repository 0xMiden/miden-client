//! Integration tests for miden-client-service.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use miden_client::account::AccountStorageMode;
use miden_client::asset::FungibleAsset;
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::note::NoteType;
use miden_client::testing::common::*;
use miden_client::transaction::TransactionRequestBuilder;
use miden_client_service::{ClientService, ServiceConfig};

use crate::tests::config::ClientConfig;

/// Comprehensive test that verifies ClientService sync and transaction submission.
/// Tests:
/// - Creating a ClientService
/// - Performing sync operations
/// - Submitting a transaction through the coordinated service
/// - Verifying the service continues to work after transaction
pub async fn test_client_service_sync_and_transaction(client_config: ClientConfig) -> Result<()> {
    let (builder, keystore) = client_config.into_client_builder().await?;
    let mut client = builder.build().await.context("failed to build client")?;

    // Wait for node and initial sync
    wait_for_node(&mut client).await;

    // Set up a faucet account using the raw client (before wrapping in service)
    let (faucet, _faucet_key) = insert_new_fungible_faucet(
        &mut client,
        AccountStorageMode::Private,
        &keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("failed to create faucet")?;

    // Now wrap the client in ClientService
    let config = ServiceConfig::new().without_background_sync();
    let service = Arc::new(ClientService::new(client, config));

    // Perform initial sync through the service
    let summary = service.sync_state().await.context("failed to sync")?;
    assert!(summary.block_num.as_u32() > 0, "Should sync to a non-zero block");

    // Build and submit a mint transaction
    let target_account_id = miden_client::account::AccountId::try_from(ACCOUNT_ID_REGULAR)
        .context("failed to create target account id")?;
    let asset =
        FungibleAsset::new(faucet.id(), MINT_AMOUNT).context("failed to create fungible asset")?;

    let tx_request = {
        let mut client = service.client().await;
        TransactionRequestBuilder::new()
            .build_mint_fungible_asset(asset, target_account_id, NoteType::Public, client.rng())
            .context("failed to build mint transaction request")?
    };

    let tx_id = service
        .submit_transaction(faucet.id(), tx_request)
        .await
        .context("failed to submit transaction")?;

    println!("Transaction submitted: {tx_id}");

    // Sync again to verify service still works after transaction
    let summary2 = service.sync_state().await.context("failed to sync after transaction")?;
    assert!(summary2.block_num >= summary.block_num, "Should sync to same or later block");

    println!("Final sync to block {}", summary2.block_num);

    Ok(())
}

/// Tests that background sync works correctly with periodic automatic syncing.
pub async fn test_client_service_background_sync(client_config: ClientConfig) -> Result<()> {
    let (builder, _keystore) = client_config.into_client_builder().await?;
    let mut client = builder.build().await.context("failed to build client")?;

    // Wait for node to be ready
    wait_for_node(&mut client).await;
    println!("Node is ready, creating service...");

    // Create service with fast background sync for testing
    let config = ServiceConfig::new().with_sync_interval(Some(Duration::from_millis(500)));
    let service = Arc::new(ClientService::new(client, config));

    // Do one manual sync to verify the service works
    let initial_sync = service.sync_state().await.context("initial sync failed")?;
    println!("Initial manual sync succeeded, block_num: {}", initial_sync.block_num);

    // Start background sync
    let mut sync_handle = service.start_background_sync();
    assert!(sync_handle.is_active(), "Background sync handle should be active");

    // Wait for a few sync cycles (500ms interval, wait 2s = ~3-4 cycles)
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Stop background sync
    sync_handle.stop();
    assert!(!sync_handle.is_active(), "Background sync handle should be inactive after stop");

    // Verify service still works after stopping background sync
    let final_sync = service.sync_state().await.context("final sync failed")?;
    assert!(
        final_sync.block_num >= initial_sync.block_num,
        "Should sync to same or later block"
    );

    println!("Background sync test completed, final block: {}", final_sync.block_num);

    Ok(())
}
