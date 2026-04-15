use std::sync::Arc;

use miden_client::store::InputNoteRecord;
use miden_client::sync::SyncSummary;
use miden_client::testing::NoteBuilder;
use miden_client::testing::account_id::ACCOUNT_ID_SENDER;
use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::NoteId;
use rand::SeedableRng;
use rand::rngs::SmallRng;

/// Returns a deterministic [`InputNoteRecord`] built from a fixed sender and RNG seed.
///
/// All calls to this function produce a record with the same [`NoteId`], which is what
/// [`test_note_id`] returns.
pub(crate) fn test_note_record() -> InputNoteRecord {
    let sender = AccountId::try_from(ACCOUNT_ID_SENDER).expect("valid sender id");
    // Fixed seed → deterministic serial_num → stable NoteId across calls.
    let rng = SmallRng::seed_from_u64(0x00c0_ffee);
    let note = NoteBuilder::new(sender, rng).build().expect("build test note");
    InputNoteRecord::from(note)
}

pub(crate) fn test_note_arc() -> Arc<InputNoteRecord> {
    Arc::new(test_note_record())
}

pub(crate) fn test_note_id() -> NoteId {
    test_note_record().id()
}

pub(crate) fn empty_summary() -> SyncSummary {
    SyncSummary {
        block_num: BlockNumber::from(0u32),
        new_public_notes: vec![],
        committed_notes: vec![],
        consumed_notes: vec![],
        updated_accounts: vec![],
        locked_accounts: vec![],
        committed_transactions: vec![],
    }
}
