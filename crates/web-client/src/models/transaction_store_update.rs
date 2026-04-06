use js_export_macro::js_export;
use miden_client::transaction::TransactionStoreUpdate as NativeTransactionStoreUpdate;

use crate::models::account_delta::AccountDelta;
use crate::models::executed_transaction::ExecutedTransaction;
use crate::models::output_notes::OutputNotes;
use crate::models::transaction_request::note_details_and_tag::NoteDetailsAndTag;
use crate::platform::{JsBytes, JsErr};
use crate::utils::{deserialize_from_bytes, serialize_to_bytes};

/// Represents the changes that need to be applied to the client store as a result of a transaction
/// execution.
#[derive(Clone)]
#[js_export]
pub struct TransactionStoreUpdate(NativeTransactionStoreUpdate);

#[js_export]
impl TransactionStoreUpdate {
    /// Returns the executed transaction associated with this update.
    #[js_export(js_name = "executedTransaction")]
    pub fn executed_transaction(&self) -> ExecutedTransaction {
        self.0.executed_transaction().into()
    }

    /// Returns the block height at which the transaction was submitted.
    #[js_export(js_name = "submissionHeight")]
    pub fn submission_height(&self) -> u32 {
        self.0.submission_height().as_u32()
    }

    /// Returns the output notes created by the transaction.
    #[js_export(js_name = "createdNotes")]
    pub fn created_notes(&self) -> OutputNotes {
        self.0.executed_transaction().output_notes().into()
    }

    /// Returns the account delta applied by the transaction.
    #[js_export(js_name = "accountDelta")]
    pub fn account_delta(&self) -> AccountDelta {
        self.0.executed_transaction().account_delta().into()
    }

    /// Returns notes expected to be created in follow-up executions.
    #[js_export(js_name = "futureNotes")]
    pub fn future_notes(&self) -> Vec<NoteDetailsAndTag> {
        self.0
            .future_notes()
            .iter()
            .cloned()
            .map(|(details, tag)| NoteDetailsAndTag::new(details.into(), tag.into()))
            .collect()
    }

    /// Serializes the update into bytes.
    pub fn serialize(&self) -> JsBytes {
        serialize_to_bytes(&self.0)
    }

    /// Deserializes an update from bytes.
    pub fn deserialize(bytes: JsBytes) -> Result<TransactionStoreUpdate, JsErr> {
        deserialize_from_bytes::<NativeTransactionStoreUpdate>(&bytes).map(TransactionStoreUpdate)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeTransactionStoreUpdate> for TransactionStoreUpdate {
    fn from(update: NativeTransactionStoreUpdate) -> Self {
        TransactionStoreUpdate(update)
    }
}

impl From<&NativeTransactionStoreUpdate> for TransactionStoreUpdate {
    fn from(update: &NativeTransactionStoreUpdate) -> Self {
        TransactionStoreUpdate(update.clone())
    }
}

impl From<&TransactionStoreUpdate> for NativeTransactionStoreUpdate {
    fn from(update: &TransactionStoreUpdate) -> Self {
        update.0.clone()
    }
}

impl From<TransactionStoreUpdate> for NativeTransactionStoreUpdate {
    fn from(update: TransactionStoreUpdate) -> Self {
        update.0
    }
}
