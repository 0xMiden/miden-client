use miden_client::transaction::TransactionStoreUpdate as NativeTransactionStoreUpdate;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::models::account_delta::AccountDelta;
use crate::models::executed_transaction::ExecutedTransaction;
use crate::models::output_notes::OutputNotes;

#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionStoreUpdate(NativeTransactionStoreUpdate);

#[wasm_bindgen]
impl TransactionStoreUpdate {
    #[wasm_bindgen(js_name = "executedTransaction")]
    pub fn executed_transaction(&self) -> ExecutedTransaction {
        self.0.executed_transaction().into()
    }

    #[wasm_bindgen(js_name = "submissionHeight")]
    pub fn submission_height(&self) -> u32 {
        self.0.submission_height().as_u32()
    }

    #[wasm_bindgen(js_name = "createdNotes")]
    pub fn created_notes(&self) -> OutputNotes {
        self.0.executed_transaction().output_notes().into()
    }

    #[wasm_bindgen(js_name = "accountDelta")]
    pub fn account_delta(&self) -> AccountDelta {
        self.0.executed_transaction().account_delta().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<TransactionStoreUpdate> for NativeTransactionStoreUpdate {
    fn from(update: TransactionStoreUpdate) -> Self {
        update.0
    }
}

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
