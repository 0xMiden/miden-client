use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, wasm_bindgen};

// Settings IndexedDB Operations
#[wasm_bindgen(module = "/src/store/web_store/js/settings.js")]
extern "C" {
    #[wasm_bindgen(js_name = getSettingValue)]
    pub fn idxdb_get_setting_value(key: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = insertSettingValue)]
    pub fn idxdb_insert_setting_value(key: String, value: Vec<u8>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = deleteSettingValue)]
    pub fn idxdb_delete_setting_value(key: String) -> js_sys::Promise;
}
