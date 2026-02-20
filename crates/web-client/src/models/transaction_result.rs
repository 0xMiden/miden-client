use miden_client::transaction::TransactionResult as NativeTransactionResult;
#[cfg(feature = "napi")]
use miden_client::{Deserializable, Serializable};
#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::executed_transaction::ExecutedTransaction;
use crate::models::transaction_id::TransactionId;
use crate::prelude::*;
#[cfg(feature = "wasm")]
use crate::models::transaction_request::note_details_and_tag::NoteDetailsAndTag;
#[cfg(feature = "wasm")]
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Represents the result of executing a transaction by the client.
///
/// It contains an `ExecutedTransaction`, and a list of `future_notes`
/// that we expect to receive in the future (you can check at swap notes for an example of this).
#[bindings]
#[derive(Clone)]
pub struct TransactionResult {
    result: NativeTransactionResult,
}

#[bindings]
impl TransactionResult {
    /// Returns the ID of the transaction.
    pub fn id(&self) -> TransactionId {
        self.result.id().into()
    }

    /// Returns the executed transaction.
    #[bindings(wasm(js_name = "executedTransaction"))]
    pub fn executed_transaction(&self) -> ExecutedTransaction {
        self.result.executed_transaction().clone().into()
    }
}

#[cfg(feature = "wasm")]
#[bindings(wasm)]
impl TransactionResult {
    /// Returns notes that are expected to be created as a result of follow-up executions.
    #[bindings(wasm(js_name = "futureNotes"))]
    pub fn future_notes(&self) -> Vec<NoteDetailsAndTag> {
        self.result
            .future_notes()
            .iter()
            .cloned()
            .map(|(note_details, note_tag)| {
                let details: crate::models::note_details::NoteDetails = note_details.into();
                let tag: crate::models::note_tag::NoteTag = note_tag.into();
                NoteDetailsAndTag::new(&details, &tag)
            })
            .collect()
    }

    /// Serializes the transaction result into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.result)
    }

    /// Deserializes a transaction result from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> JsResult<TransactionResult> {
        deserialize_from_uint8array::<NativeTransactionResult>(bytes).map(TransactionResult::from)
    }
}

#[cfg(feature = "napi")]
#[bindings(napi)]
impl TransactionResult {
    /// Serializes the transaction result into bytes.
    pub fn serialize(&self) -> Buffer {
        let bytes = self.result.to_bytes();
        Buffer::from(bytes)
    }

    /// Deserializes a transaction result from bytes.
    #[bindings(napi(factory))]
    pub fn deserialize(bytes: Buffer) -> JsResult<TransactionResult> {
        let native = NativeTransactionResult::read_from_bytes(&bytes)
            .map_err(|e| {
                platform::error_with_context(e, "Failed to deserialize TransactionResult")
            })?;
        Ok(TransactionResult::from(native))
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
