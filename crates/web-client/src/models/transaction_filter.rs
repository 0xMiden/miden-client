use miden_client::store::TransactionFilter as NativeTransactionFilter;
use miden_client::transaction::TransactionId as NativeTransactionId;
use crate::prelude::*;

use super::transaction_id::TransactionId;

/// Filter used when querying stored transactions.
#[derive(Clone)]
#[bindings]
pub struct TransactionFilter(NativeTransactionFilter);

#[bindings]
impl TransactionFilter {
    /// Matches all transactions.
    #[bindings(factory)]
    pub fn all() -> TransactionFilter {
        TransactionFilter(NativeTransactionFilter::All)
    }

    /// Matches specific transaction IDs.
    #[bindings(factory)]
    pub fn ids(ids: Vec<TransactionId>) -> TransactionFilter {
        let native_transaction_ids: Vec<NativeTransactionId> =
            ids.into_iter().map(Into::into).collect();
        TransactionFilter(NativeTransactionFilter::Ids(native_transaction_ids))
    }

    /// Matches transactions that are not yet committed.
    #[bindings(factory)]
    pub fn uncommitted() -> TransactionFilter {
        TransactionFilter(NativeTransactionFilter::Uncommitted)
    }

    /// Matches transactions that expired before the given block number.
    #[bindings(factory)]
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
