use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{base64_to_vec_u8_optional, base64_to_vec_u8_required};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountCodeObject {
    pub commitment: String,
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub code: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
pub struct AccountStorageObject {
    pub slot_name: String,
    pub slot_value: String,
    pub slot_type: u8,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageMapEntryObject {
    pub root: String,
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAssetObject {
    pub asset: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountRecordObject {
    pub id: String,
    pub nonce: String,
    pub vault_root: String,
    pub storage_commitment: String,
    pub code_commitment: String,
    #[serde(deserialize_with = "base64_to_vec_u8_optional", default)]
    pub account_seed: Option<Vec<u8>>,
    pub locked: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddressObject {
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub address: Vec<u8>,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForeignAccountCodeObject {
    pub account_id: String,
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub code: Vec<u8>,
}
