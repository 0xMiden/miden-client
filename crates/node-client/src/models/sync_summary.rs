use miden_client::sync::SyncSummary as NativeSyncSummary;

use super::account_id::AccountId;
use super::napi_wrap;
use super::note_id::NoteId;
use super::transaction_id::TransactionId;

napi_wrap!(owned SyncSummary wraps NativeSyncSummary);

#[napi]
impl SyncSummary {
    /// Returns the block height the summary is based on.
    #[napi(js_name = "blockNum")]
    pub fn block_num(&self) -> u32 {
        self.0.block_num.as_u32()
    }

    /// Returns IDs of notes committed in this sync window.
    #[napi(js_name = "committedNotes")]
    pub fn committed_notes(&self) -> Vec<NoteId> {
        self.0.committed_notes.iter().map(Into::into).collect()
    }

    /// Returns IDs of notes that were consumed.
    #[napi(js_name = "consumedNotes")]
    pub fn consumed_notes(&self) -> Vec<NoteId> {
        self.0.consumed_notes.iter().map(Into::into).collect()
    }

    /// Returns accounts that were updated.
    #[napi(js_name = "updatedAccounts")]
    pub fn updated_accounts(&self) -> Vec<AccountId> {
        self.0.updated_accounts.iter().map(Into::into).collect()
    }

    /// Returns transactions that were committed.
    #[napi(js_name = "committedTransactions")]
    pub fn committed_transactions(&self) -> Vec<TransactionId> {
        self.0.committed_transactions.iter().map(Into::into).collect()
    }
}
