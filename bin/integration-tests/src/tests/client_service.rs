//! Integration tests for miden-client-service.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use miden_client::account::AccountStorageMode;
use miden_client::asset::{Asset, FungibleAsset};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::note::NoteType;
use miden_client::testing::common::*;
use miden_client::transaction::{PaymentNoteDescription, TransactionRequestBuilder};
use miden_client_service::{ClientEvent, ClientService, EventFilter, ServiceConfig};

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

/// End-to-end reactive flow across two [`ClientService`]s:
///   - A sends an asset to B via `submit_transaction`.
///   - B *reacts* to the `NoteReceived` event with a registered handler that builds a consume tx
///     and pushes it onto a [`TransactionQueueHandle`]. B never calls `submit_transaction` directly
///     — the handler + queue do it.
///   - Test observes the outcome purely through events (`NoteConsumed` on B, balance updated).
///
/// Exercises: background sync on both sides, `on()` event handlers, `expect()` lossless
/// awaiters, the transaction queue (event → enqueue → submit), and the `persistence-mirror`
/// invariant (events fire only after the store commits).
pub async fn test_client_service_reactive_transfer(client_config: ClientConfig) -> Result<()> {
    const TRANSFER: u64 = MINT_AMOUNT / 2;

    // --- Set up client A with faucet + wallet_a and fund wallet_a. ---
    let (mut client_a, keystore_a) = client_config.clone().into_client().await?;
    let (faucet, _faucet_key) = insert_new_fungible_faucet(
        &mut client_a,
        AccountStorageMode::Public,
        &keystore_a,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("create faucet on A")?;
    let (wallet_a, _key_a) = insert_new_wallet(
        &mut client_a,
        AccountStorageMode::Public,
        &keystore_a,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("create wallet on A")?;

    let mint_tx =
        mint_and_consume(&mut client_a, wallet_a.id(), faucet.id(), NoteType::Public).await;
    wait_for_tx(&mut client_a, mint_tx).await?;

    // --- Set up client B with wallet_b. ---
    let (mut client_b, keystore_b) = client_config.into_client().await?;
    let (wallet_b, _key_b) = insert_new_wallet(
        &mut client_b,
        AccountStorageMode::Public,
        &keystore_b,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("create wallet on B")?;
    let wallet_b_id = wallet_b.id();

    // --- Wrap both in services with fast background sync. ---
    let svc_config = ServiceConfig::new().with_sync_interval(Some(Duration::from_millis(500)));
    let svc_a = Arc::new(ClientService::new(client_a, svc_config.clone()));
    let svc_b = Arc::new(ClientService::new(client_b, svc_config));
    let _sync_a = svc_a.start_background_sync();
    let _sync_b = svc_b.start_background_sync();

    // --- B starts a transaction queue; an event handler pushes consume-requests onto it. ---
    //
    // The NoteReceived event carries the full InputNoteRecord, so the handler doesn't need
    // to touch the store at all — just convert, build, enqueue. Fire-and-forget via the
    // transaction queue worker. Success surfaces via the NoteConsumed event awaited below.
    let tx_queue = svc_b.start_transaction_queue();
    let queue_for_handler = tx_queue.clone();

    svc_b.on(EventFilter::AnyNoteReceived, move |event, _svc| {
        let queue = queue_for_handler.clone();
        async move {
            let ClientEvent::NoteReceived { note } = event else {
                return;
            };
            let Ok(concrete_note) = (*note).clone().try_into() else {
                return;
            };
            let Ok(req) = TransactionRequestBuilder::new().build_consume_notes(vec![concrete_note])
            else {
                return;
            };
            drop(queue.enqueue(wallet_b_id, req));
        }
    });

    // --- Kick off the transfer on A. ---
    let asset = FungibleAsset::new(faucet.id(), TRANSFER).context("build asset")?;
    let (transfer_req, expected_note) = {
        let mut client = svc_a.client().await;
        let req = TransactionRequestBuilder::new()
            .build_pay_to_id(
                PaymentNoteDescription::new(
                    vec![Asset::Fungible(asset)],
                    wallet_a.id(),
                    wallet_b_id,
                ),
                NoteType::Public,
                client.rng(),
            )
            .context("build pay_to_id")?;
        let note = req
            .expected_output_own_notes()
            .into_iter()
            .next()
            .context("pay_to_id produces an own output note")?;
        (req, note)
    };

    // Pre-register the observable checkpoints we're going to assert against, BEFORE
    // submitting. These are lossless — events emitted during the upcoming syncs can't be
    // dropped by a slow broadcast subscriber.
    let received_awaiter = svc_b.expect(EventFilter::NoteReceived(expected_note.id()));
    let consumed_awaiter = svc_b.expect(EventFilter::AnyNoteConsumed);

    let transfer_tx_id = svc_a
        .submit_transaction(wallet_a.id(), transfer_req)
        .await
        .context("A submit transfer")?;
    println!("A submitted transfer tx: {transfer_tx_id}");

    // B must first see the note (event drives the handler, which enqueues).
    tokio::time::timeout(Duration::from_secs(60), received_awaiter)
        .await
        .context("timed out waiting for NoteReceived on B")?
        .context("NoteReceived awaiter cancelled")?;
    println!("B: NoteReceived event fired — handler should now be enqueueing consume");

    // Then, the queue-submitted consume tx must land on-chain and trigger NoteConsumed.
    tokio::time::timeout(Duration::from_secs(60), consumed_awaiter)
        .await
        .context("timed out waiting for NoteConsumed on B")?
        .context("NoteConsumed awaiter cancelled")?;
    println!("B: NoteConsumed event fired — consume tx committed");

    // --- Verify the balance actually arrived. ---
    let balance = {
        let client = svc_b.client().await;
        client
            .account_reader(wallet_b_id)
            .get_balance(faucet.id())
            .await
            .context("read wallet_b balance")?
    };
    if balance != TRANSFER {
        bail!("wallet_b balance {balance} != expected {TRANSFER}");
    }
    println!("wallet_b balance: {balance}");

    // Queue shutdown: dropping tx_queue closes the channel, the worker exits.
    drop(tx_queue);

    Ok(())
}
