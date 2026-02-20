use miden_client::transaction::TransactionRecord as NativeTransactionRecord;

use crate::prelude::*;
use crate::WebClient;
use crate::models::transaction_filter::TransactionFilter;
use crate::models::transaction_record::TransactionRecord;

#[bindings]
impl WebClient {
    #[bindings(js_name = "getTransactions")]
    pub async fn get_transactions(
        &self,
        transaction_filter: &TransactionFilter,
    ) -> platform::JsResult<Vec<TransactionRecord>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let transaction_records: Vec<NativeTransactionRecord> = client
            .get_transactions(transaction_filter.into())
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get transactions"))?;

        Ok(transaction_records.into_iter().map(Into::into).collect())
    }
}
