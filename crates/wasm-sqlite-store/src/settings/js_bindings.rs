use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;

// Settings SQLite Operations
#[wasm_bindgen(module = "/src/js/settings.js")]
extern "C" {
    #[wasm_bindgen(js_name = getSetting)]
    pub fn js_get_setting(db_id: &str, key: String) -> JsValue;

    #[wasm_bindgen(js_name = insertSetting)]
    pub fn js_insert_setting(db_id: &str, key: String, value: Vec<u8>);

    #[wasm_bindgen(js_name = removeSetting)]
    pub fn js_remove_setting(db_id: &str, key: String);

    #[wasm_bindgen(js_name = listSettingKeys)]
    pub fn js_list_setting_keys(db_id: &str) -> JsValue;
}
