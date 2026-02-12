use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::base64_to_vec_u8_required;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingValueObject {
    pub key: String,
    #[serde(deserialize_with = "base64_to_vec_u8_required", default)]
    pub value: Vec<u8>,
}
