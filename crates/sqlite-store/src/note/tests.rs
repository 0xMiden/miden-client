use std::sync::Arc;

use miden_client::note::{
    NoteAssets,
    NoteMetadata,
    NoteRecipient,
    NoteStorage,
    NoteTag,
    NoteType,
    OutputNoteReader,
};
use miden_client::store::{NoteFilter, OutputNoteRecord, OutputNoteState, Store};
use miden_client::{Felt, ZERO};
use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;
use miden_protocol::testing::account_id::{
    ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
    ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE,
};
use miden_standards::note::StandardNote;

use crate::tests::create_test_store;

/// Helper to create a test `OutputNoteRecord` in `ExpectedPartial` state with a unique recipient
/// digest.
fn create_test_output_note(sender: AccountId, tag: NoteTag, index: u32) -> OutputNoteRecord {
    let metadata = NoteMetadata::new(sender, NoteType::Public, tag);
    let assets = NoteAssets::new(vec![]).unwrap();
    // Use a unique recipient digest per note so they get distinct IDs.
    let recipient_digest: Word = [Felt::new(u64::from(index)), ZERO, ZERO, ZERO].into();
    OutputNoteRecord::new(
        recipient_digest,
        assets,
        metadata,
        OutputNoteState::ExpectedPartial,
        BlockNumber::from(0u32),
    )
}

/// Helper to create a test `OutputNoteRecord` in `Consumed` state with a unique recipient digest.
fn create_consumed_output_note(sender: AccountId, tag: NoteTag, index: u32) -> OutputNoteRecord {
    create_consumed_output_note_at_height(sender, tag, index, 1)
}

/// Helper to create a test `OutputNoteRecord` in `Consumed` state with a specific block height.
fn create_consumed_output_note_at_height(
    sender: AccountId,
    tag: NoteTag,
    index: u32,
    block_height: u32,
) -> OutputNoteRecord {
    let metadata = NoteMetadata::new(sender, NoteType::Public, tag);
    let assets = NoteAssets::new(vec![]).unwrap();
    let serial_number: Word = [Felt::new(u64::from(index)), ZERO, ZERO, ZERO].into();
    let recipient = NoteRecipient::new(
        serial_number,
        StandardNote::SWAP.script(),
        NoteStorage::new(vec![]).unwrap(),
    );
    let recipient_digest = recipient.digest();
    OutputNoteRecord::new(
        recipient_digest,
        assets,
        metadata,
        OutputNoteState::Consumed {
            block_height: BlockNumber::from(block_height),
            recipient,
        },
        BlockNumber::from(0u32),
    )
}

/// Insert output notes into the store using a raw connection.
async fn insert_output_notes(store: &crate::SqliteStore, notes: &[OutputNoteRecord]) {
    insert_output_notes_with_tx_order(store, notes, None).await;
}

/// Insert output notes into the store using a raw connection, with a specific consumed tx order.
async fn insert_output_notes_with_tx_order(
    store: &crate::SqliteStore,
    notes: &[OutputNoteRecord],
    consumed_tx_order: Option<u32>,
) {
    let notes = notes.to_vec();
    store
        .interact_with_connection(move |conn| {
            let tx = conn
                .transaction()
                .map_err(|e| miden_client::store::StoreError::QueryError(e.to_string()))?;
            for note in &notes {
                super::upsert_output_note_tx(&tx, note, consumed_tx_order)?;
            }
            tx.commit()
                .map_err(|e| miden_client::store::StoreError::QueryError(e.to_string()))
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn output_note_reader_returns_none_on_empty_store() {
    let store = create_test_store().await;
    let store: Arc<dyn Store> = Arc::new(store);

    let mut reader = OutputNoteReader::new(store);
    let result = reader.next().await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn output_note_reader_iterates_all_consumed_notes() {
    let store = create_test_store().await;
    let sender = AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();

    let notes: Vec<_> = (0..3u32)
        .map(|i| create_consumed_output_note(sender, NoteTag::from(100 + i), i))
        .collect();
    insert_output_notes(&store, &notes).await;

    let store: Arc<dyn Store> = Arc::new(store);
    let mut reader = OutputNoteReader::new(store);

    let mut collected = Vec::new();
    while let Some(note) = reader.next().await.unwrap() {
        collected.push(note);
    }

    assert_eq!(collected.len(), 3);
}

#[tokio::test]
async fn output_note_reader_skips_non_consumed_notes() {
    let store = create_test_store().await;
    let sender = AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();

    // Insert 2 consumed notes and 1 expected note.
    let notes = vec![
        create_consumed_output_note(sender, NoteTag::from(100u32), 0),
        create_test_output_note(sender, NoteTag::from(101u32), 1),
        create_consumed_output_note(sender, NoteTag::from(102u32), 2),
    ];
    insert_output_notes(&store, &notes).await;

    let store: Arc<dyn Store> = Arc::new(store);
    let mut reader = OutputNoteReader::new(store);

    let mut collected = Vec::new();
    while let Some(note) = reader.next().await.unwrap() {
        collected.push(note);
    }

    // Only the 2 consumed notes should be returned.
    assert_eq!(collected.len(), 2);
}

#[tokio::test]
async fn output_note_reader_filters_by_sender() {
    let store = create_test_store().await;
    let sender_a = AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();
    let sender_b = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET).unwrap();

    // Insert 2 consumed notes from sender_a and 1 from sender_b.
    let notes = vec![
        create_consumed_output_note(sender_a, NoteTag::from(201u32), 10),
        create_consumed_output_note(sender_b, NoteTag::from(200u32), 11),
        create_consumed_output_note(sender_a, NoteTag::from(202u32), 12),
    ];
    insert_output_notes(&store, &notes).await;

    let store: Arc<dyn Store> = Arc::new(store);
    let mut reader = OutputNoteReader::new(store).for_sender(sender_a);

    let mut collected = Vec::new();
    while let Some(note) = reader.next().await.unwrap() {
        collected.push(note);
    }

    assert_eq!(collected.len(), 2);
    for note in &collected {
        assert_eq!(note.metadata().sender(), sender_a);
    }
}

#[tokio::test]
async fn output_note_reader_reset_restarts_iteration() {
    let store = create_test_store().await;
    let sender = AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();

    let notes: Vec<_> = (0..2u32)
        .map(|i| create_consumed_output_note(sender, NoteTag::from(300 + i), 20 + i))
        .collect();
    insert_output_notes(&store, &notes).await;

    let store: Arc<dyn Store> = Arc::new(store);
    let mut reader = OutputNoteReader::new(store);

    // Read the first note.
    let first = reader.next().await.unwrap().unwrap();

    // Reset and re-read — should get the same first note.
    reader.reset();
    let after_reset = reader.next().await.unwrap().unwrap();

    assert_eq!(first.id(), after_reset.id());
}

#[tokio::test]
async fn get_output_note_by_offset_returns_correct_note() {
    let store = create_test_store().await;
    let sender = AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();

    let notes: Vec<_> = (0..3u32)
        .map(|i| create_test_output_note(sender, NoteTag::from(500 + i), 40 + i))
        .collect();
    insert_output_notes(&store, &notes).await;

    // Get all notes to know the expected order.
    let all_notes = store.get_output_notes(NoteFilter::All).await.unwrap();
    assert_eq!(all_notes.len(), 3);

    // Verify each offset returns the correct note.
    for (i, expected) in all_notes.iter().enumerate() {
        let note = store
            .get_output_note_by_offset(NoteFilter::All, None, u32::try_from(i).unwrap())
            .await
            .unwrap();
        assert_eq!(note.as_ref().unwrap().id(), expected.id());
    }

    // Offset past the end returns None.
    let none = store.get_output_note_by_offset(NoteFilter::All, None, 3).await.unwrap();
    assert!(none.is_none());
}

#[tokio::test]
async fn consumed_notes_ordered_by_block_height_then_tx_order() {
    let store = create_test_store().await;
    let sender = AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();

    // Create consumed notes at different block heights.
    let note_block3 = create_consumed_output_note_at_height(sender, NoteTag::from(100u32), 0, 3);
    let note_block1 = create_consumed_output_note_at_height(sender, NoteTag::from(101u32), 1, 1);
    let note_block2 = create_consumed_output_note_at_height(sender, NoteTag::from(102u32), 2, 2);

    // Insert in non-sorted order, each with a tx_order.
    insert_output_notes_with_tx_order(&store, std::slice::from_ref(&note_block3), Some(0)).await;
    insert_output_notes_with_tx_order(&store, std::slice::from_ref(&note_block1), Some(1)).await;
    insert_output_notes_with_tx_order(&store, std::slice::from_ref(&note_block2), Some(0)).await;

    // Retrieve consumed notes — should be ordered by block_height ASC, tx_order ASC.
    let notes = store.get_output_notes(NoteFilter::Consumed).await.unwrap();
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].id(), note_block1.id()); // block 1, tx_order 1
    assert_eq!(notes[1].id(), note_block2.id()); // block 2, tx_order 0
    assert_eq!(notes[2].id(), note_block3.id()); // block 3, tx_order 0
}

#[tokio::test]
async fn consumed_notes_same_block_ordered_by_tx_order() {
    let store = create_test_store().await;
    let sender = AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();

    // All notes consumed at the same block height, different tx_order.
    let note_tx2 = create_consumed_output_note_at_height(sender, NoteTag::from(200u32), 10, 5);
    let note_tx0 = create_consumed_output_note_at_height(sender, NoteTag::from(201u32), 11, 5);
    let note_tx1 = create_consumed_output_note_at_height(sender, NoteTag::from(202u32), 12, 5);

    insert_output_notes_with_tx_order(&store, std::slice::from_ref(&note_tx2), Some(2)).await;
    insert_output_notes_with_tx_order(&store, std::slice::from_ref(&note_tx0), Some(0)).await;
    insert_output_notes_with_tx_order(&store, std::slice::from_ref(&note_tx1), Some(1)).await;

    let notes = store.get_output_notes(NoteFilter::Consumed).await.unwrap();
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].id(), note_tx0.id()); // tx_order 0
    assert_eq!(notes[1].id(), note_tx1.id()); // tx_order 1
    assert_eq!(notes[2].id(), note_tx2.id()); // tx_order 2
}

#[tokio::test]
async fn consumed_notes_null_tx_order_sort_last_within_block() {
    let store = create_test_store().await;
    let sender = AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();

    // Two notes at the same block: one with tx_order, one without (external consumption).
    let note_with_order =
        create_consumed_output_note_at_height(sender, NoteTag::from(300u32), 20, 5);
    let note_without_order =
        create_consumed_output_note_at_height(sender, NoteTag::from(301u32), 21, 5);

    insert_output_notes_with_tx_order(&store, std::slice::from_ref(&note_with_order), Some(0))
        .await;
    insert_output_notes_with_tx_order(&store, std::slice::from_ref(&note_without_order), None)
        .await;

    let notes = store.get_output_notes(NoteFilter::Consumed).await.unwrap();
    assert_eq!(notes.len(), 2);
    // Note with tx_order should come first (non-NULL sorts before NULL in ASC).
    assert_eq!(notes[0].id(), note_with_order.id());
    assert_eq!(notes[1].id(), note_without_order.id());
}
