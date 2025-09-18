use miden_client::transaction::TransactionStoreUpdate as NativeTransactionStoreUpdate;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::executed_transaction::ExecutedTransaction;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

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

    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<TransactionStoreUpdate, JsValue> {
        deserialize_from_uint8array::<NativeTransactionStoreUpdate>(bytes)
            .map(TransactionStoreUpdate)
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
