use std::env::temp_dir;
use std::sync::Arc;

use miden_client::DebugMode;
use miden_client::account::{Account, AccountStorageMode};
use miden_client::address::{Address, AddressInterface, RoutingParameters};
use miden_client::builder::ClientBuilder;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::{NoteAttachment, NoteDetails, NoteTag, NoteType};
use miden_client::store::NoteFilter;
use miden_client::testing::common::create_test_store_path;
use miden_client::testing::mock::{MockClient, MockRpcApi};
use miden_client::testing::note_transport::{MockNoteTransportApi, MockNoteTransportNode};
use miden_client::utils::RwLock;
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_protocol::Felt;
use miden_protocol::crypto::rand::RandomCoin;
use miden_protocol::note::NoteType as ProtocolNoteType;
use miden_protocol::transaction::RawOutputNote;
use miden_protocol::utils::serde::Serializable;
use miden_standards::note::P2idNote;
use miden_standards::testing::note::NoteBuilder;
use miden_testing::{MockChainBuilder, TxContextInput};
use rand::Rng;

use crate::tests::{create_test_client_builder, insert_new_wallet};

#[tokio::test]
async fn transport_basic() {
    // Setup entities
    let mock_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));
    let (mut sender, sender_account) = create_test_user_transport(mock_node.clone()).await;
    let (mut recipient, recipient_account) = create_test_user_transport(mock_node.clone()).await;
    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet));
    let (mut observer, _observer_account) = create_test_user_transport(mock_node.clone()).await;

    // Create note
    let note = P2idNote::create(
        sender_account.id(),
        recipient_account.id(),
        vec![],
        NoteType::Private,
        NoteAttachment::default(),
        sender.rng(),
    )
    .unwrap();

    // Sync-state / fetch notes
    // No notes before sending
    recipient.sync_state().await.unwrap();
    let notes = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(notes.len(), 0);

    // Send note
    sender.send_private_note(note, &recipient_address).await.unwrap();

    // Sync-state / fetch notes
    // 1 note stored
    recipient.sync_state().await.unwrap();
    let notes = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(notes.len(), 1);

    // Sync again, should be only 1 note stored
    recipient.sync_state().await.unwrap();
    let notes = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(notes.len(), 1);

    // Third user shouldn't receive any note
    observer.sync_state().await.unwrap();
    let notes = observer.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(notes.len(), 0);
}

/// Verifies that cursor-based pagination works: a second sync only receives newly sent notes.
#[tokio::test]
async fn transport_cursor_pagination() {
    let mock_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));
    let (mut sender, sender_account) = create_test_user_transport(mock_node.clone()).await;
    let (mut recipient, recipient_account) = create_test_user_transport(mock_node.clone()).await;
    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet));

    let note_a = P2idNote::create(
        sender_account.id(),
        recipient_account.id(),
        vec![],
        NoteType::Private,
        NoteAttachment::default(),
        sender.rng(),
    )
    .unwrap();

    let note_b = P2idNote::create(
        sender_account.id(),
        recipient_account.id(),
        vec![],
        NoteType::Private,
        NoteAttachment::default(),
        sender.rng(),
    )
    .unwrap();

    // Send note A, sync → recipient receives 1 note
    sender.send_private_note(note_a.clone(), &recipient_address).await.unwrap();
    recipient.sync_state().await.unwrap();
    let notes = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(notes.len(), 1, "should have 1 note after first sync");
    assert_eq!(notes[0].id(), note_a.id());

    // Send note B, sync → recipient receives note B (cursor advanced past A)
    sender.send_private_note(note_b.clone(), &recipient_address).await.unwrap();
    recipient.sync_state().await.unwrap();
    let notes = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(notes.len(), 2, "should have 2 notes total after second sync");
}

/// Verifies that `fetch_all_private_notes` (cursor reset) does not duplicate notes in the store.
#[tokio::test]
async fn transport_duplicate_note_handling() {
    let mock_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));
    let (mut sender, sender_account) = create_test_user_transport(mock_node.clone()).await;
    let (mut recipient, recipient_account) = create_test_user_transport(mock_node.clone()).await;
    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet));

    let note = P2idNote::create(
        sender_account.id(),
        recipient_account.id(),
        vec![],
        NoteType::Private,
        NoteAttachment::default(),
        sender.rng(),
    )
    .unwrap();

    sender.send_private_note(note, &recipient_address).await.unwrap();

    // First fetch
    recipient.sync_state().await.unwrap();
    let notes = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(notes.len(), 1);

    // Reset cursor and re-fetch everything
    recipient.fetch_all_private_notes().await.unwrap();
    let notes = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(notes.len(), 1, "should still have 1 note, not duplicated");
}

/// Verifies that an observer whose tracked tags don't match the note's tag receives nothing.
#[tokio::test]
async fn transport_fetch_no_matching_tags() {
    let mock_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));
    let (mut sender, sender_account) = create_test_user_transport(mock_node.clone()).await;
    let (mut recipient, recipient_account) = create_test_user_transport(mock_node.clone()).await;
    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet));
    let (mut observer, _observer_account) = create_test_user_transport(mock_node.clone()).await;

    let note = P2idNote::create(
        sender_account.id(),
        recipient_account.id(),
        vec![],
        NoteType::Private,
        NoteAttachment::default(),
        sender.rng(),
    )
    .unwrap();

    sender.send_private_note(note, &recipient_address).await.unwrap();

    // Observer syncs — tags don't match, should get nothing
    observer.sync_state().await.unwrap();
    let notes = observer.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(notes.len(), 0, "observer with non-matching tags should receive 0 notes");

    // Recipient syncs — tags match, should get the note
    recipient.sync_state().await.unwrap();
    let notes = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(notes.len(), 1, "recipient with matching tags should receive 1 note");
}

/// Tests that a private note committed on-chain at the same block the client has synced to
/// is still found when imported via the NTL path. This reproduces the race condition where
/// fast sync (e.g. every 3s) causes `sync_height` to advance past the note's commitment
/// block before the NTL delivers the note details.
#[tokio::test]
async fn fetch_private_notes_finds_note_committed_at_sync_height() {
    // 1. Build a mock chain with a private note committed at block 1.
    let mut mock_chain_builder = MockChainBuilder::new();
    let mock_account = mock_chain_builder
        .add_existing_mock_account(miden_testing::Auth::IncrNonce)
        .unwrap();

    let private_note =
        NoteBuilder::new(mock_account.id(), RandomCoin::new([1, 2, 3, 4].map(Felt::new).into()))
            .note_type(ProtocolNoteType::Private)
            .tag(NoteTag::new(0).into())
            .build()
            .unwrap();

    let spawn_note =
        mock_chain_builder.add_spawn_note(std::slice::from_ref(&private_note)).unwrap();
    let mut mock_chain = mock_chain_builder.build().unwrap();

    // Block 1: commit the private note.
    let tx = Box::pin(
        mock_chain
            .build_tx_context(TxContextInput::AccountId(mock_account.id()), &[], &[spawn_note])
            .unwrap()
            .extend_expected_output_notes(vec![RawOutputNote::Full(private_note.clone())])
            .build()
            .unwrap()
            .execute(),
    )
    .await
    .unwrap();
    mock_chain.add_pending_executed_transaction(&tx).unwrap();
    mock_chain.prove_next_block().unwrap();

    // Advance the chain several blocks past the note's commitment block.
    for _ in 0..5 {
        mock_chain.prove_next_block().unwrap();
    }

    // 2. Create client with empty NTL (note not yet delivered).
    let mock_transport_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));

    let rpc_api = MockRpcApi::new(mock_chain);
    let arc_rpc_api = Arc::new(rpc_api);
    let transport_client = MockNoteTransportApi::new(mock_transport_node.clone());

    let mut rng = rand::rng();
    let coin_seed: [u64; 4] = rng.random();
    let rng = RandomCoin::new(coin_seed.map(Felt::new).into());

    let keystore_path = temp_dir();
    let keystore = FilesystemKeyStore::new(keystore_path.clone()).unwrap();

    let builder: ClientBuilder<FilesystemKeyStore> = ClientBuilder::new()
        .rpc(arc_rpc_api)
        .rng(Box::new(rng))
        .sqlite_store(create_test_store_path())
        .authenticator(Arc::new(keystore))
        .in_debug_mode(DebugMode::Enabled)
        .tx_discard_delta(None)
        .note_transport(Arc::new(transport_client));

    let mut client = builder.build().await.unwrap();
    client.ensure_genesis_in_place().await.unwrap();

    // 3. Register tag 0 so chain sync sees the note's block.
    client.add_note_tag(NoteTag::new(0)).await.unwrap();

    // 4. Sync to chain tip. The NTL is empty so no transport notes are imported.
    client.sync_state().await.unwrap();
    let sync_height = client.get_sync_height().await.unwrap();
    assert!(sync_height.as_u32() > 1, "client should have synced past block 1");

    // 5. Now the NTL delivers the note (simulates late delivery after the first sync).
    let details = NoteDetails::from(private_note.clone());
    let details_bytes = details.to_bytes();
    mock_transport_node
        .write()
        .add_note(private_note.header().clone(), details_bytes);

    // 6. Second sync_state: fetch_transport_notes imports the note, then chain sync runs.
    // Without the fix, after_block_num = sync_height, scan misses the note at block 1.
    // With the fix, lookback window catches it.
    client.sync_state().await.unwrap();

    // 7. The note should be Committed after the second sync.
    let committed_notes = client.get_input_notes(NoteFilter::Committed).await.unwrap();
    assert!(
        committed_notes.iter().any(|n| n.id() == private_note.id()),
        "note committed before sync_height should be found via lookback during NTL import"
    );
}

// HELPERS
// ================================================================================================

pub async fn create_test_client_transport(
    mock_node: Arc<RwLock<MockNoteTransportNode>>,
) -> (MockClient<FilesystemKeyStore>, FilesystemKeyStore) {
    let (builder, _, keystore) = create_test_client_builder().await;
    let transport_client = MockNoteTransportApi::new(mock_node);
    let builder_w_transport = builder.note_transport(Arc::new(transport_client));

    let mut client = builder_w_transport.build().await.unwrap();
    client.ensure_genesis_in_place().await.unwrap();

    (client, keystore)
}

pub async fn create_test_user_transport(
    mock_node: Arc<RwLock<MockNoteTransportNode>>,
) -> (MockClient<FilesystemKeyStore>, Account) {
    let (mut client, keystore) = Box::pin(create_test_client_transport(mock_node.clone())).await;
    let account = insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();
    (client, account)
}
