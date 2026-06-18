use std::env::temp_dir;
use std::sync::Arc;

use miden_client::DebugMode;
use miden_client::account::{Account, AccountType};
use miden_client::address::{Address, AddressInterface, RoutingParameters};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::builder::ClientBuilder;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::{Note, NoteAttachments, NoteDetails, NoteTag, NoteType};
use miden_client::note_transport::{NoteTransportClient, NoteTransportCursor};
use miden_client::store::NoteFilter;
use miden_client::testing::common::create_test_store_path;
use miden_client::testing::mock::{MockClient, MockRpcApi};
use miden_client::testing::note_transport::{
    FaultyNoteTransportApi,
    MockNoteTransportApi,
    MockNoteTransportNode,
};
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::utils::RwLock;
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_protocol::Felt;
use miden_protocol::asset::FungibleAsset;
use miden_protocol::block::BlockNumber;
use miden_protocol::crypto::rand::RandomCoin;
use miden_protocol::note::NoteType as ProtocolNoteType;
use miden_protocol::transaction::RawOutputNote;
use miden_protocol::utils::serde::Serializable;
use miden_standards::note::P2idNote;
use miden_standards::testing::note::NoteBuilder;
use miden_testing::{MockChain, MockChainBuilder, TxContextInput};
use rand::Rng;

use crate::tests::{create_test_client_builder, insert_new_wallet, setup_wallet_and_faucet};

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
        NoteAttachments::empty(),
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
        NoteAttachments::empty(),
        sender.rng(),
    )
    .unwrap();

    let note_b = P2idNote::create(
        sender_account.id(),
        recipient_account.id(),
        vec![],
        NoteType::Private,
        NoteAttachments::empty(),
        sender.rng(),
    )
    .unwrap();

    // Send note A, sync → recipient receives 1 note
    sender.send_private_note(note_a.clone(), &recipient_address).await.unwrap();
    recipient.sync_state().await.unwrap();
    let notes = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(notes.len(), 1, "should have 1 note after first sync");
    // The note is delivered via the transport layer and isn't committed on-chain, so it has no
    // metadata (and thus no `NoteId`); it's identified by its details commitment.
    assert_eq!(notes[0].details_commitment(), note_a.details_commitment());

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
        NoteAttachments::empty(),
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

/// Verifies that `fetch_all_private_notes` drains notes across multiple
/// server-paginated batches.
///
/// Regression test for the interaction between the transport server's
/// response-size `LIMIT` and the client's previously-single-shot
/// `fetch_all_private_notes`. Before the drain loop, a server cap of N per
/// response meant `fetch_all_private_notes` silently returned only the first
/// N notes and the rest were invisible until the next paginated sync tick.
#[tokio::test]
async fn fetch_all_private_notes_drains_across_batches() {
    const BATCH_CAP: usize = 3;
    const TOTAL_NOTES: usize = 10;

    let mock_node = Arc::new(RwLock::new(MockNoteTransportNode::with_max_batch(BATCH_CAP)));
    let (mut sender, sender_account) = create_test_user_transport(mock_node.clone()).await;
    let (mut recipient, recipient_account) = create_test_user_transport(mock_node.clone()).await;
    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet));

    // Send TOTAL_NOTES > BATCH_CAP private notes so a single-batch fetch
    // cannot drain the backlog.
    for _ in 0..TOTAL_NOTES {
        let note = P2idNote::create(
            sender_account.id(),
            recipient_account.id(),
            vec![],
            NoteType::Private,
            NoteAttachments::empty(),
            sender.rng(),
        )
        .unwrap();
        sender.send_private_note(note, &recipient_address).await.unwrap();
    }

    // With BATCH_CAP=3 and TOTAL_NOTES=10, a single-shot fetch would return
    // only 3. The drain loop should issue successive calls until all 10 are
    // pulled.
    recipient.fetch_all_private_notes().await.unwrap();

    let notes = recipient.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(
        notes.len(),
        TOTAL_NOTES,
        "fetch_all_private_notes must drain across batches; got {} of {}",
        notes.len(),
        TOTAL_NOTES
    );
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
        NoteAttachments::empty(),
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

    let private_note = NoteBuilder::new(
        mock_account.id(),
        RandomCoin::new([1, 2, 3, 4].map(Felt::new_unchecked).into()),
    )
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
    let rng = RandomCoin::new(coin_seed.map(|v| Felt::new_unchecked(v >> 1)).into());

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
    // No after_block_num hint, so the receiver must fall back to its lookback window.
    mock_transport_node
        .write()
        .add_note(*private_note.header(), details_bytes, None);

    // 6. Second sync_state: fetch_transport_notes imports the note, then chain sync runs.
    // Without the fix, after_block_num = sync_height, scan misses the note at block 1.
    // With the fix, lookback window catches it.
    let summary = client.sync_state().await.unwrap();
    assert!(
        summary.new_private_notes.contains(&private_note.id()),
        "summary should report the NTL-imported note in new_private_notes"
    );

    // 7. The note should be Committed after the second sync.
    let committed_notes = client.get_input_notes(NoteFilter::Committed).await.unwrap();
    assert!(
        committed_notes.iter().any(|n| n.id() == Some(private_note.id())),
        "note committed before sync_height should be found via lookback during NTL import"
    );
}

/// Builds a mock chain that commits a private note (tag 0) near genesis, then proves
/// `extra_blocks` further blocks so the note's commitment sits that far behind the chain tip.
/// Returns the built chain and the committed note.
async fn commit_private_note_then_advance(extra_blocks: usize) -> (MockChain, Note) {
    let mut mock_chain_builder = MockChainBuilder::new();
    let mock_account = mock_chain_builder
        .add_existing_mock_account(miden_testing::Auth::IncrNonce)
        .unwrap();

    let private_note = NoteBuilder::new(
        mock_account.id(),
        RandomCoin::new([1, 2, 3, 4].map(Felt::new_unchecked).into()),
    )
    .note_type(ProtocolNoteType::Private)
    .tag(NoteTag::new(0).into())
    .build()
    .unwrap();

    let spawn_note =
        mock_chain_builder.add_spawn_note(std::slice::from_ref(&private_note)).unwrap();
    let mut mock_chain = mock_chain_builder.build().unwrap();

    // Commit the private note.
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

    for _ in 0..extra_blocks {
        mock_chain.prove_next_block().unwrap();
    }

    (mock_chain, private_note)
}

/// Builds a client over `rpc_api` + `mock_transport_node`, syncs it to the chain tip, and
/// registers tag 0 so chain sync sees the note's block. The NTL is not drained here.
async fn build_synced_ntl_client(
    rpc_api: Arc<MockRpcApi>,
    mock_transport_node: Arc<RwLock<MockNoteTransportNode>>,
) -> MockClient<FilesystemKeyStore> {
    let transport_client = MockNoteTransportApi::new(mock_transport_node);

    let mut rng = rand::rng();
    let coin_seed: [u64; 4] = rng.random();
    let rng = RandomCoin::new(coin_seed.map(|v| Felt::new_unchecked(v >> 1)).into());

    let keystore = FilesystemKeyStore::new(temp_dir()).unwrap();

    let builder: ClientBuilder<FilesystemKeyStore> = ClientBuilder::new()
        .rpc(rpc_api)
        .rng(Box::new(rng))
        .sqlite_store(create_test_store_path())
        .authenticator(Arc::new(keystore))
        .in_debug_mode(DebugMode::Enabled)
        .tx_discard_delta(None)
        .note_transport(Arc::new(transport_client));

    let mut client = builder.build().await.unwrap();
    client.ensure_genesis_in_place().await.unwrap();
    client.add_note_tag(NoteTag::new(0)).await.unwrap();
    client.sync_state().await.unwrap();
    client
}

/// A note whose commitment sits more than the lookback window (20 blocks) behind the
/// recipient's sync height is missed by the lookback alone. The sender-provided
/// `after_block_num` hint lowers the scan floor to (at or below) the commitment block so the
/// note still commits on import.
#[tokio::test]
async fn after_block_num_hint_commits_note_beyond_lookback() {
    let (mock_chain, private_note) = commit_private_note_then_advance(30).await;
    let mock_transport_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));
    let mut client =
        build_synced_ntl_client(Arc::new(MockRpcApi::new(mock_chain)), mock_transport_node.clone())
            .await;

    let sync_height = client.get_sync_height().await.unwrap();
    assert!(
        sync_height.as_u32() > 21,
        "note must sit more than the lookback window behind the tip"
    );

    // Deliver the note with a hint at genesis — a valid lower bound on its commitment block.
    let details_bytes = NoteDetails::from(private_note.clone()).to_bytes();
    mock_transport_node.write().add_note(
        *private_note.header(),
        details_bytes,
        Some(BlockNumber::from(0)),
    );

    client.sync_state().await.unwrap();

    let note_details_commitment = NoteDetails::from(private_note.clone()).commitment();
    let committed = client.get_input_notes(NoteFilter::Committed).await.unwrap();
    assert!(
        committed.iter().any(|n| n.details_commitment() == note_details_commitment),
        "the after_block_num hint should lower the scan floor so a note beyond the lookback \
         window still commits on import"
    );
}

/// Control for [`after_block_num_hint_commits_note_beyond_lookback`]: without the hint the same
/// note stays `Expected`, proving the hint — not the lookback — is what commits it.
#[tokio::test]
async fn without_hint_note_beyond_lookback_stays_expected() {
    let (mock_chain, private_note) = commit_private_note_then_advance(30).await;
    let mock_transport_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));
    let mut client =
        build_synced_ntl_client(Arc::new(MockRpcApi::new(mock_chain)), mock_transport_node.clone())
            .await;

    let details_bytes = NoteDetails::from(private_note.clone()).to_bytes();
    mock_transport_node
        .write()
        .add_note(*private_note.header(), details_bytes, None);

    client.sync_state().await.unwrap();

    // Expected notes carry no metadata and thus no NoteId, so match by details commitment.
    let note_details_commitment = NoteDetails::from(private_note.clone()).commitment();
    let committed = client.get_input_notes(NoteFilter::Committed).await.unwrap();
    assert!(
        !committed.iter().any(|n| n.details_commitment() == note_details_commitment),
        "without a hint, a note beyond the lookback window must not commit on import"
    );
    let expected = client.get_input_notes(NoteFilter::Expected).await.unwrap();
    assert!(
        expected.iter().any(|n| n.details_commitment() == note_details_commitment),
        "the note should remain Expected (imported but not yet observed on-chain)"
    );
}

/// The one-shot import scan can miss a note's on-chain commitment when the node returns an
/// incomplete (but successful) `sync_notes` response. Forward chain sync only revisits blocks
/// ahead of the last sync height, so without a retry the note would stay `Expected` forever. A
/// subsequent sync re-scans still-`Expected` notes from their stored floor and recovers it.
#[tokio::test]
async fn expected_note_rescan_recovers_from_incomplete_import_scan() {
    // The note is committed within the lookback window, so the only thing keeping it from
    // committing on import is the dropped scan — not the scan floor.
    let (mock_chain, private_note) = commit_private_note_then_advance(3).await;
    let mock_transport_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));
    let rpc_api = Arc::new(MockRpcApi::new(mock_chain));
    let mut client = build_synced_ntl_client(rpc_api.clone(), mock_transport_node.clone()).await;

    // The import-time scan (check_expected_notes -> sync_notes) returns an incomplete response.
    rpc_api.drop_next_sync_notes(1);

    let details_bytes = NoteDetails::from(private_note.clone()).to_bytes();
    mock_transport_node
        .write()
        .add_note(*private_note.header(), details_bytes, None);

    let note_details_commitment = NoteDetails::from(private_note.clone()).commitment();

    // First sync imports the note, but the dropped scan leaves it Expected.
    client.sync_state().await.unwrap();
    let committed = client.get_input_notes(NoteFilter::Committed).await.unwrap();
    assert!(
        !committed.iter().any(|n| n.details_commitment() == note_details_commitment),
        "an incomplete import scan should leave the note Expected"
    );

    // Second sync: the rescan re-checks the still-Expected note, and the now-complete scan
    // commits it.
    client.sync_state().await.unwrap();
    let committed = client.get_input_notes(NoteFilter::Committed).await.unwrap();
    assert!(
        committed.iter().any(|n| n.details_commitment() == note_details_commitment),
        "a subsequent sync should rescan the Expected note and commit it"
    );
}

/// Exercises the real sender path: a note created by the client's own transaction is relayed via
/// `send_private_note`, and the relayed `after_block_num` equals the note's stored
/// `expected_height` (a safe lower bound on its commitment block). Guards against the sender
/// silently relaying no hint, which would reintroduce the lost-note bug for notes committed more
/// than the lookback window before delivery.
#[tokio::test]
async fn send_private_note_relays_output_note_expected_height() {
    let mock_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));
    let (mut client, keystore) = create_test_client_transport(mock_node.clone()).await;
    let (wallet, faucet) =
        setup_wallet_and_faucet(&mut client, AccountType::Private, &keystore, RPO_FALCON_SCHEME_ID)
            .await
            .unwrap();

    // Mint a private note from the faucet to the wallet. This records an output note with
    // expected_height = the transaction's submission height.
    let asset = FungibleAsset::new(faucet.id(), 100).unwrap();
    let tx_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(asset, wallet.id(), NoteType::Private, client.rng())
        .unwrap();
    let note = tx_request.expected_output_own_notes().first().unwrap().clone();
    Box::pin(client.submit_new_transaction(faucet.id(), tx_request)).await.unwrap();

    let expected_height = client
        .get_output_notes(NoteFilter::List(vec![note.id()]))
        .await
        .unwrap()
        .first()
        .unwrap()
        .expected_height();

    // Relay the note. The recipient address is irrelevant to the hint.
    let address = Address::new(wallet.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet));
    client.send_private_note(note.clone(), &address).await.unwrap();

    // The NTL should have stored the note with after_block_num = the note's expected_height.
    let (relayed, _) = mock_node
        .read()
        .get_notes(&[note.metadata().tag()], NoteTransportCursor::init());
    let entry = relayed
        .iter()
        .find(|info| info.header.id() == note.id())
        .expect("relayed note should be present in the NTL");
    assert_eq!(
        entry.after_block_num,
        Some(expected_height),
        "sender should relay the output note's expected_height as the after_block_num hint"
    );
}

/// A private note must reach the recipient even when the sender's first relay
/// attempt fails, provided the transport later recovers.
///
/// Without the durable outbox, `send_private_note` relays the payload exactly
/// once; if that call fails the payload is dropped (no retry, no persistence)
/// and the recipient never learns about the note. The outbox makes the relay
/// retriable, so a transient transport failure no longer loses the note.
///
/// The test doesn't constrain the fix's shape (inline retry, retry on
/// `sync_state`, or an explicit `flush_relay_outbox`): it polls by alternating
/// sender/recipient `sync_state` calls until the note arrives or the budget is
/// exhausted.
#[tokio::test]
async fn private_note_relay_recovers_after_transient_ntl_failure() {
    let mock_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));

    // Fail the next send_note attempt, then recover — a single transient
    // transport failure.
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
        NoteAttachments::empty(),
        sender.rng(),
    )
    .unwrap();
    // Transport-delivered notes carry no metadata (hence no `NoteId`); match by
    // details commitment.
    let note_commitment = note.details_commitment();

    // First relay attempt — the faulty NTL rejects it. We don't assert on the
    // return value: the relay may fail here and be retried later.
    let _ = sender.send_private_note(note, &recipient_address).await;

    // Drive both clients forward; the retry must deliver the note within a few
    // rounds.
    let mut delivered = false;
    for _ in 0..5 {
        let _ = sender.sync_state().await;
        recipient.sync_state().await.unwrap();
        let received = recipient.get_input_notes(NoteFilter::All).await.unwrap();
        if received.iter().any(|n| n.details_commitment() == note_commitment) {
            delivered = true;
            break;
        }
    }

    assert!(
        delivered,
        "a single transient NTL failure permanently lost a private note — sender debited, \
         recipient never learns of it. send_attempts={}",
        faulty.send_attempts()
    );

    // The fix must actually retry the relay — a single attempt that succeeded
    // by chance is not durability.
    assert!(
        faulty.send_attempts() >= 2,
        "fix must retry the relay; observed only {} send_note attempt(s)",
        faulty.send_attempts()
    );
}

/// The durable outbox entry survives a failed `send_private_note` and is
/// re-sent by an explicit `flush_relay_outbox`, without a full sync. A second
/// flush is a no-op once the entry has drained.
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
        NoteAttachments::empty(),
        sender.rng(),
    )
    .unwrap();
    // Transport-delivered notes carry no metadata (hence no `NoteId`); match by
    // details commitment.
    let note_commitment = note.details_commitment();

    // First relay fails; the payload must survive in the outbox.
    let first_attempt = sender.send_private_note(note, &recipient_address).await;
    assert!(
        first_attempt.is_err(),
        "expected NTL failure on first attempt, got {first_attempt:?}"
    );

    // Recipient sees nothing yet — the NTL never received the note.
    recipient.sync_state().await.unwrap();
    assert!(
        recipient.get_input_notes(NoteFilter::All).await.unwrap().is_empty(),
        "recipient should not yet see the note (NTL was empty after the failed relay)",
    );

    // Explicit flush re-sends (the faulty API has used up its single rejection).
    sender.flush_relay_outbox().await.expect("flush should re-send the queued note");
    assert!(faulty.send_attempts() >= 2, "flush must re-attempt the relay");

    recipient.sync_state().await.unwrap();
    assert!(
        recipient
            .get_input_notes(NoteFilter::All)
            .await
            .unwrap()
            .iter()
            .any(|n| n.details_commitment() == note_commitment),
        "recipient should receive the note after the flush re-send",
    );

    // A second flush is a no-op: the entry was removed when the retry succeeded.
    let attempts_after_first_flush = faulty.send_attempts();
    sender.flush_relay_outbox().await.expect("second flush should succeed (no-op)");
    assert_eq!(
        faulty.send_attempts(),
        attempts_after_first_flush,
        "outbox should be empty after a successful flush; second flush must not re-send",
    );
}

/// A relay that keeps failing must not block `sync_state`. The outbox flush
/// runs at the start of the transport step; if its error propagated, a single
/// undeliverable note would wedge every subsequent sync. The entry must stay in
/// the outbox for later retry while the sync itself succeeds.
#[tokio::test]
async fn persistent_relay_failure_does_not_block_sync_state() {
    let mock_node = Arc::new(RwLock::new(MockNoteTransportNode::new()));

    // Fail effectively forever, modelling a note the NTL never accepts.
    let faulty = Arc::new(FaultyNoteTransportApi::new(mock_node.clone(), usize::MAX));
    let (mut sender, sender_account) =
        create_test_user_with_transport(faulty.clone() as Arc<dyn NoteTransportClient>).await;
    let (_recipient, recipient_account) = create_test_user_transport(mock_node.clone()).await;
    let recipient_address = Address::new(recipient_account.id())
        .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet));

    let note = P2idNote::create(
        sender_account.id(),
        recipient_account.id(),
        vec![],
        NoteType::Private,
        NoteAttachments::empty(),
        sender.rng(),
    )
    .unwrap();

    // The relay fails and the payload is persisted to the outbox.
    let _ = sender.send_private_note(note, &recipient_address).await;

    // sync_state flushes the outbox (which fails) but must still complete: the
    // relay failure is logged, not propagated.
    sender
        .sync_state()
        .await
        .expect("sync_state must not fail when an outbox entry can't be relayed");

    // The undeliverable entry is retained for a future attempt, not dropped.
    let direct = sender.flush_relay_outbox().await;
    assert!(
        direct.is_err(),
        "directly flushing an undeliverable entry should surface the error"
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
    let account = insert_new_wallet(&mut client, AccountType::Private, &keystore).await.unwrap();
    (client, account)
}

pub async fn create_test_client_with_transport(
    transport: Arc<dyn NoteTransportClient>,
) -> (MockClient<FilesystemKeyStore>, FilesystemKeyStore) {
    let (builder, _, keystore) = create_test_client_builder().await;
    let mut client = builder.note_transport(transport).build().await.unwrap();
    client.ensure_genesis_in_place().await.unwrap();
    (client, keystore)
}

pub async fn create_test_user_with_transport(
    transport: Arc<dyn NoteTransportClient>,
) -> (MockClient<FilesystemKeyStore>, Account) {
    let (mut client, keystore) = Box::pin(create_test_client_with_transport(transport)).await;
    let account = insert_new_wallet(&mut client, AccountType::Private, &keystore).await.unwrap();
    (client, account)
}
