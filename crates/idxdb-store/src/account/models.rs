use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountCodeIdxdbObject {
    pub root: String,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub code: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
pub struct AccountStorageIdxdbObject {
    pub slot_index: u8,
    pub slot_value: String,
    pub slot_type: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageMapEntryIdxdbObject {
    pub root: String,
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAssetIdxdbObject {
    pub asset: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountRecordIdxdbObject {
    pub id: String,
    pub nonce: String,
    pub vault_root: String,
    pub storage_root: String,
    pub code_root: String,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_optional", default)]
    pub account_seed: Option<Vec<u8>>,
    pub locked: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForeignAccountCodeIdxdbObject {
    pub account_id: String,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub code: Vec<u8>,
}
