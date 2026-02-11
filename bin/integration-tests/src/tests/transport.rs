use std::sync::Arc;

use anyhow::{Context, Result};
use miden_client::account::AccountStorageMode;
use miden_client::address::{Address, AddressInterface, RoutingParameters};
use miden_client::asset::FungibleAsset;
use miden_client::auth::{AuthSchemeId, RPO_FALCON_SCHEME_ID};
use miden_client::note::NoteType;
use miden_client::note_transport::NOTE_TRANSPORT_DEFAULT_ENDPOINT;
use miden_client::note_transport::grpc::GrpcNoteTransportClient;
use miden_client::store::{InputNoteState, NoteFilter};
use miden_client::testing::common::{
    FilesystemKeyStore,
    TestClient,
    assert_account_has_single_asset,
    consume_notes,
    execute_tx_and_sync,
    insert_new_fungible_faucet,
    insert_new_wallet,
    wait_for_blocks,
    wait_for_node,
    wait_for_tx,
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

    run_flow(
        sender,
        &sender_keystore,
        recipient,
        &recipient_keystore,
        true,
        RPO_FALCON_SCHEME_ID,
    )
    .await
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

    run_flow(
        sender,
        &sender_keystore,
        recipient,
        &recipient_keystore,
        false,
        RPO_FALCON_SCHEME_ID,
    )
    .await
}

async fn builder_with_transport(
    client_config: ClientConfig,
) -> Result<(miden_client::builder::ClientBuilder<FilesystemKeyStore>, FilesystemKeyStore)> {
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
) -> Result<(miden_client::builder::ClientBuilder<FilesystemKeyStore>, FilesystemKeyStore)> {
    let (builder, keystore) = client_config
        .into_client_builder()
        .await
        .context("failed to get client builder")?;
    Ok((builder, keystore))
}

async fn run_flow(
    mut sender: TestClient,
    sender_keystore: &FilesystemKeyStore,
    mut recipient: TestClient,
    recipient_keystore: &FilesystemKeyStore,
    recipient_should_receive: bool,
    auth_scheme: AuthSchemeId,
) -> Result<()> {
    // Ensure node is up
    wait_for_node(&mut sender).await;

    // Create accounts
    let (recipient_account, _sk2) = insert_new_wallet(
        &mut recipient,
        AccountStorageMode::Private,
        recipient_keystore,
        auth_scheme,
    )
    .await
    .context("failed to insert recipient wallet")?;

    // Create a faucet in sender
    let (faucet_account, _faucet_sk) = insert_new_fungible_faucet(
        &mut sender,
        AccountStorageMode::Private,
        sender_keystore,
        auth_scheme,
    )
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
    if recipient_should_receive {
        wait_for_blocks(&mut recipient, 1).await;
        let notes = recipient.get_input_notes(NoteFilter::All).await?;
        assert_eq!(
            notes[0].commitment().unwrap(), // we should have a commitment at this point
            note.commitment(),
            "re-synced note id should match minted note id"
        );

        let consumable_notes = recipient.get_consumable_notes(Some(recipient_account.id())).await?;
        assert_eq!(consumable_notes.len(), 1, "recipient should have 1 consumable note");
    } else {
        assert!(notes.is_empty(), "recipient should still have 0 input notes");
    }

    Ok(())
}

// TRANSPORT NOTE INCLUSION PROOF AND CONSUMPTION TESTS
// ================================================================================================

/// Full end-to-end test: transport fetch → inclusion proof verification → consumption.
pub async fn test_transport_note_inclusion_proof_and_consumption(
    client_config: ClientConfig,
) -> Result<()> {
    if std::env::var("TEST_WITH_NOTE_TRANSPORT").unwrap_or_default() != "1" {
        eprintln!("Skipping note transport test (set TEST_WITH_NOTE_TRANSPORT=1 to enable)");
        return Ok(());
    }

    let (rpc_endpoint, rpc_timeout, ..) = client_config.as_parts();
    let sender_config = ClientConfig::new(rpc_endpoint.clone(), rpc_timeout);
    let recipient_config = ClientConfig::new(rpc_endpoint, rpc_timeout);

    let (sender_builder, sender_keystore) = builder_with_transport(sender_config)
        .await
        .context("failed to get sender builder")?;
    let mut sender = sender_builder.build().await.context("failed to build sender")?;
    let (recipient_builder, recipient_keystore) = builder_with_transport(recipient_config)
        .await
        .context("failed to get recipient builder")?;
    let mut recipient = recipient_builder.build().await.context("failed to build recipient")?;

    wait_for_node(&mut sender).await;

    let (faucet_account, _) = insert_new_fungible_faucet(
        &mut sender,
        AccountStorageMode::Private,
        &sender_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("failed to insert faucet")?;

    let (recipient_account, _) = insert_new_wallet(
        &mut recipient,
        AccountStorageMode::Private,
        &recipient_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("failed to insert wallet")?;

    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet))
        .context("failed to build address")?;

    // Initial sync
    recipient.sync_state().await.context("recipient initial sync")?;

    // Mint private note
    let fungible_asset = FungibleAsset::new(faucet_account.id(), 100).context("asset")?;
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

    execute_tx_and_sync(&mut sender, faucet_account.id(), tx_request)
        .await
        .context("mint tx failed")?;

    // Send via transport
    sender
        .send_private_note(note.clone(), &recipient_address)
        .await
        .context("send_private_note failed")?;

    // Recipient syncs (transport fetch + state sync)
    recipient.sync_state().await.context("recipient sync")?;

    // Verify note state
    let notes = recipient.get_input_notes(NoteFilter::All).await?;
    assert_eq!(notes.len(), 1, "recipient should have 1 note");
    assert!(
        matches!(notes[0].state(), InputNoteState::Committed(..)),
        "note should be committed, got: {:?}",
        notes[0].state()
    );
    assert!(notes[0].inclusion_proof().is_some(), "note should have inclusion proof");

    // Verify consumability
    let consumable = recipient.get_consumable_notes(Some(recipient_account.id())).await?;
    assert_eq!(consumable.len(), 1, "1 consumable note expected");

    // Consume the note
    let tx_id = consume_notes(&mut recipient, recipient_account.id(), &[note]).await;
    wait_for_tx(&mut recipient, tx_id).await?;

    // Verify balance
    assert_account_has_single_asset(&recipient, recipient_account.id(), faucet_account.id(), 100)
        .await;

    Ok(())
}

/// Tests that a note committed before the recipient's sync height is still found and consumable.
/// This validates the fix where `after_block_num` is set to block 0 instead of `sync_height`.
pub async fn test_transport_note_from_untracked_block(client_config: ClientConfig) -> Result<()> {
    if std::env::var("TEST_WITH_NOTE_TRANSPORT").unwrap_or_default() != "1" {
        eprintln!("Skipping note transport test (set TEST_WITH_NOTE_TRANSPORT=1 to enable)");
        return Ok(());
    }

    let (rpc_endpoint, rpc_timeout, ..) = client_config.as_parts();
    let sender_config = ClientConfig::new(rpc_endpoint.clone(), rpc_timeout);
    let recipient_config = ClientConfig::new(rpc_endpoint, rpc_timeout);

    let (sender_builder, sender_keystore) = builder_with_transport(sender_config)
        .await
        .context("failed to get sender builder")?;
    let mut sender = sender_builder.build().await.context("failed to build sender")?;
    let (recipient_builder, recipient_keystore) = builder_with_transport(recipient_config)
        .await
        .context("failed to get recipient builder")?;
    let mut recipient = recipient_builder.build().await.context("failed to build recipient")?;

    wait_for_node(&mut sender).await;

    let (faucet_account, _) = insert_new_fungible_faucet(
        &mut sender,
        AccountStorageMode::Private,
        &sender_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("failed to insert faucet")?;

    let (recipient_account, _) = insert_new_wallet(
        &mut recipient,
        AccountStorageMode::Private,
        &recipient_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("failed to insert wallet")?;

    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet))
        .context("failed to build address")?;

    // Initial sync
    recipient.sync_state().await.context("recipient initial sync")?;

    // Mint private note (committed at block N)
    let fungible_asset = FungibleAsset::new(faucet_account.id(), 100).context("asset")?;
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

    execute_tx_and_sync(&mut sender, faucet_account.id(), tx_request)
        .await
        .context("mint tx failed")?;

    // Advance recipient past the note's commit block so sync_height > block N
    wait_for_blocks(&mut recipient, 3).await;

    // Now send via transport (note committed at a block the recipient already synced past)
    sender
        .send_private_note(note.clone(), &recipient_address)
        .await
        .context("send_private_note failed")?;

    // Recipient syncs — with the fix, check_expected_notes starts from block 0
    recipient.sync_state().await.context("recipient sync after transport")?;

    // Verify note was found and committed despite being in a past block
    let notes = recipient.get_input_notes(NoteFilter::All).await?;
    assert_eq!(notes.len(), 1, "recipient should have 1 note");
    assert!(
        matches!(notes[0].state(), InputNoteState::Committed(..)),
        "note should be committed (proves the fix works), got: {:?}",
        notes[0].state()
    );
    assert!(notes[0].inclusion_proof().is_some(), "note should have inclusion proof");

    // Consume and verify balance
    let tx_id = consume_notes(&mut recipient, recipient_account.id(), &[note]).await;
    wait_for_tx(&mut recipient, tx_id).await?;

    assert_account_has_single_asset(&recipient, recipient_account.id(), faucet_account.id(), 100)
        .await;

    Ok(())
}

/// Tests fetching and consuming multiple notes committed in different blocks.
pub async fn test_transport_multiple_notes_different_blocks(
    client_config: ClientConfig,
) -> Result<()> {
    if std::env::var("TEST_WITH_NOTE_TRANSPORT").unwrap_or_default() != "1" {
        eprintln!("Skipping note transport test (set TEST_WITH_NOTE_TRANSPORT=1 to enable)");
        return Ok(());
    }

    let (rpc_endpoint, rpc_timeout, ..) = client_config.as_parts();
    let sender_config = ClientConfig::new(rpc_endpoint.clone(), rpc_timeout);
    let recipient_config = ClientConfig::new(rpc_endpoint, rpc_timeout);

    let (sender_builder, sender_keystore) = builder_with_transport(sender_config)
        .await
        .context("failed to get sender builder")?;
    let mut sender = sender_builder.build().await.context("failed to build sender")?;
    let (recipient_builder, recipient_keystore) = builder_with_transport(recipient_config)
        .await
        .context("failed to get recipient builder")?;
    let mut recipient = recipient_builder.build().await.context("failed to build recipient")?;

    wait_for_node(&mut sender).await;

    let (faucet_account, _) = insert_new_fungible_faucet(
        &mut sender,
        AccountStorageMode::Private,
        &sender_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("failed to insert faucet")?;

    let (recipient_account, _) = insert_new_wallet(
        &mut recipient,
        AccountStorageMode::Private,
        &recipient_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("failed to insert wallet")?;

    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet))
        .context("failed to build address")?;

    // Initial sync
    recipient.sync_state().await.context("recipient initial sync")?;

    // Mint 3 notes with different amounts, each committed in a separate block via
    // execute_tx_and_sync (which waits for each tx to be committed before returning).
    let amounts = [10u64, 20, 30];
    let mut minted_notes = Vec::new();
    for amount in amounts {
        let fungible_asset =
            FungibleAsset::new(faucet_account.id(), amount).context("failed to create asset")?;
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
        execute_tx_and_sync(&mut sender, faucet_account.id(), tx_request)
            .await
            .context("mint tx failed")?;
        minted_notes.push(note);
    }

    // Send all 3 notes via transport
    for note in &minted_notes {
        sender
            .send_private_note(note.clone(), &recipient_address)
            .await
            .context("send_private_note failed")?;
    }

    // Recipient syncs
    recipient.sync_state().await.context("recipient sync")?;

    // Verify all notes received and committed
    let input_notes = recipient.get_input_notes(NoteFilter::All).await?;
    assert_eq!(input_notes.len(), 3, "recipient should have 3 notes");
    for input_note in &input_notes {
        assert!(
            matches!(input_note.state(), InputNoteState::Committed(..)),
            "note should be committed, got: {:?}",
            input_note.state()
        );
        assert!(input_note.inclusion_proof().is_some(), "note should have inclusion proof");
    }

    // Verify at least 2 different commit blocks
    let mut block_nums: Vec<_> = input_notes
        .iter()
        .filter_map(|n| n.inclusion_proof().map(|p| p.location().block_num()))
        .collect();
    block_nums.sort();
    block_nums.dedup();
    assert!(
        block_nums.len() >= 2,
        "expected at least 2 different commit blocks, got {}",
        block_nums.len()
    );

    // Verify all consumable
    let consumable = recipient.get_consumable_notes(Some(recipient_account.id())).await?;
    assert_eq!(consumable.len(), 3, "3 consumable notes expected");

    // Consume all notes
    let tx_id = consume_notes(&mut recipient, recipient_account.id(), &minted_notes).await;
    wait_for_tx(&mut recipient, tx_id).await?;

    // Verify total balance (10 + 20 + 30 = 60)
    assert_account_has_single_asset(&recipient, recipient_account.id(), faucet_account.id(), 60)
        .await;

    Ok(())
}

/// Tests that a note sent via transport before being committed on-chain starts as Expected,
/// then transitions to Committed once the mint tx is executed and synced.
pub async fn test_transport_note_not_yet_committed(client_config: ClientConfig) -> Result<()> {
    if std::env::var("TEST_WITH_NOTE_TRANSPORT").unwrap_or_default() != "1" {
        eprintln!("Skipping note transport test (set TEST_WITH_NOTE_TRANSPORT=1 to enable)");
        return Ok(());
    }

    let (rpc_endpoint, rpc_timeout, ..) = client_config.as_parts();
    let sender_config = ClientConfig::new(rpc_endpoint.clone(), rpc_timeout);
    let recipient_config = ClientConfig::new(rpc_endpoint, rpc_timeout);

    let (sender_builder, sender_keystore) = builder_with_transport(sender_config)
        .await
        .context("failed to get sender builder")?;
    let mut sender = sender_builder.build().await.context("failed to build sender")?;
    let (recipient_builder, recipient_keystore) = builder_with_transport(recipient_config)
        .await
        .context("failed to get recipient builder")?;
    let mut recipient = recipient_builder.build().await.context("failed to build recipient")?;

    wait_for_node(&mut sender).await;

    let (faucet_account, _) = insert_new_fungible_faucet(
        &mut sender,
        AccountStorageMode::Private,
        &sender_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("failed to insert faucet")?;

    let (recipient_account, _) = insert_new_wallet(
        &mut recipient,
        AccountStorageMode::Private,
        &recipient_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .context("failed to insert wallet")?;

    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet))
        .context("failed to build address")?;

    // Initial sync
    recipient.sync_state().await.context("recipient initial sync")?;

    // Build mint tx and extract the note BEFORE executing
    let fungible_asset = FungibleAsset::new(faucet_account.id(), 100).context("asset")?;
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

    // Send via transport BEFORE the note is committed on-chain
    sender
        .send_private_note(note.clone(), &recipient_address)
        .await
        .context("send_private_note failed")?;

    // Recipient syncs — transport fetch finds the note, but it's not on chain yet
    recipient.sync_state().await.context("recipient sync (pre-commit)")?;

    let notes = recipient.get_input_notes(NoteFilter::All).await?;
    assert_eq!(notes.len(), 1, "recipient should have 1 note after transport fetch");
    assert!(
        matches!(notes[0].state(), InputNoteState::Expected(..)),
        "note should be Expected (not yet on chain), got: {:?}",
        notes[0].state()
    );
    assert!(notes[0].inclusion_proof().is_none(), "no inclusion proof before commit");

    let consumable = recipient.get_consumable_notes(Some(recipient_account.id())).await?;
    assert_eq!(consumable.len(), 0, "note should not be consumable yet");

    // Now execute the mint tx — note commits on chain
    execute_tx_and_sync(&mut sender, faucet_account.id(), tx_request)
        .await
        .context("mint tx failed")?;

    // Recipient syncs again — note tag tracking finds it on chain
    recipient.sync_state().await.context("recipient sync (post-commit)")?;

    let notes = recipient.get_input_notes(NoteFilter::All).await?;
    assert_eq!(notes.len(), 1);
    assert!(
        matches!(notes[0].state(), InputNoteState::Committed(..)),
        "note should now be Committed, got: {:?}",
        notes[0].state()
    );
    assert!(notes[0].inclusion_proof().is_some(), "should have inclusion proof after commit");

    // Consume the note
    let tx_id = consume_notes(&mut recipient, recipient_account.id(), &[note]).await;
    wait_for_tx(&mut recipient, tx_id).await?;

    assert_account_has_single_asset(&recipient, recipient_account.id(), faucet_account.id(), 100)
        .await;

    Ok(())
}
