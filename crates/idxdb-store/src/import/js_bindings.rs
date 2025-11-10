use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, wasm_bindgen};
use wasmbind_js_file_macro::wasmbind_dump_js_file_as_inline;

#[wasmbind_dump_js_file_as_inline(path = "${outDir}/src/js/import.js")]
extern "C" {
    #[wasm_bindgen(js_name = forceImportStore)]
    pub fn idxdb_force_import_store(store_dump: JsValue) -> js_sys::Promise;

}
