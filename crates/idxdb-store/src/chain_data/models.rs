use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{base64_to_vec_u8_optional, base64_to_vec_u8_required};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeaderIdxdbObject {
    pub block_num: String,
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub header: Vec<u8>,
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub partial_blockchain_peaks: Vec<u8>,
    pub has_client_notes: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialBlockchainNodeIdxdbObject {
    pub id: u64,
    pub node: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialBlockchainPeaksIdxdbObject {
    #[serde(deserialize_with = "base64_to_vec_u8_optional", default)]
    pub peaks: Option<Vec<u8>>,
}
