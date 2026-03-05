use miden_client::transaction::TransactionRecord as NativeTransactionRecord;

use js_export_macro::js_export;

use super::models::transaction_filter::TransactionFilter;
use super::models::transaction_record::TransactionRecord;
use crate::platform::{JsErr, from_str_err};
use crate::{WebClient, js_error_with_context};

#[js_export]
impl WebClient {
    #[js_export(js_name = "getTransactions")]
    pub async fn get_transactions(
        &self,
        transaction_filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let transaction_records: Vec<NativeTransactionRecord> = client
            .get_transactions(transaction_filter.into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get transactions"))?;

        Ok(transaction_records.into_iter().map(Into::into).collect())
    }
}
