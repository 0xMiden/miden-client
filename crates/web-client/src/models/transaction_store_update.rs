use miden_client::transaction::TransactionStoreUpdate as NativeTransactionStoreUpdate;
#[cfg(feature = "napi")]
use miden_client::{Deserializable, Serializable};
#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::prelude::*;

#[cfg(feature = "wasm")]
use crate::models::account_delta::AccountDelta;
use crate::models::executed_transaction::ExecutedTransaction;
#[cfg(feature = "wasm")]
use crate::models::output_notes::OutputNotes;
#[cfg(feature = "wasm")]
use crate::models::transaction_request::note_details_and_tag::NoteDetailsAndTag;
#[cfg(feature = "wasm")]
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Represents the changes that need to be applied to the client store as a result of a transaction
/// execution.
#[derive(Clone)]
#[bindings]
pub struct TransactionStoreUpdate(NativeTransactionStoreUpdate);

// Shared methods (identical for wasm and napi)
#[bindings]
impl TransactionStoreUpdate {
    /// Returns the executed transaction associated with this update.
    #[bindings]
    pub fn executed_transaction(&self) -> ExecutedTransaction {
        self.0.executed_transaction().into()
    }

    /// Returns the block height at which the transaction was submitted.
    #[bindings]
    pub fn submission_height(&self) -> u32 {
        self.0.submission_height().as_u32()
    }
}

// Wasm-only methods
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl TransactionStoreUpdate {
    /// Returns the notes created by the transaction.
    
    pub fn created_notes(&self) -> OutputNotes {
        self.0.executed_transaction().output_notes().into()
    }

    /// Returns the account delta applied by the transaction.
    
    pub fn account_delta(&self) -> AccountDelta {
        self.0.executed_transaction().account_delta().into()
    }

    /// Returns notes expected to be created in follow-up executions.
    
    pub fn future_notes(&self) -> Vec<NoteDetailsAndTag> {
        self.0
            .future_notes()
            .iter()
            .cloned()
            .map(|(details, tag)| {
                let details: crate::models::note_details::NoteDetails = details.into();
                let tag: crate::models::note_tag::NoteTag = tag.into();
                NoteDetailsAndTag::new(&details, &tag)
            })
            .collect()
    }

    /// Serializes the update into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes an update from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> JsResult<TransactionStoreUpdate> {
        deserialize_from_uint8array::<NativeTransactionStoreUpdate>(bytes)
            .map(TransactionStoreUpdate)
    }
}

// Napi-only methods
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl TransactionStoreUpdate {
    /// Serializes the update into bytes.
    #[napi]
    pub fn serialize(&self) -> Buffer {
        let bytes = self.0.to_bytes();
        Buffer::from(bytes)
    }

    /// Deserializes an update from bytes.
    #[napi(factory)]
    pub fn deserialize(bytes: Buffer) -> JsResult<TransactionStoreUpdate> {
        let native = NativeTransactionStoreUpdate::read_from_bytes(&bytes).map_err(|e| {
            platform::error_with_context(e, "Failed to deserialize TransactionStoreUpdate")
        })?;
        Ok(TransactionStoreUpdate(native))
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
