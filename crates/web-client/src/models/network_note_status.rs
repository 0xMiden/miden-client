use miden_client::rpc::domain::status::{
    NetworkNoteStatus as NativeNetworkNoteStatus,
    NetworkNoteStatusInfo as NativeNetworkNoteStatusInfo,
};
use wasm_bindgen::prelude::*;

/// Status of a network note in the node.
#[wasm_bindgen(js_name = "NetworkNoteStatusInfo")]
pub struct NetworkNoteStatusInfo {
    status: NativeNetworkNoteStatus,
    last_error: Option<String>,
    attempt_count: u32,
    last_attempt_block_num: Option<u32>,
}

#[wasm_bindgen(js_class = "NetworkNoteStatusInfo")]
impl NetworkNoteStatusInfo {
    /// Returns the status as a string: "Pending", "Processed", "Discarded", or "Committed".
    #[wasm_bindgen(getter)]
    pub fn status(&self) -> String {
        self.status.to_string()
    }

    /// Returns the last error message, if any.
    #[wasm_bindgen(js_name = "lastError", getter)]
    pub fn last_error(&self) -> Option<String> {
        self.last_error.clone()
    }

    /// Returns the number of processing attempts.
    #[wasm_bindgen(js_name = "attemptCount", getter)]
    pub fn attempt_count(&self) -> u32 {
        self.attempt_count
    }

    /// Returns the block number of the last processing attempt, if any.
    #[wasm_bindgen(js_name = "lastAttemptBlockNum", getter)]
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
