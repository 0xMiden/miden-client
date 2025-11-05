use miden_client::note::Note as NativeNote;
use miden_client::transaction::TransactionRequest as NativeTransactionRequest;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::note::Note;
use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

pub mod note_and_args;
pub mod note_details_and_tag;
pub mod note_id_and_args;
pub mod transaction_request_builder;

use note_details_and_tag::NoteDetailsAndTag;

/// Encapsulates all inputs required to execute a transaction.
#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionRequest(NativeTransactionRequest);

#[wasm_bindgen]
impl TransactionRequest {
    /// Serializes the transaction request into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a transaction request from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<TransactionRequest, JsValue> {
        deserialize_from_uint8array::<NativeTransactionRequest>(bytes).map(TransactionRequest)
    }

    #[wasm_bindgen(js_name = "expectedOutputOwnNotes")]
    /// Returns notes the transaction expects to create and own.
    pub fn expected_output_own_notes(&self) -> Result<Vec<Note>, JsValue> {
        let native_notes: Vec<NativeNote> = self.0.expected_output_own_notes();
        let notes: Vec<Note> = native_notes.into_iter().map(Into::into).collect();
        Ok(notes)
    }

    #[wasm_bindgen(js_name = "expectedFutureNotes")]
    /// Returns future notes expected to be created by the transaction.
    pub fn expected_future_notes(&self) -> Result<Vec<NoteDetailsAndTag>, JsValue> {
        self.0
            .expected_future_notes()
            .cloned()  // This converts &(T, U) to (T, U)
            .map(|(note_details, note_tag)| {
                Ok(NoteDetailsAndTag::new(
                    note_details.into(),
                    note_tag.into()
                ))
            })
            .collect::<Result<Vec<NoteDetailsAndTag>, _>>()
    }

    #[wasm_bindgen(js_name = "scriptArg")]
    /// Returns the optional script argument provided to the transaction.
    pub fn script_arg(&self) -> Option<Word> {
        self.0.script_arg().map(Word::from)
    }

    #[wasm_bindgen(js_name = "authArg")]
    /// Returns the optional authentication argument provided to the transaction.
    pub fn auth_arg(&self) -> Option<Word> {
        self.0.auth_arg().map(Word::from)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<TransactionRequest> for NativeTransactionRequest {
    fn from(transaction_request: TransactionRequest) -> Self {
        transaction_request.0
    }
}

impl From<&TransactionRequest> for NativeTransactionRequest {
    fn from(transaction_request: &TransactionRequest) -> Self {
        transaction_request.0.clone()
    }
}

impl From<NativeTransactionRequest> for TransactionRequest {
    fn from(transaction_request: NativeTransactionRequest) -> Self {
        TransactionRequest(transaction_request)
    }
}

impl From<&NativeTransactionRequest> for TransactionRequest {
    fn from(transaction_request: &NativeTransactionRequest) -> Self {
        TransactionRequest(transaction_request.clone())
    }
}
