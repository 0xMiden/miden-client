use miden_client::store::TransactionFilter as NativeTransactionFilter;
use miden_client::transaction::TransactionId as NativeTransactionId;
use wasm_bindgen::prelude::*;

use super::transaction_id::TransactionId;

/// Filter used when querying stored transactions.
#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionFilter(NativeTransactionFilter);

#[wasm_bindgen]
impl TransactionFilter {
    /// Matches all transactions.
    pub fn all() -> TransactionFilter {
        TransactionFilter(NativeTransactionFilter::All)
    }

    /// Matches specific transaction IDs.
    pub fn ids(ids: Vec<TransactionId>) -> TransactionFilter {
        let native_transaction_ids: Vec<NativeTransactionId> =
            ids.into_iter().map(Into::into).collect();
        TransactionFilter(NativeTransactionFilter::Ids(native_transaction_ids))
    }

    /// Matches transactions that are not yet committed.
    pub fn uncommitted() -> TransactionFilter {
        TransactionFilter(NativeTransactionFilter::Uncommitted)
    }

    /// Matches transactions that expired before the given block number.
    #[wasm_bindgen(js_name = "expiredBefore")]
    pub fn expired_before(block_num: u32) -> TransactionFilter {
        TransactionFilter(NativeTransactionFilter::ExpiredBefore(block_num.into()))
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
