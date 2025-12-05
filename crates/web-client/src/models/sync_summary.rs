use miden_client::sync::SyncSummary as NativeSyncSummary;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::account_id::AccountId;
use crate::models::note_id::NoteId;
use crate::models::transaction_id::TransactionId;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Contains stats about the sync operation.
#[wasm_bindgen]
pub struct SyncSummary(NativeSyncSummary);

#[wasm_bindgen]
impl SyncSummary {
    /// Returns the block height the summary is based on.
    #[wasm_bindgen(js_name = "blockNum")]
    pub fn block_num(&self) -> u32 {
        self.0.block_num.as_u32()
    }

    /// Returns IDs of notes committed in this sync window.
    #[wasm_bindgen(js_name = "committedNotes")]
    pub fn committed_notes(&self) -> Vec<NoteId> {
        self.0.committed_notes.iter().map(Into::into).collect()
    }

    /// Returns IDs of notes that were consumed.
    #[wasm_bindgen(js_name = "consumedNotes")]
    pub fn consumed_notes(&self) -> Vec<NoteId> {
        self.0.consumed_notes.iter().map(Into::into).collect()
    }

    /// Returns accounts that were updated.
    #[wasm_bindgen(js_name = "updatedAccounts")]
    pub fn updated_accounts(&self) -> Vec<AccountId> {
        self.0.updated_accounts.iter().map(Into::into).collect()
    }

    /// Returns transactions that were committed.
    #[wasm_bindgen(js_name = "committedTransactions")]
    pub fn committed_transactions(&self) -> Vec<TransactionId> {
        self.0.committed_transactions.iter().map(Into::into).collect()
    }

    /// Serializes the sync summary into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a sync summary from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<SyncSummary, JsValue> {
        deserialize_from_uint8array::<NativeSyncSummary>(bytes).map(SyncSummary)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeSyncSummary> for SyncSummary {
    fn from(native_sync_summary: NativeSyncSummary) -> Self {
        SyncSummary(native_sync_summary)
    }
}
