use miden_client::BlockNumber;
use miden_client::transaction::TransactionResult as NativeTransactionResult;
use wasm_bindgen::prelude::*;

use crate::models::executed_transaction::ExecutedTransaction;
use crate::models::transaction_id::TransactionId;
use crate::models::transaction_request::note_details_and_tag::NoteDetailsAndTag;
use crate::models::transaction_store_update::TransactionStoreUpdate;

/// WASM wrapper around the native [`TransactionResult`].
#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionResult {
    result: NativeTransactionResult,
}

#[wasm_bindgen]
impl TransactionResult {
    /// Returns the ID of the transaction.
    pub fn id(&self) -> TransactionId {
        self.result.id().into()
    }

    /// Returns the executed transaction.
    #[wasm_bindgen(js_name = "executedTransaction")]
    pub fn executed_transaction(&self) -> ExecutedTransaction {
        self.result.executed_transaction().clone().into()
    }

    /// Returns notes that are expected to be created as a result of follow-up executions.
    #[wasm_bindgen(js_name = "futureNotes")]
    pub fn future_notes(&self) -> Vec<NoteDetailsAndTag> {
        self.result
            .future_notes()
            .iter()
            .cloned()
            .map(|(note_details, note_tag)| {
                NoteDetailsAndTag::new(note_details.into(), note_tag.into())
            })
            .collect()
    }

    /// Builds a store update using the provided submission height.
    #[wasm_bindgen(js_name = "transactionUpdateWithHeight")]
    pub fn transaction_update_with_height(&self, submission_height: u32) -> TransactionStoreUpdate {
        self.result.to_transaction_update(BlockNumber::from(submission_height)).into()
    }
}

impl TransactionResult {
    pub(crate) fn new(result: NativeTransactionResult) -> Self {
        Self { result }
    }

    pub(crate) fn native(&self) -> &NativeTransactionResult {
        &self.result
    }
}

impl From<NativeTransactionResult> for TransactionResult {
    fn from(result: NativeTransactionResult) -> Self {
        Self::new(result)
    }
}
