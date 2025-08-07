use miden_client::store::TransactionFilter as NativeTransactionFilter;
use miden_client::transaction::TransactionId as NativeTransactionId;
use wasm_bindgen::prelude::*;

use super::transaction_id::TransactionId;

#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionFilter(NativeTransactionFilter);

#[wasm_bindgen]
impl TransactionFilter {
    pub fn all() -> TransactionFilter {
        TransactionFilter(NativeTransactionFilter::All)
    }

    pub fn ids(ids: Vec<TransactionId>) -> TransactionFilter {
        let native_transaction_ids: Vec<NativeTransactionId> =
            ids.into_iter().map(Into::into).collect();
        TransactionFilter(NativeTransactionFilter::Ids(native_transaction_ids))
    }

    pub fn uncommitted() -> TransactionFilter {
        TransactionFilter(NativeTransactionFilter::Uncommitted)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<TransactionFilter> for NativeTransactionFilter {
    fn from(filter: TransactionFilter) -> Self {
        filter.0
    }
}

impl From<&TransactionFilter> for NativeTransactionFilter {
    fn from(filter: &TransactionFilter) -> Self {
        filter.0.clone()
    }
}
