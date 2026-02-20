use miden_client::sync::SyncSummary as NativeSyncSummary;

use crate::prelude::*;

use crate::models::account_id::AccountId;
use crate::models::note_id::NoteId;
use crate::models::transaction_id::TransactionId;

/// Contains stats about the sync operation.
#[bindings]
pub struct SyncSummary(NativeSyncSummary);

// Shared methods
#[bindings]
impl SyncSummary {
    /// Returns the block height the summary is based on.
    pub fn block_num(&self) -> u32 {
        self.0.block_num.as_u32()
    }

    /// Returns IDs of notes committed in this sync window.
    #[bindings]
    pub fn committed_notes(&self) -> Vec<NoteId> {
        self.0.committed_notes.iter().map(Into::into).collect()
    }

    /// Returns IDs of notes that were consumed.
    #[bindings]
    pub fn consumed_notes(&self) -> Vec<NoteId> {
        self.0.consumed_notes.iter().map(Into::into).collect()
    }

    /// Returns accounts that were updated.
    #[bindings]
    pub fn updated_accounts(&self) -> Vec<AccountId> {
        self.0.updated_accounts.iter().map(Into::into).collect()
    }

    /// Returns transactions that were committed.
    #[bindings]
    pub fn committed_transactions(&self) -> Vec<TransactionId> {
        self.0.committed_transactions.iter().map(Into::into).collect()
    }

    /// Serializes the sync summary into bytes.
    pub fn serialize(&self) -> JsBytes {
        platform::serialize_to_bytes(&self.0)
    }

    /// Deserializes a sync summary from bytes.
    #[bindings(factory)]
    pub fn deserialize(bytes: &JsBytes) -> JsResult<SyncSummary> {
        platform::deserialize_from_bytes::<NativeSyncSummary>(bytes).map(SyncSummary)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeSyncSummary> for SyncSummary {
    fn from(native_sync_summary: NativeSyncSummary) -> Self {
        SyncSummary(native_sync_summary)
    }
}
