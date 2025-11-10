#[wasm_bindgen(
    module = "/src/js/utils.js",
    inline_js = __INLINE_JS__
)]
extern "C" {
    #[wasm_bindgen(js_name = logWebStoreError)]
    pub fn log_web_store_error(error: JsValue, error_context: alloc::string::String);
}
