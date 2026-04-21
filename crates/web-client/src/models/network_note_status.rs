use alloc::string::{String, ToString};

use js_export_macro::js_export;
use miden_client::rpc::domain::status::{
    NetworkNoteStatus as NativeNetworkNoteStatus,
    NetworkNoteStatusInfo as NativeNetworkNoteStatusInfo,
};

/// Status of a network note in the node.
#[js_export(js_name = "NetworkNoteStatusInfo")]
pub struct NetworkNoteStatusInfo {
    status: NativeNetworkNoteStatus,
    last_error: Option<String>,
    attempt_count: u32,
    last_attempt_block_num: Option<u32>,
}

#[js_export]
impl NetworkNoteStatusInfo {
    /// Returns the status as a string: `"Pending"`, `"NullifierInflight"`, `"Discarded"`, or
    /// `"NullifierCommitted"`.
    #[js_export(getter)]
    pub fn status(&self) -> String {
        self.status.to_string()
    }

    /// Returns the last error message, if any.
    #[js_export(getter, js_name = "lastError")]
    pub fn last_error(&self) -> Option<String> {
        self.last_error.clone()
    }

    /// Returns the number of processing attempts.
    #[js_export(getter, js_name = "attemptCount")]
    pub fn attempt_count(&self) -> u32 {
        self.attempt_count
    }

    /// Returns the block number of the last processing attempt, if any.
    #[js_export(getter, js_name = "lastAttemptBlockNum")]
    pub fn last_attempt_block_num(&self) -> Option<u32> {
        self.last_attempt_block_num
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNetworkNoteStatusInfo> for NetworkNoteStatusInfo {
    fn from(native: NativeNetworkNoteStatusInfo) -> Self {
        Self {
            status: native.status,
            last_error: native.last_error,
            attempt_count: native.attempt_count,
            last_attempt_block_num: native.last_attempt_block_num,
        }
    }
}
