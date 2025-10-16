use std::sync::Arc;

use anyhow::{Context, Result};
use miden_client::Felt;
use miden_client::account::{AccountIdAddress, AccountStorageMode, Address, AddressInterface};
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::{NoteType, create_p2id_note};
use miden_client::note_transport::NOTE_TRANSPORT_DEFAULT_ENDPOINT;
use miden_client::note_transport::grpc::GrpcNoteTransportClient;
use miden_client::store::NoteFilter;
use miden_client::testing::common::{TestClient, TestClientKeyStore, insert_new_wallet};

use crate::tests::config::ClientConfig;

pub async fn test_note_transport_flow(client_config: ClientConfig) -> Result<()> {
    // Build sender client with transport
    let (sender_builder, sender_keystore) = builder_with_transport(client_config.clone())
        .await
        .context("failed to get sender builder")?;
    let sender = sender_builder.build().await.context("failed to build sender client")?;
    // Build recipient client with transport
    let (recipient_builder, recipient_keystore) = builder_with_transport(client_config)
        .await
        .context("failed to get recipient builder")?;
    let recipient = recipient_builder.build().await.context("failed to build recipient client")?;

    run_flow(sender, &sender_keystore, recipient, &recipient_keystore).await
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

async fn run_flow(
    mut sender: TestClient,
    sender_keystore: &TestClientKeyStore,
    mut recipient: TestClient,
    recipient_keystore: &TestClientKeyStore,
) -> Result<()> {
    // Create accounts
    let (sender_account, _sk1) =
        insert_new_wallet(&mut sender, AccountStorageMode::Private, sender_keystore)
            .await
            .context("failed to insert sender wallet")?;
    let (recipient_account, _sk2) =
        insert_new_wallet(&mut recipient, AccountStorageMode::Private, recipient_keystore)
            .await
            .context("failed to insert recipient wallet")?;

    // Build recipient address
    let recipient_address = Address::AccountId(AccountIdAddress::new(
        recipient_account.id(),
        AddressInterface::BasicWallet,
    ));

    // Ensure recipient has no input notes
    recipient.sync_state().await.context("recipient initial sync")?;
    let notes = recipient.get_input_notes(NoteFilter::All).await?;
    anyhow::ensure!(notes.is_empty(), "recipient should start with 0 input notes");

    // Create a private P2ID note addressed to recipient
    let note = create_p2id_note(
        sender_account.id(),
        recipient_account.id(),
        vec![],
        NoteType::Private,
        Felt::default(),
        sender.rng(),
    )?;

    // Send over transport
    sender
        .send_private_note(note, &recipient_address)
        .await
        .context("send_private_note failed")?;

    // Recipient fetches via sync (includes transport fetch)
    recipient.sync_state().await?;
    let notes = recipient.get_input_notes(NoteFilter::All).await?;
    anyhow::ensure!(notes.len() == 1, "recipient should have exactly 1 input note after fetch");

    // Re-sync to verify cursor prevents duplicates
    recipient.sync_state().await?;
    let notes = recipient.get_input_notes(NoteFilter::All).await?;
    anyhow::ensure!(
        notes.len() == 1,
        "recipient should still have exactly 1 input note after re-sync"
    );

    Ok(())
}
