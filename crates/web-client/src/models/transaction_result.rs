use miden_client::transaction::TransactionResult as NativeTransactionResult;
use js_export_macro::js_export;

use crate::models::executed_transaction::ExecutedTransaction;
use crate::models::transaction_id::TransactionId;
use crate::models::transaction_request::note_details_and_tag::NoteDetailsAndTag;
use crate::platform::{JsBytes, JsErr};
use crate::utils::{deserialize_from_bytes, serialize_to_bytes};

/// Represents the result of executing a transaction by the client.
///
/// It contains an `ExecutedTransaction`, and a list of `future_notes`
/// that we expect to receive in the future (you can check at swap notes for an example of this).
#[derive(Clone)]
#[js_export]
pub struct TransactionResult {
    result: NativeTransactionResult,
}

#[js_export]
impl TransactionResult {
    /// Returns the ID of the transaction.
    pub fn id(&self) -> TransactionId {
        self.result.id().into()
    }

    /// Returns the executed transaction.
    #[js_export(js_name = "executedTransaction")]
    pub fn executed_transaction(&self) -> ExecutedTransaction {
        self.result.executed_transaction().clone().into()
    }

    /// Returns notes that are expected to be created as a result of follow-up executions.
    #[js_export(js_name = "futureNotes")]
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

    /// Serializes the transaction result into bytes.
    pub fn serialize(&self) -> JsBytes {
        serialize_to_bytes(&self.result)
    }

    /// Deserializes a transaction result from bytes.
    pub fn deserialize(bytes: JsBytes) -> Result<TransactionResult, JsErr> {
        deserialize_from_bytes::<NativeTransactionResult>(&bytes).map(TransactionResult::from)
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
