use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingValueIdxdbObject {
    pub key: String,
    #[serde(deserialize_with = "crate::base64_to_vec_u8_required", default)]
    pub value: Vec<u8>,
}
