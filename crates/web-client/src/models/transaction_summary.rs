use miden_client::transaction::TransactionSummary as NativeTransactionSummary;
#[cfg(feature = "napi")]
use miden_client::{Deserializable, Serializable};
#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::prelude::*;

#[cfg(feature = "wasm")]
use super::account_delta::AccountDelta;
#[cfg(feature = "wasm")]
use super::input_notes::InputNotes;
#[cfg(feature = "wasm")]
use super::output_notes::OutputNotes;
use super::word::Word;
#[cfg(feature = "wasm")]
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Represents a transaction summary.
#[derive(Clone)]
#[bindings]
pub struct TransactionSummary(NativeTransactionSummary);

// Shared impl block for methods with identical signatures and bodies.
#[bindings]
impl TransactionSummary {
    /// Computes the commitment to this `TransactionSummary`.
    pub fn to_commitment(&self) -> Word {
        self.0.to_commitment().into()
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl TransactionSummary {
    /// Serializes the summary into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a summary from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> JsResult<TransactionSummary> {
        deserialize_from_uint8array::<NativeTransactionSummary>(bytes).map(TransactionSummary)
    }

    /// Returns the account delta described by the summary.
    
    pub fn account_delta(&self) -> JsResult<AccountDelta> {
        Ok(self.0.account_delta().into())
    }

    /// Returns the input notes referenced by the summary.
    
    pub fn input_notes(&self) -> JsResult<InputNotes> {
        Ok(self.0.input_notes().into())
    }

    /// Returns the output notes referenced by the summary.
    
    pub fn output_notes(&self) -> JsResult<OutputNotes> {
        Ok(self.0.output_notes().into())
    }

    /// Returns the random salt mixed into the summary commitment.
    pub fn salt(&self) -> JsResult<Word> {
        Ok(self.0.salt().into())
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl TransactionSummary {
    /// Serializes the summary into bytes.
    pub fn serialize(&self) -> Buffer {
        let bytes = self.0.to_bytes();
        Buffer::from(bytes)
    }

    /// Deserializes a summary from bytes.
    #[napi(factory)]
    pub fn deserialize(bytes: Buffer) -> JsResult<TransactionSummary> {
        let native = NativeTransactionSummary::read_from_bytes(&bytes).map_err(|e| {
            platform::error_with_context(e, "Failed to deserialize TransactionSummary")
        })?;
        Ok(TransactionSummary(native))
    }

    /// Returns the random salt mixed into the summary commitment.
    pub fn salt(&self) -> Word {
        self.0.salt().into()
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
