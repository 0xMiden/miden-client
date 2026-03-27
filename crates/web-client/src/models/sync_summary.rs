use js_export_macro::js_export;
use miden_client::sync::SyncSummary as NativeSyncSummary;

use crate::models::account_id::AccountId;
use crate::models::note_id::NoteId;
use crate::models::transaction_id::TransactionId;
use crate::platform::{JsBytes, JsErr};
use crate::utils::{deserialize_from_bytes, serialize_to_bytes};

/// Contains stats about the sync operation.
#[js_export]
pub struct SyncSummary(NativeSyncSummary);

#[js_export]
impl SyncSummary {
    /// Returns the block height the summary is based on.
    #[js_export(js_name = "blockNum")]
    pub fn block_num(&self) -> u32 {
        self.0.block_num.as_u32()
    }

    /// Returns IDs of notes committed in this sync window.
    #[js_export(js_name = "committedNotes")]
    pub fn committed_notes(&self) -> Vec<NoteId> {
        self.0.committed_notes.iter().map(Into::into).collect()
    }

    /// Returns IDs of notes that were consumed.
    #[js_export(js_name = "consumedNotes")]
    pub fn consumed_notes(&self) -> Vec<NoteId> {
        self.0.consumed_notes.iter().map(Into::into).collect()
    }

    /// Returns accounts that were updated.
    #[js_export(js_name = "updatedAccounts")]
    pub fn updated_accounts(&self) -> Vec<AccountId> {
        self.0.updated_accounts.iter().map(Into::into).collect()
    }

    /// Returns transactions that were committed.
    #[js_export(js_name = "committedTransactions")]
    pub fn committed_transactions(&self) -> Vec<TransactionId> {
        self.0.committed_transactions.iter().map(Into::into).collect()
    }

    /// Serializes the sync summary into bytes.
    pub fn serialize(&self) -> JsBytes {
        serialize_to_bytes(&self.0)
    }

    /// Deserializes a sync summary from bytes.
    pub fn deserialize(bytes: JsBytes) -> Result<SyncSummary, JsErr> {
        deserialize_from_bytes::<NativeSyncSummary>(&bytes).map(SyncSummary)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeSyncSummary> for SyncSummary {
    fn from(native_sync_summary: NativeSyncSummary) -> Self {
        SyncSummary(native_sync_summary)
    }
}
