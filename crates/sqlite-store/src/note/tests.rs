use std::sync::Arc;

use miden_client::note::{
    InputNoteReader,
    NoteAssets,
    NoteMetadata,
    NoteReader,
    NoteReaderSource,
    NoteRecipient,
    NoteRecord,
    NoteStorage,
    NoteTag,
    NoteType,
    OutputNoteReader,
};
use miden_client::store::input_note_states::{
    ConsumedExternalNoteState,
    ConsumedUnauthenticatedLocalNoteState,
    ExpectedNoteState,
    NoteSubmissionData,
};
use miden_client::store::{InputNoteRecord, NoteFilter, OutputNoteRecord, OutputNoteState, Store};
use miden_client::{Felt, ZERO};
use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::NoteDetails;
use miden_protocol::testing::account_id::{
    ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
    ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE,
};
use miden_protocol::transaction::TransactionId;
use miden_standards::note::StandardNote;

use crate::tests::create_test_store;

// HELPERS
// ================================================================================================

/// Helper to create a consumed-external input note (no consumer account).
fn create_consumed_external_input_note(index: u32, block_height: u32) -> InputNoteRecord {
    let serial_number: Word = [Felt::new(u64::from(index) + 2000), ZERO, ZERO, ZERO].into();
    let assets = NoteAssets::new(vec![]).unwrap();
    let recipient = NoteRecipient::new(
        serial_number,
        StandardNote::SWAP.script(),
        NoteStorage::new(vec![]).unwrap(),
    );
    let details = NoteDetails::new(assets, recipient);

    let state = ConsumedExternalNoteState {
        nullifier_block_height: BlockNumber::from(block_height),
    };

    InputNoteRecord::new(details, Some(0), state.into())
}

/// Helper to create an expected (non-consumed) input note.
fn create_expected_input_note(index: u32) -> InputNoteRecord {
    let serial_number: Word = [Felt::new(u64::from(index) + 3000), ZERO, ZERO, ZERO].into();
    let assets = NoteAssets::new(vec![]).unwrap();
    let recipient = NoteRecipient::new(
        serial_number,
        StandardNote::SWAP.script(),
        NoteStorage::new(vec![]).unwrap(),
    );
    let details = NoteDetails::new(assets, recipient);

    let state = ExpectedNoteState {
        metadata: None,
        after_block_num: BlockNumber::from(0u32),
        tag: None,
    };

    InputNoteRecord::new(details, Some(0), state.into())
}

/// Helper to create a consumed-unauthenticated-local input note with a specific consumer.
fn create_consumed_input_note_with_consumer(
    consumer: AccountId,
    index: u32,
    block_height: u32,
) -> InputNoteRecord {
    let serial_number: Word = [Felt::new(u64::from(index) + 5000), ZERO, ZERO, ZERO].into();
    let assets = NoteAssets::new(vec![]).unwrap();
    let recipient = NoteRecipient::new(
        serial_number,
        StandardNote::SWAP.script(),
        NoteStorage::new(vec![]).unwrap(),
    );
    let details = NoteDetails::new(assets, recipient);

    let metadata = NoteMetadata::new(consumer, NoteType::Public).with_tag(NoteTag::from(index));

    let state = ConsumedUnauthenticatedLocalNoteState {
        metadata,
        nullifier_block_height: BlockNumber::from(block_height),
        submission_data: NoteSubmissionData {
            submitted_at: Some(0),
            consumer_account: consumer,
            consumer_transaction: TransactionId::from_raw(Word::default()),
        },
    };

    InputNoteRecord::new(details, Some(0), state.into())
}

/// Insert input notes into the store with an optional `consumed_tx_order`.
async fn insert_input_notes_with_tx_order(
    store: &crate::SqliteStore,
    notes: &[InputNoteRecord],
    consumed_tx_order: Option<u32>,
) {
    let notes = notes.to_vec();
    store
        .interact_with_connection(move |conn| {
            let tx = conn
                .transaction()
                .map_err(|e| miden_client::store::StoreError::QueryError(e.to_string()))?;
            for note in &notes {
                super::upsert_input_note_tx(&tx, note, consumed_tx_order)?;
            }
            tx.commit()
                .map_err(|e| miden_client::store::StoreError::QueryError(e.to_string()))
        })
        .await
        .unwrap();
}

fn create_expected_output_note(index: u32, expected_height: u32) -> OutputNoteRecord {
    let serial_number: Word = [Felt::new(u64::from(index) + 7000), ZERO, ZERO, ZERO].into();
    let assets = NoteAssets::new(vec![]).unwrap();
    let recipient = NoteRecipient::new(
        serial_number,
        StandardNote::SWAP.script(),
        NoteStorage::new(vec![]).unwrap(),
    );
    let sender = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET).unwrap();
    let metadata = NoteMetadata::new(sender, NoteType::Public).with_tag(NoteTag::from(index));

    OutputNoteRecord::new(
        recipient.digest(),
        assets,
        metadata,
        OutputNoteState::ExpectedFull { recipient },
        BlockNumber::from(expected_height),
    )
}

async fn insert_output_notes(store: &crate::SqliteStore, notes: &[OutputNoteRecord]) {
    let notes = notes.to_vec();
    store
        .interact_with_connection(move |conn| {
            let tx = conn
                .transaction()
                .map_err(|e| miden_client::store::StoreError::QueryError(e.to_string()))?;
            for note in &notes {
                super::upsert_output_note_tx(&tx, note)?;
            }
            tx.commit()
                .map_err(|e| miden_client::store::StoreError::QueryError(e.to_string()))
        })
        .await
        .unwrap();
}

// INPUT NOTE READER TESTS
// ================================================================================================

#[tokio::test]
async fn input_note_reader_returns_none_on_empty_store() {
    let store = create_test_store().await;
    let store: Arc<dyn Store> = Arc::new(store);

    let mut reader = InputNoteReader::new(store);
    let result = reader.next().await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn input_note_reader_iterates_all_consumed_notes() {
    let store = create_test_store().await;

    let notes: Vec<_> = (0..3u32).map(|i| create_consumed_external_input_note(i, 1)).collect();
    store.upsert_input_notes(&notes).await.unwrap();

    let store: Arc<dyn Store> = Arc::new(store);
    let mut reader = InputNoteReader::new(store);

    let mut collected = Vec::new();
    while let Some(note) = reader.next().await.unwrap() {
        collected.push(note);
    }

    assert_eq!(collected.len(), 3);
}

#[tokio::test]
async fn input_note_reader_skips_non_consumed_notes() {
    let store = create_test_store().await;

    // Insert 2 consumed notes and 1 expected note.
    let consumed1 = create_consumed_external_input_note(0, 1);
    let expected = create_expected_input_note(1);
    let consumed2 = create_consumed_external_input_note(2, 1);

    store.upsert_input_notes(&[consumed1, expected, consumed2]).await.unwrap();

    let store: Arc<dyn Store> = Arc::new(store);
    let mut reader = InputNoteReader::new(store);

    let mut collected = Vec::new();
    while let Some(note) = reader.next().await.unwrap() {
        collected.push(note);
    }

    // Only the 2 consumed notes should be returned.
    assert_eq!(collected.len(), 2);
}

#[tokio::test]
async fn input_note_reader_with_filter_iterates_expected_notes() {
    let store = create_test_store().await;

    let consumed = create_consumed_external_input_note(0, 1);
    let expected_a = create_expected_input_note(1);
    let expected_b = create_expected_input_note(2);

    store
        .upsert_input_notes(&[consumed, expected_a.clone(), expected_b.clone()])
        .await
        .unwrap();

    let store: Arc<dyn Store> = Arc::new(store);
    let mut reader = InputNoteReader::new(store).with_filter(NoteFilter::Expected);

    let mut collected = Vec::new();
    while let Some(note) = reader.next().await.unwrap() {
        collected.push(note);
    }

    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0].id(), expected_a.id());
    assert_eq!(collected[1].id(), expected_b.id());
}

#[tokio::test]
async fn output_note_reader_iterates_expected_output_notes() {
    let store = create_test_store().await;

    let note_a = create_expected_output_note(0, 5);
    let note_b = create_expected_output_note(1, 7);
    insert_output_notes(&store, &[note_a.clone(), note_b.clone()]).await;

    let store: Arc<dyn Store> = Arc::new(store);
    let mut reader = OutputNoteReader::new(store).with_filter(NoteFilter::Expected);

    let mut collected = Vec::new();
    while let Some(note) = reader.next().await.unwrap() {
        collected.push(note);
    }

    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0].id(), note_a.id());
    assert_eq!(collected[1].id(), note_b.id());
}

#[tokio::test]
async fn generic_note_reader_yields_output_record_variants() {
    let store = create_test_store().await;

    let note = create_expected_output_note(0, 9);
    insert_output_notes(&store, core::slice::from_ref(&note)).await;

    let store: Arc<dyn Store> = Arc::new(store);
    let mut reader =
        NoteReader::new(store, NoteReaderSource::Output).with_filter(NoteFilter::Expected);

    let next = reader.next().await.unwrap();

    assert!(matches!(next, Some(NoteRecord::Output(output)) if output.id() == note.id()));
}

#[tokio::test]
async fn input_note_reader_filters_by_consumer() {
    let store = create_test_store().await;
    let consumer_a =
        AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();
    let consumer_b = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET).unwrap();

    // Two notes for consumer_a with tx_order, one for consumer_b with tx_order.
    let note_a1 = create_consumed_input_note_with_consumer(consumer_a, 10, 1);
    let note_b = create_consumed_input_note_with_consumer(consumer_b, 11, 1);
    let note_a2 = create_consumed_input_note_with_consumer(consumer_a, 12, 1);

    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_a1), Some(0)).await;
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_b), Some(0)).await;
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_a2), Some(1)).await;

    let store: Arc<dyn Store> = Arc::new(store);
    let mut reader = InputNoteReader::new(store).for_consumer(consumer_a);

    let mut collected = Vec::new();
    while let Some(note) = reader.next().await.unwrap() {
        collected.push(note);
    }

    assert_eq!(collected.len(), 2);
    for note in &collected {
        assert_eq!(note.consumer_account(), Some(consumer_a));
    }
}

#[tokio::test]
async fn input_note_reader_excludes_notes_without_tx_order_when_consumer_is_set() {
    let store = create_test_store().await;
    let consumer = AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();

    // Insert two notes for the same consumer: one with tx_order, one without.
    let note_with_order = create_consumed_input_note_with_consumer(consumer, 30, 1);
    let note_without_order = create_consumed_input_note_with_consumer(consumer, 31, 1);

    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_with_order), Some(0)).await;
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_without_order), None).await;

    let store: Arc<dyn Store> = Arc::new(store);
    let mut reader = InputNoteReader::new(store).for_consumer(consumer);

    let mut collected = Vec::new();
    while let Some(note) = reader.next().await.unwrap() {
        collected.push(note);
    }

    // Only the note with tx_order should be returned.
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].id(), note_with_order.id());
}

#[tokio::test]
async fn input_note_reader_filters_by_block_range() {
    let store = create_test_store().await;

    // Create consumed notes at different block heights.
    let note_b1 = create_consumed_external_input_note(0, 1);
    let note_b3 = create_consumed_external_input_note(1, 3);
    let note_b5 = create_consumed_external_input_note(2, 5);
    let note_b7 = create_consumed_external_input_note(3, 7);

    store
        .upsert_input_notes(&[note_b1, note_b3.clone(), note_b5.clone(), note_b7])
        .await
        .unwrap();

    let store: Arc<dyn Store> = Arc::new(store);

    // Filter to blocks 3..=5
    let mut reader = InputNoteReader::new(store)
        .in_block_range(BlockNumber::from(3u32), BlockNumber::from(5u32));

    let mut collected = Vec::new();
    while let Some(note) = reader.next().await.unwrap() {
        collected.push(note);
    }

    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0].id(), note_b3.id());
    assert_eq!(collected[1].id(), note_b5.id());
}

#[tokio::test]
async fn input_note_reader_filters_by_consumer_and_block_range() {
    let store = create_test_store().await;
    let consumer_a =
        AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();
    let consumer_b = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET).unwrap();

    // consumer_a at blocks 1, 3, 5; consumer_b at block 3.
    let alice_at_1 = create_consumed_input_note_with_consumer(consumer_a, 20, 1);
    let alice_at_3 = create_consumed_input_note_with_consumer(consumer_a, 21, 3);
    let bob_at_3 = create_consumed_input_note_with_consumer(consumer_b, 22, 3);
    let alice_at_5 = create_consumed_input_note_with_consumer(consumer_a, 23, 5);

    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&alice_at_1), Some(0)).await;
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&alice_at_3), Some(0)).await;
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&bob_at_3), Some(1)).await;
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&alice_at_5), Some(0)).await;

    let store: Arc<dyn Store> = Arc::new(store);

    // Filter to consumer_a in blocks 3..=5 — should return alice_at_3 and alice_at_5 only.
    let mut reader = InputNoteReader::new(store)
        .for_consumer(consumer_a)
        .in_block_range(BlockNumber::from(3u32), BlockNumber::from(5u32));

    let mut collected = Vec::new();
    while let Some(note) = reader.next().await.unwrap() {
        collected.push(note);
    }

    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0].id(), alice_at_3.id());
    assert_eq!(collected[1].id(), alice_at_5.id());
    for note in &collected {
        assert_eq!(note.consumer_account(), Some(consumer_a));
    }
}

// ORDERING TESTS (INPUT NOTES)
// ================================================================================================

#[tokio::test]
async fn consumed_input_notes_ordered_by_block_height_then_tx_order() {
    let store = create_test_store().await;

    // Create consumed notes at different block heights.
    let note_block3 = create_consumed_external_input_note(0, 3);
    let note_block1 = create_consumed_external_input_note(1, 1);
    let note_block2 = create_consumed_external_input_note(2, 2);

    // Insert in non-sorted order, each with a tx_order.
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_block3), Some(0)).await;
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_block1), Some(1)).await;
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_block2), Some(0)).await;

    // Retrieve consumed notes — should be ordered by block_height ASC, tx_order ASC.
    let notes = store.get_input_notes(NoteFilter::Consumed).await.unwrap();
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].id(), note_block1.id()); // block 1, tx_order 1
    assert_eq!(notes[1].id(), note_block2.id()); // block 2, tx_order 0
    assert_eq!(notes[2].id(), note_block3.id()); // block 3, tx_order 0
}

#[tokio::test]
async fn consumed_input_notes_same_block_ordered_by_tx_order() {
    let store = create_test_store().await;

    // All notes consumed at the same block height, different tx_order.
    let note_tx2 = create_consumed_external_input_note(10, 5);
    let note_tx0 = create_consumed_external_input_note(11, 5);
    let note_tx1 = create_consumed_external_input_note(12, 5);

    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_tx2), Some(2)).await;
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_tx0), Some(0)).await;
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_tx1), Some(1)).await;

    let notes = store.get_input_notes(NoteFilter::Consumed).await.unwrap();
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].id(), note_tx0.id()); // tx_order 0
    assert_eq!(notes[1].id(), note_tx1.id()); // tx_order 1
    assert_eq!(notes[2].id(), note_tx2.id()); // tx_order 2
}

#[tokio::test]
async fn consumed_input_notes_null_tx_order_sort_last_within_block() {
    let store = create_test_store().await;

    // Two notes at the same block: one with tx_order, one without (external consumption).
    let note_with_order = create_consumed_external_input_note(20, 5);
    let note_without_order = create_consumed_external_input_note(21, 5);

    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_with_order), Some(0)).await;
    insert_input_notes_with_tx_order(&store, std::slice::from_ref(&note_without_order), None).await;

    let notes = store.get_input_notes(NoteFilter::Consumed).await.unwrap();
    assert_eq!(notes.len(), 2);
    // Note with tx_order should come first (non-NULL sorts before NULL in ASC).
    assert_eq!(notes[0].id(), note_with_order.id());
    assert_eq!(notes[1].id(), note_without_order.id());
}
