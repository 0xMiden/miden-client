use miden_client::transaction::TransactionSummary as NativeTransactionSummary;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use super::account_delta::AccountDelta;
use super::input_notes::InputNotes;
use super::output_notes::OutputNotes;
use super::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Summary of a transaction used when requesting signatures.
#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionSummary(NativeTransactionSummary);

#[wasm_bindgen]
impl TransactionSummary {
    /// Serializes the transaction summary into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a transaction summary from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<TransactionSummary, JsValue> {
        deserialize_from_uint8array::<NativeTransactionSummary>(bytes).map(TransactionSummary)
    }

    #[wasm_bindgen(js_name = "accountDelta")]
    /// Returns the account delta captured in the summary.
    pub fn account_delta(&self) -> Result<AccountDelta, JsValue> {
        Ok(self.0.account_delta().into())
    }

    #[wasm_bindgen(js_name = "inputNotes")]
    /// Returns the input notes included in the summary.
    pub fn input_notes(&self) -> Result<InputNotes, JsValue> {
        Ok(self.0.input_notes().into())
    }

    #[wasm_bindgen(js_name = "outputNotes")]
    /// Returns the expected output notes included in the summary.
    pub fn output_notes(&self) -> Result<OutputNotes, JsValue> {
        Ok(self.0.output_notes().into())
    }

    /// Returns the random salt mixed into the summary commitment.
    pub fn salt(&self) -> Result<Word, JsValue> {
        Ok(self.0.salt().into())
    }
}

// CONVERSIONS
// ================================================================================================

impl From<TransactionSummary> for NativeTransactionSummary {
    fn from(transaction_summary: TransactionSummary) -> Self {
        transaction_summary.0
    }
}

impl From<&TransactionSummary> for NativeTransactionSummary {
    fn from(transaction_summary: &TransactionSummary) -> Self {
        transaction_summary.0.clone()
    }
}

impl From<NativeTransactionSummary> for TransactionSummary {
    fn from(transaction_summary: NativeTransactionSummary) -> Self {
        TransactionSummary(transaction_summary)
    }
}

impl From<&NativeTransactionSummary> for TransactionSummary {
    fn from(transaction_summary: &NativeTransactionSummary) -> Self {
        TransactionSummary(transaction_summary.clone())
    }
}
