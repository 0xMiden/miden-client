#[wasm_bindgen(
    module = "/src/js/accounts.js",
    inline_js = __INLINE_JS__
)]
extern "C" {
    #[wasm_bindgen(js_name = insertAccountAuth)]
    pub fn idxdb_insert_account_auth(pub_key: String, secret_key: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountAuthByPubKey)]
    pub fn idxdb_get_account_auth_by_pub_key(pub_key: String) -> js_sys::Promise;
}
