#[wasm_bindgen(
    module = "/src/js/import.js",
    inline_js = __INLINE_JS__
)]
extern "C" {
    #[wasm_bindgen(js_name = forceImportStore)]
    pub fn idxdb_force_import_store(store_dump: JsValue) -> js_sys::Promise;
}
