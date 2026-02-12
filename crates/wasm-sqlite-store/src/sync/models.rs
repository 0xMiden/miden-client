use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::base64_to_vec_u8_required;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncHeightObject {
    pub block_num: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteTagObject {
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub tag: Vec<u8>,
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub source: Vec<u8>,
}
