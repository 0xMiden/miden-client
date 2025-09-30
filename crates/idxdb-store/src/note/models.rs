use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputNoteIdxdbObject {
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub assets: Vec<u8>,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub serial_number: Vec<u8>,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub inputs: Vec<u8>,
    pub created_at: String,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub serialized_note_script: Vec<u8>,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub state: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputNoteIdxdbObject {
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub assets: Vec<u8>,
    pub recipient_digest: String,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub metadata: Vec<u8>,
    pub expected_height: u32,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub state: Vec<u8>,
}
