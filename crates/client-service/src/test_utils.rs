use miden_client::sync::SyncSummary;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::NoteId;

pub(crate) fn test_note_id() -> NoteId {
    NoteId::from_raw(miden_protocol::EMPTY_WORD)
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
