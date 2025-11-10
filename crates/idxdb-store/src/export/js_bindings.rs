use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, wasm_bindgen};
use wasmbind_js_file_macro::wasmbind_dump_js_file_as_inline;

#[wasmbind_dump_js_file_as_inline(path = "${outDir}/src/js/export.js")]
extern "C" {
    #[wasm_bindgen(js_name = exportStore)]
    pub fn idxdb_export_store() -> js_sys::Promise;
}
