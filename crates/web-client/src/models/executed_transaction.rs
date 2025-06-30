use miden_objects::transaction::ExecutedTransaction as NativeExecutedTransaction;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use super::{
    account::Account, account_delta::AccountDelta, account_header::AccountHeader,
    account_id::AccountId, block_header::BlockHeader, input_notes::InputNotes,
    output_notes::OutputNotes, transaction_args::TransactionArgs, transaction_id::TransactionId,
};
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[derive(Clone)]
#[wasm_bindgen]
pub struct ExecutedTransaction(NativeExecutedTransaction);

#[wasm_bindgen]
impl ExecutedTransaction {
    pub fn id(&self) -> TransactionId {
        self.0.id().into()
    }

    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.0.account_id().into()
    }

    #[wasm_bindgen(js_name = "initialAccount")]
    pub fn initial_account(&self) -> Account {
        self.0.initial_account().into()
    }

    #[wasm_bindgen(js_name = "finalAccount")]
    pub fn final_account(&self) -> AccountHeader {
        self.0.final_account().into()
    }

    #[wasm_bindgen(js_name = "inputNotes")]
    pub fn input_notes(&self) -> InputNotes {
        self.0.input_notes().into()
    }

    #[wasm_bindgen(js_name = "outputNotes")]
    pub fn output_notes(&self) -> OutputNotes {
        self.0.output_notes().into()
    }

    #[wasm_bindgen(js_name = "txArgs")]
    pub fn tx_args(&self) -> TransactionArgs {
        self.0.tx_args().into()
    }

    #[wasm_bindgen(js_name = "blockHeader")]
    pub fn block_header(&self) -> BlockHeader {
        self.0.block_header().into()
    }

    #[wasm_bindgen(js_name = "accountDelta")]
    pub fn account_delta(&self) -> AccountDelta {
        self.0.account_delta().into()
    }

    // TODO: tx_inputs

    // TODO: advice_witness

    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<ExecutedTransaction, JsValue> {
        deserialize_from_uint8array::<NativeExecutedTransaction>(bytes).map(ExecutedTransaction)
    }
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

impl From<ExecutedTransaction> for NativeExecutedTransaction {
    fn from(executed_transaction: ExecutedTransaction) -> Self {
        executed_transaction.0
    }
}

impl From<&ExecutedTransaction> for NativeExecutedTransaction {
    fn from(executed_transaction: &ExecutedTransaction) -> Self {
        executed_transaction.0.clone()
    }
}
