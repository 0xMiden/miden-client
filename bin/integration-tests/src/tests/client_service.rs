//! Integration tests for miden-client-service.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use miden_client::account::AccountStorageMode;
use miden_client::asset::FungibleAsset;
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::note::NoteType;
use miden_client::testing::common::*;
use miden_client::transaction::TransactionRequestBuilder;
use miden_client_service::{AsyncEventHandler, ClientService, ServiceConfig, ServiceEvent};

use crate::tests::config::ClientConfig;

/// Test event handler that counts events.
struct CountingEventHandler {
    sync_completed_count: AtomicUsize,
}

impl CountingEventHandler {
    fn new() -> Self {
        Self {
            sync_completed_count: AtomicUsize::new(0),
        }
    }

    fn sync_completed_count(&self) -> usize {
        self.sync_completed_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl AsyncEventHandler for CountingEventHandler {
    async fn handle(&self, event: ServiceEvent) {
        if let ServiceEvent::SyncCompleted { .. } = event {
            self.sync_completed_count.fetch_add(1, Ordering::SeqCst);
        }
    }
}

/// Comprehensive test that verifies ClientService sync, events, and transaction submission.
/// Tests:
/// - Creating a ClientService and registering event handlers
/// - Performing sync operations with event verification
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
    let config = ServiceConfig::new().without_background_sync().with_sync_events(true);
    let service = Arc::new(ClientService::new(client, config));

    // Register event handler
    let handler = Arc::new(CountingEventHandler::new());
    service.register_async_handler(handler.clone()).await;

    // Perform initial sync through the service
    let summary = service.sync_state().await.context("failed to sync")?;
    assert!(summary.block_num.as_u32() > 0, "Should sync to a non-zero block");

    // Verify event was emitted
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(handler.sync_completed_count(), 1, "Should have received SyncCompleted event");

    // Build and submit a mint transaction
    let target_account_id = miden_client::account::AccountId::try_from(ACCOUNT_ID_REGULAR)
        .context("failed to create target account id")?;
    let asset =
        FungibleAsset::new(faucet.id(), MINT_AMOUNT).context("failed to create fungible asset")?;

    let tx_request = service
        .with_client_mut(|client| {
            TransactionRequestBuilder::new().build_mint_fungible_asset(
                asset,
                target_account_id,
                NoteType::Public,
                client.rng(),
            )
        })
        .await
        .context("failed to build mint transaction request")?;

    let tx_id = service
        .submit_transaction(faucet.id(), tx_request)
        .await
        .context("failed to submit transaction")?;

    println!("Transaction submitted: {tx_id}");

    // Sync again to verify service still works after transaction
    let summary2 = service.sync_state().await.context("failed to sync after transaction")?;
    assert!(summary2.block_num >= summary.block_num, "Should sync to same or later block");

    // Verify second event was emitted
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(
        handler.sync_completed_count(),
        2,
        "Should have received two SyncCompleted events"
    );

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
    let config = ServiceConfig::new()
        .with_sync_interval(Some(Duration::from_millis(500)))
        .with_sync_events(true);

    let service = Arc::new(ClientService::new(client, config));

    // Register event handler FIRST before any syncs
    let handler = Arc::new(CountingEventHandler::new());
    service.register_async_handler(handler.clone()).await;

    // Do one manual sync to verify the service works (now handler will see this event)
    let initial_sync = service.sync_state().await.context("initial sync failed")?;
    println!("Initial manual sync succeeded, block_num: {}", initial_sync.block_num);

    // Give time for the async event handler to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify we got the manual sync event
    let manual_sync_count = handler.sync_completed_count();
    println!("After manual sync, event count: {manual_sync_count}");
    assert_eq!(
        manual_sync_count, 1,
        "Should have received 1 SyncCompleted event from manual sync"
    );

    // Start background sync
    let mut sync_handle = service.start_background_sync();

    // Wait for a few sync cycles (500ms interval, wait 3s = ~5-6 cycles)
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Stop background sync
    sync_handle.stop();

    // Give time for any in-progress sync and spawned event handlers to complete
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Should have received at least 2 total events (1 manual + at least 1 from background)
    let sync_count = handler.sync_completed_count();
    println!("Total sync completed count: {sync_count}");
    assert!(
        sync_count >= 2,
        "Should have received at least 2 SyncCompleted events (1 manual + 1+ background), got {sync_count}"
    );

    println!("Background sync completed {} background sync cycles", sync_count - 1);

    Ok(())
}
