use miden_client::transaction::TransactionSummary as NativeTransactionSummary;
use js_export_macro::js_export;

use super::account_delta::AccountDelta;
use super::input_notes::InputNotes;
use super::output_notes::OutputNotes;
use super::word::Word;
use crate::platform::{JsBytes, JsErr};
use crate::utils::{deserialize_from_bytes, serialize_to_bytes};

/// Represents a transaction summary.
#[derive(Clone)]
#[js_export]
pub struct TransactionSummary(NativeTransactionSummary);

#[js_export]
impl TransactionSummary {
    /// Serializes the summary into bytes.
    pub fn serialize(&self) -> JsBytes {
        serialize_to_bytes(&self.0)
    }

    /// Deserializes a summary from bytes.
    pub fn deserialize(bytes: JsBytes) -> Result<TransactionSummary, JsErr> {
        deserialize_from_bytes::<NativeTransactionSummary>(&bytes).map(TransactionSummary)
    }

    /// Returns the account delta described by the summary.
    #[js_export(js_name = "accountDelta")]
    pub fn account_delta(&self) -> Result<AccountDelta, JsErr> {
        Ok(self.0.account_delta().into())
    }

    /// Returns the input notes referenced by the summary.
    #[js_export(js_name = "inputNotes")]
    pub fn input_notes(&self) -> Result<InputNotes, JsErr> {
        Ok(self.0.input_notes().into())
    }

    /// Returns the output notes referenced by the summary.
    #[js_export(js_name = "outputNotes")]
    pub fn output_notes(&self) -> Result<OutputNotes, JsErr> {
        Ok(self.0.output_notes().into())
    }

    /// Returns the random salt mixed into the summary commitment.
    pub fn salt(&self) -> Result<Word, JsErr> {
        Ok(self.0.salt().into())
    }

    /// Computes the commitment to this `TransactionSummary`.
    #[js_export(js_name = "toCommitment")]
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

impl_napi_from_value!(TransactionSummary);
