use std::sync::Arc;

use miden_client::note::{
    NoteAssets, NoteMetadata, NoteRecipient, NoteStorage, NoteTag, NoteType, OutputNoteReader,
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
    let metadata = NoteMetadata::new(sender, NoteType::Public, tag);
    let assets = NoteAssets::new(vec![]).unwrap();
    let serial_number: Word = [Felt::new(u64::from(index)), ZERO, ZERO, ZERO].into();
    let recipient = NoteRecipient::new(
        serial_number,
        StandardNote::SWAP.script(),
        NoteStorage::new(vec![]).unwrap(),
    );
    let recipient_digest = recipient.digest().into();
    OutputNoteRecord::new(
        recipient_digest,
        assets,
        metadata,
        OutputNoteState::Consumed {
            block_height: BlockNumber::from(1u32),
            recipient,
        },
        BlockNumber::from(0u32),
    )
}

/// Insert output notes into the store using a raw connection.
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
