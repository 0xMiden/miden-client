#[wasm_bindgen(
    module = "/src/js/schema.js",
    inline_js = __INLINE_JS__
)]
extern "C" {
    #[wasm_bindgen(js_name = openDatabase)]
    pub fn setup_indexed_db() -> js_sys::Promise;
}
