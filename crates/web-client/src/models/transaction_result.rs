use miden_client::transaction::TransactionResult as NativeTransactionResult;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::account_delta::AccountDelta;
use crate::models::executed_transaction::ExecutedTransaction;
use crate::models::input_notes::InputNotes;
use crate::models::output_notes::OutputNotes;
use crate::models::transaction_args::TransactionArgs;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Result of executing a transaction, including notes and account deltas.
#[wasm_bindgen]
pub struct TransactionResult(NativeTransactionResult);

#[wasm_bindgen]
impl TransactionResult {
    #[wasm_bindgen(js_name = "executedTransaction")]
    /// Returns the executed transaction details.
    pub fn executed_transaction(&self) -> ExecutedTransaction {
        self.0.executed_transaction().into()
    }

    #[wasm_bindgen(js_name = "createdNotes")]
    /// Returns notes created by the transaction.
    pub fn created_notes(&self) -> OutputNotes {
        self.0.created_notes().into()
    }

    // TODO: relevant_notes

    #[wasm_bindgen(js_name = "blockNum")]
    /// Returns the block number the transaction was executed in.
    pub fn block_num(&self) -> u32 {
        self.0.block_num().as_u32()
    }

    #[wasm_bindgen(js_name = "transactionArguments")]
    /// Returns the arguments consumed by the transaction script.
    pub fn transaction_arguments(&self) -> TransactionArgs {
        self.0.transaction_arguments().into()
    }

    #[wasm_bindgen(js_name = "accountDelta")]
    /// Returns the resulting account delta.
    pub fn account_delta(&self) -> AccountDelta {
        self.0.account_delta().into()
    }

    #[wasm_bindgen(js_name = "consumedNotes")]
    /// Returns the notes consumed by the transaction.
    pub fn consumed_notes(&self) -> InputNotes {
        self.0.consumed_notes().into()
    }

    /// Serializes the transaction result into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a transaction result from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<TransactionResult, JsValue> {
        deserialize_from_uint8array::<NativeTransactionResult>(bytes).map(TransactionResult)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeTransactionResult> for TransactionResult {
    fn from(native_transaction_result: NativeTransactionResult) -> Self {
        TransactionResult(native_transaction_result)
    }
}

impl From<&NativeTransactionResult> for TransactionResult {
    fn from(native_transaction_result: &NativeTransactionResult) -> Self {
        TransactionResult(native_transaction_result.clone())
    }
}

impl From<TransactionResult> for NativeTransactionResult {
    fn from(transaction_result: TransactionResult) -> Self {
        transaction_result.0
    }
}

impl From<&TransactionResult> for NativeTransactionResult {
    fn from(transaction_result: &TransactionResult) -> Self {
        transaction_result.0.clone()
    }
}
