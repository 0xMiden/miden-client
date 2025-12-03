use miden_client::account::AccountHeader as NativeAccountHeader;
use miden_client::transaction::ExecutedTransaction as NativeExecutedTransaction;
use wasm_bindgen::prelude::*;

use super::account_delta::AccountDelta;
use super::account_header::AccountHeader;
use super::account_id::AccountId;
use super::block_header::BlockHeader;
use super::input_notes::InputNotes;
use super::output_notes::OutputNotes;
use super::transaction_args::TransactionArgs;
use super::transaction_id::TransactionId;

/// A transaction together with execution results and proofs.
#[derive(Clone)]
#[wasm_bindgen]
pub struct ExecutedTransaction(NativeExecutedTransaction);

#[wasm_bindgen]
impl ExecutedTransaction {
    /// Returns the transaction ID.
    pub fn id(&self) -> TransactionId {
        self.0.id().into()
    }

    /// Returns the account the transaction was executed against.
    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.0.account_id().into()
    }

    //TODO: Expose partial account
    /// Returns the initial account header before execution.
    #[wasm_bindgen(js_name = "initialAccountHeader")]
    pub fn initial_account_header(&self) -> AccountHeader {
        NativeAccountHeader::from(self.0.initial_account()).into()
    }

    /// Returns the final account header after execution.
    #[wasm_bindgen(js_name = "finalAccountHeader")]
    pub fn final_account_header(&self) -> AccountHeader {
        self.0.final_account().into()
    }

    /// Returns the input notes consumed by the transaction.
    #[wasm_bindgen(js_name = "inputNotes")]
    pub fn input_notes(&self) -> InputNotes {
        self.0.input_notes().into()
    }

    /// Returns the output notes produced by the transaction.
    #[wasm_bindgen(js_name = "outputNotes")]
    pub fn output_notes(&self) -> OutputNotes {
        self.0.output_notes().into()
    }

    /// Returns the arguments passed to the transaction script.
    #[wasm_bindgen(js_name = "txArgs")]
    pub fn tx_args(&self) -> TransactionArgs {
        self.0.tx_args().into()
    }

    /// Returns the block header that included the transaction.
    #[wasm_bindgen(js_name = "blockHeader")]
    pub fn block_header(&self) -> BlockHeader {
        self.0.block_header().into()
    }

    /// Returns the account delta resulting from execution.
    #[wasm_bindgen(js_name = "accountDelta")]
    pub fn account_delta(&self) -> AccountDelta {
        self.0.account_delta().into()
    }

    // TODO: tx_inputs

    // TODO: advice_witness
}

// CONVERSIONS
// ================================================================================================

impl From<NativeExecutedTransaction> for ExecutedTransaction {
    fn from(native_executed_transaction: NativeExecutedTransaction) -> Self {
        ExecutedTransaction(native_executed_transaction)
    }
}

impl From<&NativeExecutedTransaction> for ExecutedTransaction {
    fn from(native_executed_transaction: &NativeExecutedTransaction) -> Self {
        ExecutedTransaction(native_executed_transaction.clone())
    }
}
