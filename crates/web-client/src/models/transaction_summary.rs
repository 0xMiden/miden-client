use miden_client::transaction::TransactionSummary as NativeTransactionSummary;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use super::account_delta::AccountDelta;
use super::input_notes::InputNotes;
use super::output_notes::OutputNotes;
use super::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Represents a transaction summary.
#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionSummary(NativeTransactionSummary);

#[wasm_bindgen]
impl TransactionSummary {
    /// Serializes the summary into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a summary from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<TransactionSummary, JsValue> {
        deserialize_from_uint8array::<NativeTransactionSummary>(bytes).map(TransactionSummary)
    }

    /// Returns the account delta described by the summary.
    #[wasm_bindgen(js_name = "accountDelta")]
    pub fn account_delta(&self) -> Result<AccountDelta, JsValue> {
        Ok(self.0.account_delta().into())
    }

    /// Returns the input notes referenced by the summary.
    #[wasm_bindgen(js_name = "inputNotes")]
    pub fn input_notes(&self) -> Result<InputNotes, JsValue> {
        Ok(self.0.input_notes().into())
    }

    /// Returns the output notes referenced by the summary.
    #[wasm_bindgen(js_name = "outputNotes")]
    pub fn output_notes(&self) -> Result<OutputNotes, JsValue> {
        Ok(self.0.output_notes().into())
    }

    /// Returns the random salt mixed into the summary commitment.
    pub fn salt(&self) -> Result<Word, JsValue> {
        Ok(self.0.salt().into())
    }

    /// Computes the commitment to this TransactionSummary.
    #[wasm_bindgen(js_name = "toCommitment")]
    pub fn to_commitment(&self) -> Word {
        self.0.to_commitment().into()
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
