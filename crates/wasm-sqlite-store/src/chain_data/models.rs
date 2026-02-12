use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{base64_to_vec_u8_optional, base64_to_vec_u8_required};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeaderObject {
    pub block_num: u32,
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub header: Vec<u8>,
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub partial_blockchain_peaks: Vec<u8>,
    pub has_client_notes: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialBlockchainNodeObject {
    pub id: String,
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub node: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialBlockchainPeaksObject {
    #[serde(deserialize_with = "base64_to_vec_u8_optional", default)]
    pub partial_blockchain_peaks: Option<Vec<u8>>,
}
