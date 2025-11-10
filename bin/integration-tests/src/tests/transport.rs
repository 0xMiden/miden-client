use std::sync::Arc;

use anyhow::{Context, Result};
use miden_client::account::AccountStorageMode;
use miden_client::address::{Address, AddressInterface, RoutingParameters};
use miden_client::asset::FungibleAsset;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::NoteType;
use miden_client::note_transport::NOTE_TRANSPORT_DEFAULT_ENDPOINT;
use miden_client::note_transport::grpc::GrpcNoteTransportClient;
use miden_client::store::NoteFilter;
use miden_client::testing::common::{
    TestClient, TestClientKeyStore, execute_tx_and_sync, insert_new_fungible_faucet,
    insert_new_wallet, wait_for_node,
};
use miden_client::transaction::TransactionRequestBuilder;

use crate::tests::config::ClientConfig;

pub async fn test_note_transport_flow(client_config: ClientConfig) -> Result<()> {
    // Skip unless explicitly opted in via environment variable
    if std::env::var("TEST_WITH_NOTE_TRANSPORT").unwrap_or_default() != "1" {
        eprintln!("Skipping note transport test (set TEST_WITH_NOTE_TRANSPORT=1 to enable)");
        return Ok(());
    }

    // Create distinct configs so each client gets its own temp store/keystore
    let (rpc_endpoint, rpc_timeout, ..) = client_config.as_parts();
    let sender_config = ClientConfig::new(rpc_endpoint.clone(), rpc_timeout);
    let recipient_config = ClientConfig::new(rpc_endpoint, rpc_timeout);

    // Build sender client with transport
    let (sender_builder, sender_keystore) = builder_with_transport(sender_config)
        .await
        .context("failed to get sender builder")?;
    let sender = sender_builder.build().await.context("failed to build sender client")?;
    // Build recipient client with transport
    let (recipient_builder, recipient_keystore) = builder_with_transport(recipient_config)
        .await
        .context("failed to get recipient builder")?;
    let recipient = recipient_builder.build().await.context("failed to build recipient client")?;

    run_flow(sender, &sender_keystore, recipient, &recipient_keystore, true).await
}

/// Sender has transport; recipient does NOT. Recipient should not receive private notes.
pub async fn test_note_transport_sender_only(client_config: ClientConfig) -> Result<()> {
    // Skip unless explicitly opted in via environment variable
    if std::env::var("TEST_WITH_NOTE_TRANSPORT").unwrap_or_default() != "1" {
        eprintln!("Skipping note transport test (set TEST_WITH_NOTE_TRANSPORT=1 to enable)");
        return Ok(());
    }

    // Distinct configs for unique stores
    let (rpc_endpoint, rpc_timeout, ..) = client_config.as_parts();
    let sender_config = ClientConfig::new(rpc_endpoint.clone(), rpc_timeout);
    let recipient_config = ClientConfig::new(rpc_endpoint, rpc_timeout);

    // Sender with transport
    let (sender_builder, sender_keystore) = builder_with_transport(sender_config)
        .await
        .context("failed to get sender builder")?;
    let sender = sender_builder.build().await.context("failed to build sender client")?;

    // Recipient WITHOUT transport
    let (recipient_builder, recipient_keystore) = builder_without_transport(recipient_config)
        .await
        .context("failed to get recipient builder without transport")?;
    let recipient = recipient_builder.build().await.context("failed to build recipient client")?;

    run_flow(sender, &sender_keystore, recipient, &recipient_keystore, false).await
}

async fn builder_with_transport(
    client_config: ClientConfig,
) -> Result<(
    miden_client::builder::ClientBuilder<FilesystemKeyStore<rand::rngs::StdRng>>,
    FilesystemKeyStore<rand::rngs::StdRng>,
)> {
    let (mut builder, keystore) = client_config
        .into_client_builder()
        .await
        .context("failed to get client builder")?;

    // Determine endpoint from env, fallback to default constant
    let endpoint = std::env::var("TEST_MIDEN_NOTE_TRANSPORT_ENDPOINT")
        .unwrap_or_else(|_| NOTE_TRANSPORT_DEFAULT_ENDPOINT.to_string());
    let timeout = std::env::var("TEST_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(1_000);

    let nt_client = Arc::new(
        GrpcNoteTransportClient::connect(endpoint, timeout)
            .await
            .context("failed connecting note transport client")?,
    );

    builder = builder.note_transport(nt_client);
    Ok((builder, keystore))
}

async fn builder_without_transport(
    client_config: ClientConfig,
) -> Result<(
    miden_client::builder::ClientBuilder<FilesystemKeyStore<rand::rngs::StdRng>>,
    FilesystemKeyStore<rand::rngs::StdRng>,
)> {
    let (builder, keystore) = client_config
        .into_client_builder()
        .await
        .context("failed to get client builder")?;
    Ok((builder, keystore))
}

async fn run_flow(
    mut sender: TestClient,
    sender_keystore: &TestClientKeyStore,
    mut recipient: TestClient,
    recipient_keystore: &TestClientKeyStore,
    recipient_should_receive: bool,
) -> Result<()> {
    // Ensure node is up
    wait_for_node(&mut sender).await;

    // Create accounts
    let (recipient_account, _sk2) =
        insert_new_wallet(&mut recipient, AccountStorageMode::Private, recipient_keystore)
            .await
            .context("failed to insert recipient wallet")?;

    // Create a faucet in sender
    let (faucet_account, _faucet_sk) =
        insert_new_fungible_faucet(&mut sender, AccountStorageMode::Private, sender_keystore)
            .await
            .context("failed to insert faucet in sender")?;

    // Build recipient address
    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet))
        .context("failed to build recipient address")?;

    // Ensure recipient has no input notes
    recipient.sync_state().await.context("recipient initial sync")?;
    let notes = recipient.get_input_notes(NoteFilter::All).await?;
    assert!(notes.is_empty(), "recipient should start with 0 input notes");

    // Build private mint tx from faucet to recipient; capture expected note
    let fungible_asset = FungibleAsset::new(faucet_account.id(), 10).context("asset")?;
    let tx_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(
            fungible_asset,
            recipient_account.id(),
            NoteType::Private,
            sender.rng(),
        )
        .context("build mint tx")?;
    let note = tx_request
        .expected_output_own_notes()
        .last()
        .cloned()
        .context("expected output note missing")?;

    // Execute mint and wait for commit
    execute_tx_and_sync(&mut sender, faucet_account.id(), tx_request)
        .await
        .context("mint tx failed")?;

    // Send over transport
    sender
        .send_private_note(note.clone(), &recipient_address)
        .await
        .context("send_private_note failed")?;

    // Recipient fetches via sync (includes transport fetch only if configured)
    recipient.sync_state().await?;
    let notes = recipient.get_input_notes(NoteFilter::All).await?;
    if recipient_should_receive {
        assert_eq!(notes.len(), 1, "recipient should have exactly 1 input note after fetch");
        assert_eq!(notes[0].id(), note.id(), "received note id should match minted note id");
    } else {
        assert!(notes.is_empty(), "recipient should have 0 input notes without transport");
    }

    // Re-sync to verify cursor dedup (or still nothing if no transport)
    recipient.sync_state().await?;
    let notes = recipient.get_input_notes(NoteFilter::All).await?;
    if recipient_should_receive {
        assert_eq!(
            notes.len(),
            1,
            "recipient should still have exactly 1 input note after re-sync"
        );
        assert_eq!(notes[0].id(), note.id(), "re-synced note id should match minted note id");
    } else {
        assert!(notes.is_empty(), "recipient should still have 0 input notes");
    }

    Ok(())
}
