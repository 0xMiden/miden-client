use miden_client::transaction::TransactionRecord as NativeTransactionRecord;
use wasm_bindgen::prelude::*;

use super::models::transaction_filter::TransactionFilter;
use super::models::transaction_record::TransactionRecord;
use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "getTransactions")]
    pub async fn get_transactions(
        &mut self,
        transaction_filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let transaction_records: Vec<NativeTransactionRecord> = client
                .get_transactions(transaction_filter.into())
                .await
                .map_err(|err| js_error_with_context(err, "failed to get transactions"))?;

            Ok(transaction_records.into_iter().map(Into::into).collect())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }
}
