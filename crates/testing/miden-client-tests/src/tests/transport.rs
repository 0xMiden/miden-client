use std::env::temp_dir;
use std::sync::Arc;

use miden_client::DebugMode;
use miden_client::account::{Account, AccountStorageMode};
use miden_client::address::{Address, AddressInterface, RoutingParameters};
use miden_client::builder::ClientBuilder;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::{NoteAttachment, NoteDetails, NoteTag, NoteType};
use miden_client::note_transport::NoteTransportClient;
use miden_client::store::NoteFilter;
use miden_client::testing::common::create_test_store_path;
use miden_client::testing::mock::{MockClient, MockRpcApi};
use miden_client::testing::note_transport::{
    FaultyNoteTransportApi,
    MockNoteTransportApi,
    MockNoteTransportNode,
};
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

/// Reproduction + fix-gate for the silent-loss failure mode observed in the
/// 2026-04-27 stress run.
///
/// **Background.** When `Client::send_private_note` is called, the chain
/// transaction has already committed (sender debited). The relay step calls
/// `NoteTransportClient::send_note` exactly once. If that call fails, the
/// payload is discarded — there is no outbox, no retry, no persistence — and
/// the recipient never learns about the note. The sender's vault stays
/// debited; the receiver gets nothing.
///
/// **What this test asserts.** The recipient must eventually receive the
/// note even when the sender's first relay attempt fails, as long as the NTL
/// itself recovers. The test does not constrain the fix's shape: the fix may
/// (a) retry inline inside `send_private_note`, (b) piggyback retries on
/// `sync_state`, or (c) expose an explicit `flush_relay_outbox` call. The
/// polling loop below tolerates all three by alternating sender/recipient
/// `sync_state` calls until the note arrives or the budget is exhausted.
///
/// **Status.** Fails on `origin/main` (commit a2491e02). Passes once any
/// durable-relay strategy is in place.
#[tokio::test]
async fn private_note_relay_recovers_after_transient_ntl_failure() {
    let mock_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));

    // Fail the next send_note attempt, then recover. Models a single transient
    // NTL hiccup — the failure mode the stress run encountered repeatedly
    // under load (lock contention, page reloads cancelling in-flight relays).
    let faulty = Arc::new(FaultyNoteTransportApi::new(mock_node.clone(), 1));
    let (mut sender, sender_account) =
        create_test_user_with_transport(faulty.clone() as Arc<dyn NoteTransportClient>).await;
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
    let note_id = note.id();

    // First relay attempt — the faulty NTL rejects it. We don't assert on the
    // return value: the fix may make `send_private_note` itself retry and
    // return Ok, or it may surface the Err and rely on a later retry path.
    let _ = sender.send_private_note(note, &recipient_address).await;

    // Drive both clients forward. Either the sender's retry path runs inside
    // sync_state (or as a side effect of fetch_private_notes), or it ran
    // synchronously inside the send_private_note call. Either way the note
    // must reach the recipient within a small number of rounds.
    let mut delivered = false;
    for _ in 0..5 {
        let _ = sender.sync_state().await;
        recipient.sync_state().await.unwrap();
        let received = recipient.get_input_notes(NoteFilter::All).await.unwrap();
        if received.iter().any(|n| n.id() == note_id) {
            delivered = true;
            break;
        }
    }

    assert!(
        delivered,
        "BUG (stress-20260427): a single transient NTL failure permanently loses a private \
         note — sender debited, recipient never learns of it. send_attempts={}",
        faulty.send_attempts()
    );

    // Sanity: the fix must actually retry the relay — a single attempt that
    // succeeded by chance is not durability.
    assert!(
        faulty.send_attempts() >= 2,
        "fix must retry the relay; observed only {} send_note attempt(s)",
        faulty.send_attempts()
    );
}

/// Tightens the durability contract beyond the previous test:
/// `flush_relay_outbox` is a public, sync_state-independent retry path, the
/// outbox entry survives a failed `send_private_note` until that retry
/// succeeds, and a successful retry removes the entry so it isn't re-sent.
///
/// Why a separate test from `private_note_relay_recovers_after_transient_ntl_failure`:
/// that one is intentionally lenient about the recovery mechanism —
/// inline retry, sync_state-piggyback, or explicit flush all satisfy it.
/// This one nails down the explicit-flush contract callers depend on
/// when they want to drain the outbox without paying for a full sync
/// cycle (e.g. after a connectivity-restored event).
#[tokio::test]
async fn flush_relay_outbox_retries_failed_relay_without_full_sync() {
    let mock_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));
    let faulty = Arc::new(FaultyNoteTransportApi::new(mock_node.clone(), 1));
    let (mut sender, sender_account) =
        create_test_user_with_transport(faulty.clone() as Arc<dyn NoteTransportClient>).await;
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
    let note_id = note.id();

    // 1. First attempt: the faulty NTL rejects, send_private_note surfaces the error. The chain
    //    side has nothing to roll back here (this test isolates the relay step), so the only
    //    durability requirement is that the payload survive the failed call.
    let first_attempt = sender.send_private_note(note, &recipient_address).await;
    assert!(
        first_attempt.is_err(),
        "expected NTL failure on first attempt, got {first_attempt:?}",
    );
    assert_eq!(
        faulty.send_attempts(),
        1,
        "send_private_note should have made exactly one transport attempt",
    );

    // 2. Recipient syncs — nothing to deliver yet because the NTL never got the payload. Pins the
    //    contract: the sender's outbox is the only surviving copy.
    recipient.sync_state().await.unwrap();
    let received = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert!(
        received.iter().all(|n| n.id() != note_id),
        "recipient should not yet see the note (NTL was empty after the failed relay)",
    );

    // 3. Caller drives the retry explicitly — no `sync_state` round-trip. The faulty NTL has used
    //    up its single rejection (fail_next: 1) so this attempt will succeed.
    sender.flush_relay_outbox().await.expect("flush_relay_outbox should succeed");
    assert!(
        faulty.send_attempts() >= 2,
        "explicit flush must re-attempt the relay; observed only {} send_note attempt(s)",
        faulty.send_attempts(),
    );

    // 4. Recipient syncs and now sees the note — proves the explicit flush actually delivered
    //    through the NTL, not just dropped the entry.
    recipient.sync_state().await.unwrap();
    let received = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert!(
        received.iter().any(|n| n.id() == note_id),
        "recipient must receive the note after the explicit flush retried the relay",
    );

    // 5. A second flush is a no-op: the outbox entry was removed when the retry succeeded. Without
    //    this property, every sync would re-send every previously-relayed note (the receiver
    //    dedups, but the wasted NTL traffic is still a regression).
    let attempts_after_first_flush = faulty.send_attempts();
    sender.flush_relay_outbox().await.expect("second flush should succeed (no-op)");
    assert_eq!(
        faulty.send_attempts(),
        attempts_after_first_flush,
        "outbox should be empty after a successful flush; second flush must not re-send",
    );
}

// HELPERS
// ================================================================================================

pub async fn create_test_client_transport(
    mock_node: Arc<RwLock<MockNoteTransportNode>>,
) -> (MockClient<FilesystemKeyStore>, FilesystemKeyStore) {
    create_test_client_with_transport(Arc::new(MockNoteTransportApi::new(mock_node))).await
}

pub async fn create_test_user_transport(
    mock_node: Arc<RwLock<MockNoteTransportNode>>,
) -> (MockClient<FilesystemKeyStore>, Account) {
    let (mut client, keystore) = Box::pin(create_test_client_transport(mock_node)).await;
    let account = insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();
    (client, account)
}

pub async fn create_test_client_with_transport(
    transport: Arc<dyn NoteTransportClient>,
) -> (MockClient<FilesystemKeyStore>, FilesystemKeyStore) {
    let (builder, _, keystore) = create_test_client_builder().await;
    let builder_w_transport = builder.note_transport(transport);

    let mut client = builder_w_transport.build().await.unwrap();
    client.ensure_genesis_in_place().await.unwrap();

    (client, keystore)
}

pub async fn create_test_user_with_transport(
    transport: Arc<dyn NoteTransportClient>,
) -> (MockClient<FilesystemKeyStore>, Account) {
    let (mut client, keystore) = Box::pin(create_test_client_with_transport(transport)).await;
    let account = insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();
    (client, account)
}
