#[wasm_bindgen(
    module = "/src/js/accounts.js",
    inline_js = __INLINE_JS__
)]
extern "C" {
    // GETS
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = getAccountIds)]
    pub fn idxdb_get_account_ids() -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAllAccountHeaders)]
    pub fn idxdb_get_account_headers() -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountHeader)]
    pub fn idxdb_get_account_header(account_id: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountHeaderByCommitment)]
    pub fn idxdb_get_account_header_by_commitment(account_commitment: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountCode)]
    pub fn idxdb_get_account_code(code_root: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountStorage)]
    pub fn idxdb_get_account_storage(storage_root: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountStorageMaps)]
    pub fn idxdb_get_account_storage_maps(roots: Vec<String>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountVaultAssets)]
    pub fn idxdb_get_account_vault_assets(vault_root: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountAuthByPubKey)]
    pub fn idxdb_get_account_auth_by_pub_key(pub_key: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountAddresses)]
    pub fn idxdb_get_account_addresses(account_id: String) -> js_sys::Promise;

    // INSERTS
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = upsertAccountCode)]
    pub fn idxdb_upsert_account_code(code_root: String, code: Vec<u8>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertAccountStorage)]
    pub fn idxdb_upsert_account_storage(storage_slots: Vec<JsStorageSlot>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertStorageMapEntries)]
    pub fn idxdb_upsert_storage_map_entries(entries: Vec<JsStorageMapEntry>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertVaultAssets)]
    pub fn idxdb_upsert_vault_assets(assets: Vec<JsVaultAsset>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertAccountRecord)]
    pub fn idxdb_upsert_account_record(
        id: String,
        code_root: String,
        storage_root: String,
        vault_root: String,
        nonce: String,
        committed: bool,
        commitment: String,
        account_seed: Option<Vec<u8>>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = insertAccountAddress)]
    pub fn idxdb_insert_account_address(account_id: String, address: Vec<u8>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = removeAccountAddress)]
    pub fn idxdb_remove_account_address(address: Vec<u8>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = upsertForeignAccountCode)]
    pub fn idxdb_upsert_foreign_account_code(
        account_id: String,
        code: Vec<u8>,
        code_root: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getForeignAccountCode)]
    pub fn idxdb_get_foreign_account_code(account_ids: Vec<String>) -> js_sys::Promise;

    // UPDATES
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = lockAccount)]
    pub fn idxdb_lock_account(account_id: String) -> js_sys::Promise;

    // DELETES
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = undoAccountStates)]
    pub fn idxdb_undo_account_states(account_hashes: Vec<String>) -> js_sys::Promise;
}
