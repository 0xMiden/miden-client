use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;

#[wasm_bindgen(module = "/src/js/import.js")]
extern "C" {
    #[wasm_bindgen(js_name = forceImportStore)]
    pub fn idxdb_force_import_store(
        store_dump: JsValue,
        client_version: &str,
        store_name: &str,
    ) -> js_sys::Promise;

}
