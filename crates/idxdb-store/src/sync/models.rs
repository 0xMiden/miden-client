use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncHeightIdxdbObject {
    pub block_num: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteTagIdxdbObject {
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub tag: Vec<u8>,
    pub source_note_id: Option<String>,
    pub source_account_id: Option<String>,
}
