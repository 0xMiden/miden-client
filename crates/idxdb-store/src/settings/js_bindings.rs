use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, wasm_bindgen};

// Settings IndexedDB Operations
#[wasm_bindgen(module = "/src/js/settings.js")]
extern "C" {
    #[wasm_bindgen(js_name = getValue)]
    pub fn idxdb_get_value(key: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = insertValue)]
    pub fn idxdb_insert_value(key: String, value: Vec<u8>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = removeValue)]
    pub fn idxdb_remove_value(key: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = listKeys)]
    pub fn idxdb_list_keys() -> js_sys::Promise;
}
