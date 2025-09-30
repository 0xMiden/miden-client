use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionIdxdbObject {
    pub id: String,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub details: Vec<u8>,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_optional", default)]
    pub script_root: Option<Vec<u8>>,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_optional", default)]
    pub tx_script: Option<Vec<u8>>,
    pub block_num: String,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub status: Vec<u8>,
}
