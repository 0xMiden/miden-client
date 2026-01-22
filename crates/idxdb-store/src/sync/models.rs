use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::base64_to_vec_u8_required;

/// Result of acquiring a sync lock from JavaScript.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncLockHandle {
    /// True if we acquired the lock, false if we're coalescing with an in-progress sync
    pub acquired: bool,
    /// If coalescing, the serialized result from the in-progress sync
    pub coalesced_result: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncHeightIdxdbObject {
    pub block_num: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteTagIdxdbObject {
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub tag: Vec<u8>,
    pub source_note_id: Option<String>,
    pub source_account_id: Option<String>,
}
