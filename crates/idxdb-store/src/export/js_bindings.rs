use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, wasm_bindgen};

#[wasm_bindgen(module = "/src/js/export.js")]
extern "C" {
    #[wasm_bindgen(js_name = exportStore)]
    pub fn idxdb_export_store() -> js_sys::Promise;
}
