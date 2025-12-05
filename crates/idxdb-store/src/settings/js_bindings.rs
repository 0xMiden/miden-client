use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;

// Settings IndexedDB Operations
#[wasm_bindgen(module = "/src/js/settings.js")]
extern "C" {
    #[wasm_bindgen(js_name = getSetting)]
    pub fn idxdb_get_setting(key: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = insertSetting)]
    pub fn idxdb_insert_setting(key: String, value: Vec<u8>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = removeSetting)]
    pub fn idxdb_remove_setting(key: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = listSettingKeys)]
    pub fn idxdb_list_setting_keys() -> js_sys::Promise;
}
