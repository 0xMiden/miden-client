use miden_client::transaction::{DiscardCause, TransactionStatus as NativeTransactionStatus};
use wasm_bindgen::prelude::*;

/// Status of a transaction in the node or store.
#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionStatus(NativeTransactionStatus);

#[wasm_bindgen]
impl TransactionStatus {
    /// Creates a pending transaction status.
    pub fn pending() -> TransactionStatus {
        TransactionStatus(NativeTransactionStatus::Pending)
    }

    /// Creates a committed status with block number and timestamp.
    pub fn committed(block_num: u32, commit_timestamp: u64) -> TransactionStatus {
        TransactionStatus(NativeTransactionStatus::Committed {
            block_number: block_num.into(),
            commit_timestamp,
        })
    }

    /// Creates a discarded status from a discard cause string.
    pub fn discarded(cause: &str) -> TransactionStatus {
        let native_cause = DiscardCause::from_string(cause).expect("Invalid discard cause");

        TransactionStatus(NativeTransactionStatus::Discarded(native_cause))
    }

    /// Returns true if the transaction is still pending.
    #[wasm_bindgen(js_name = "isPending")]
    pub fn is_pending(&self) -> bool {
        matches!(self.0, NativeTransactionStatus::Pending)
    }

    /// Returns true if the transaction has been committed.
    #[wasm_bindgen(js_name = "isCommitted")]
    pub fn is_committed(&self) -> bool {
        matches!(self.0, NativeTransactionStatus::Committed { .. })
    }

    /// Returns true if the transaction was discarded.
    #[wasm_bindgen(js_name = "isDiscarded")]
    pub fn is_discarded(&self) -> bool {
        matches!(self.0, NativeTransactionStatus::Discarded(_))
    }

    /// Returns the block number if the transaction was committed.
    #[wasm_bindgen(js_name = "getBlockNum")]
    pub fn get_block_num(&self) -> Option<u32> {
        match self.0 {
            NativeTransactionStatus::Committed { block_number, .. } => Some(block_number.as_u32()),
            _ => None,
        }
    }

    /// Returns the commit timestamp if the transaction was committed.
    #[wasm_bindgen(js_name = "getCommitTimestamp")]
    pub fn get_commit_timestamp(&self) -> Option<u64> {
        match self.0 {
            NativeTransactionStatus::Committed { commit_timestamp, .. } => Some(commit_timestamp),
            _ => None,
        }
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeTransactionStatus> for TransactionStatus {
    fn from(native_status: NativeTransactionStatus) -> Self {
        TransactionStatus(native_status)
    }
}

impl From<&NativeTransactionStatus> for TransactionStatus {
    fn from(native_status: &NativeTransactionStatus) -> Self {
        TransactionStatus(native_status.clone())
    }
}
