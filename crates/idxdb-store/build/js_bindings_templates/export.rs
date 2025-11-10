#[wasm_bindgen(
    module = "/src/js/export.js",
    inline_js = __INLINE_JS__
)]
extern "C" {
    #[wasm_bindgen(js_name = exportStore)]
    pub fn idxdb_export_store() -> js_sys::Promise;
}
