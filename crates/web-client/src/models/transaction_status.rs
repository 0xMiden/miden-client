use js_export_macro::js_export;
use miden_client::transaction::{DiscardCause, TransactionStatus as NativeTransactionStatus};

use crate::platform::{JsErr, JsU64, from_str_err, js_u64_to_u64};

/// Status of a transaction in the node or store.
#[derive(Clone)]
#[js_export]
pub struct TransactionStatus(NativeTransactionStatus);

#[js_export]
impl TransactionStatus {
    /// Creates a pending transaction status.
    pub fn pending() -> TransactionStatus {
        TransactionStatus(NativeTransactionStatus::Pending)
    }

    /// Creates a committed status with block number and timestamp.
    pub fn committed(block_num: u32, commit_timestamp: JsU64) -> TransactionStatus {
        TransactionStatus(NativeTransactionStatus::Committed {
            block_number: block_num.into(),
            commit_timestamp: js_u64_to_u64(commit_timestamp),
        })
    }

    /// Creates a discarded status from a discard cause string.
    pub fn discarded(cause: String) -> Result<TransactionStatus, JsErr> {
        let native_cause = DiscardCause::from_string(&cause)
            .map_err(|err| from_str_err(&format!("Invalid discard cause: {err}")))?;

        Ok(TransactionStatus(NativeTransactionStatus::Discarded(native_cause)))
    }

    /// Returns true if the transaction is still pending.
    #[js_export(js_name = "isPending")]
    pub fn is_pending(&self) -> bool {
        matches!(self.0, NativeTransactionStatus::Pending)
    }

    /// Returns true if the transaction has been committed.
    #[js_export(js_name = "isCommitted")]
    pub fn is_committed(&self) -> bool {
        matches!(self.0, NativeTransactionStatus::Committed { .. })
    }

    /// Returns true if the transaction was discarded.
    #[js_export(js_name = "isDiscarded")]
    pub fn is_discarded(&self) -> bool {
        matches!(self.0, NativeTransactionStatus::Discarded(_))
    }

    /// Returns the block number if the transaction was committed.
    #[js_export(js_name = "getBlockNum")]
    pub fn get_block_num(&self) -> Option<u32> {
        match self.0 {
            NativeTransactionStatus::Committed { block_number, .. } => Some(block_number.as_u32()),
            _ => None,
        }
    }

    /// Returns the commit timestamp if the transaction was committed.
    #[js_export(js_name = "getCommitTimestamp")]
    pub fn get_commit_timestamp(&self) -> Option<JsU64> {
        match self.0 {
            NativeTransactionStatus::Committed { commit_timestamp, .. } => {
                Some(commit_timestamp as JsU64)
            },
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
