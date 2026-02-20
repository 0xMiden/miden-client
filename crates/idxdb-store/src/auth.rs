use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys;

// WEB KEYSTORE FFI BINDINGS
// ================================================================================================

#[wasm_bindgen(module = "/src/js/auth.js")]
extern "C" {
    #[wasm_bindgen(js_name = insertAccountAuth)]
    pub fn idxdb_insert_account_auth(
        db_id: &str,
        pub_key_commitment_hex: String,
        secret_key: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountAuthByPubKeyCommitment)]
    pub fn idxdb_get_account_auth_by_pub_key_commitment(
        db_id: &str,
        pub_key_commitment_hex: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = removeAccountAuth)]
    pub fn idxdb_remove_account_auth(
        db_id: &str,
        pub_key_commitment_hex: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = insertAccountKeyMapping)]
    pub fn idxdb_insert_account_key_mapping(
        db_id: &str,
        account_id_hex: String,
        pub_key_commitment_hex: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = removeAccountKeyMapping)]
    pub fn idxdb_remove_account_key_mapping(
        db_id: &str,
        account_id_hex: String,
        pub_key_commitment_hex: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getKeyCommitmentsByAccountId)]
    pub fn idxdb_get_key_commitments_by_account_id(
        db_id: &str,
        account_id_hex: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = removeAllMappingsForKey)]
    pub fn idxdb_remove_all_mappings_for_key(
        db_id: &str,
        pub_key_commitment_hex: String,
    ) -> js_sys::Promise;
}
