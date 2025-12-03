use miden_client::transaction::TransactionStoreUpdate as NativeTransactionStoreUpdate;
use wasm_bindgen::prelude::{wasm_bindgen, *};
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::account_delta::AccountDelta;
use crate::models::executed_transaction::ExecutedTransaction;
use crate::models::output_notes::OutputNotes;
use crate::models::transaction_request::note_details_and_tag::NoteDetailsAndTag;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Update describing the effects of a stored transaction.
#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionStoreUpdate(NativeTransactionStoreUpdate);

#[wasm_bindgen]
impl TransactionStoreUpdate {
    /// Returns the executed transaction associated with this update.
    #[wasm_bindgen(js_name = "executedTransaction")]
    pub fn executed_transaction(&self) -> ExecutedTransaction {
        self.0.executed_transaction().into()
    }

    /// Returns the block height at which the transaction was submitted.
    #[wasm_bindgen(js_name = "submissionHeight")]
    pub fn submission_height(&self) -> u32 {
        self.0.submission_height().as_u32()
    }

    /// Returns the notes created by the transaction.
    #[wasm_bindgen(js_name = "createdNotes")]
    pub fn created_notes(&self) -> OutputNotes {
        self.0.executed_transaction().output_notes().into()
    }

    /// Returns the account delta applied by the transaction.
    #[wasm_bindgen(js_name = "accountDelta")]
    pub fn account_delta(&self) -> AccountDelta {
        self.0.executed_transaction().account_delta().into()
    }

    /// Returns notes expected to be created in follow-up executions.
    #[wasm_bindgen(js_name = "futureNotes")]
    pub fn future_notes(&self) -> Vec<NoteDetailsAndTag> {
        self.0
            .future_notes()
            .iter()
            .cloned()
            .map(|(details, tag)| NoteDetailsAndTag::new(details.into(), tag.into()))
            .collect()
    }

    /// Serializes the update into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes an update from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<TransactionStoreUpdate, JsValue> {
        deserialize_from_uint8array::<NativeTransactionStoreUpdate>(bytes)
            .map(TransactionStoreUpdate)
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
