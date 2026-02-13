use miden_client::sync::SyncSummary as NativeSyncSummary;

use super::account_id::AccountId;
use super::note_id::NoteId;
use super::transaction_id::TransactionId;
use super::{napi_delegate, napi_wrap};

napi_wrap!(owned SyncSummary wraps NativeSyncSummary);

napi_delegate!(impl SyncSummary {
    /// Returns IDs of notes committed in this sync window.
    collect committed_notes -> Vec<NoteId>;
    /// Returns IDs of notes that were consumed.
    collect consumed_notes -> Vec<NoteId>;
    /// Returns accounts that were updated.
    collect updated_accounts -> Vec<AccountId>;
    /// Returns transactions that were committed.
    collect committed_transactions -> Vec<TransactionId>;
});

#[napi]
impl SyncSummary {
    /// Returns the block height the summary is based on.
    #[napi]
    pub fn block_num(&self) -> u32 {
        self.0.block_num.as_u32()
    }
}
