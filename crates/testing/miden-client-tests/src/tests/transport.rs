use std::sync::Arc;

use miden_client::account::{Account, AccountStorageMode};
use miden_client::address::{Address, AddressInterface, RoutingParameters};
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::{NoteAttachment, NoteType};
use miden_client::store::NoteFilter;
use miden_client::testing::mock::MockClient;
use miden_client::testing::note_transport::{MockNoteTransportApi, MockNoteTransportNode};
use miden_client::utils::RwLock;
use miden_standards::note::create_p2id_note;

use crate::tests::{create_test_client_builder, insert_new_wallet};

#[tokio::test]
async fn transport_basic() {
    // Setup entities
    let mock_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));
    let (mut sender, sender_account) = create_test_user_transport(mock_node.clone()).await;
    let (mut recipient, recipient_account) = create_test_user_transport(mock_node.clone()).await;
    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet))
        .unwrap();
    let (mut observer, _observer_account) = create_test_user_transport(mock_node.clone()).await;

    // Create note
    let note = create_p2id_note(
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
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet))
        .unwrap();

    let note_a = create_p2id_note(
        sender_account.id(),
        recipient_account.id(),
        vec![],
        NoteType::Private,
        NoteAttachment::default(),
        sender.rng(),
    )
    .unwrap();

    let note_b = create_p2id_note(
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
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet))
        .unwrap();

    let note = create_p2id_note(
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
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet))
        .unwrap();
    let (mut observer, _observer_account) = create_test_user_transport(mock_node.clone()).await;

    let note = create_p2id_note(
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
