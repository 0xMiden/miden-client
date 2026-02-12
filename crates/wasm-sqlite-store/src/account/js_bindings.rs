use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;

// Account SQLite Operations
#[wasm_bindgen(module = "/src/js/accounts.js")]
extern "C" {
    // GETS
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = getAccountIds)]
    pub fn js_get_account_ids(db_id: &str) -> JsValue;

    #[wasm_bindgen(js_name = getAllAccountHeaders)]
    pub fn js_get_account_headers(db_id: &str) -> JsValue;

    #[wasm_bindgen(js_name = getAccountHeader)]
    pub fn js_get_account_header(db_id: &str, account_id: String) -> JsValue;

    #[wasm_bindgen(js_name = getAccountHeaderByCommitment)]
    pub fn js_get_account_header_by_commitment(db_id: &str, account_commitment: String) -> JsValue;

    #[wasm_bindgen(js_name = getAccountCode)]
    pub fn js_get_account_code(db_id: &str, commitment: String) -> JsValue;

    #[wasm_bindgen(js_name = getAccountStorage)]
    pub fn js_get_account_storage(db_id: &str, storage_commitment: String) -> JsValue;

    #[wasm_bindgen(js_name = getAccountStorageMaps)]
    pub fn js_get_account_storage_maps(db_id: &str, roots: Vec<String>) -> JsValue;

    #[wasm_bindgen(js_name = getAccountVaultAssets)]
    pub fn js_get_account_vault_assets(db_id: &str, vault_root: String) -> JsValue;

    #[wasm_bindgen(js_name = getAccountAddresses)]
    pub fn js_get_account_addresses(db_id: &str, account_id: String) -> JsValue;

    // INSERTS
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = upsertAccountCode)]
    pub fn js_upsert_account_code(db_id: &str, code_commitment: String, code: Vec<u8>);

    #[wasm_bindgen(js_name = upsertAccountStorage)]
    pub fn js_upsert_account_storage(db_id: &str, slots: JsValue);

    #[wasm_bindgen(js_name = upsertStorageMapEntries)]
    pub fn js_upsert_storage_map_entries(db_id: &str, entries: JsValue);

    #[wasm_bindgen(js_name = upsertVaultAssets)]
    pub fn js_upsert_vault_assets(db_id: &str, assets: JsValue);

    #[wasm_bindgen(js_name = upsertAccountRecord)]
    pub fn js_upsert_account_record(
        db_id: &str,
        id: String,
        code_commitment: String,
        storage_commitment: String,
        vault_root: String,
        nonce: String,
        committed: bool,
        commitment: String,
        account_seed: Option<Vec<u8>>,
    );

    #[wasm_bindgen(js_name = insertAccountAddress)]
    pub fn js_insert_account_address(db_id: &str, account_id: String, address: Vec<u8>);

    #[wasm_bindgen(js_name = removeAccountAddress)]
    pub fn js_remove_account_address(db_id: &str, address: Vec<u8>);

    #[wasm_bindgen(js_name = upsertForeignAccountCode)]
    pub fn js_upsert_foreign_account_code(
        db_id: &str,
        account_id: String,
        code: Vec<u8>,
        code_commitment: String,
    );

    #[wasm_bindgen(js_name = getForeignAccountCode)]
    pub fn js_get_foreign_account_code(db_id: &str, account_ids: Vec<String>) -> JsValue;

    // UPDATES
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = lockAccount)]
    pub fn js_lock_account(db_id: &str, account_id: String);

    // DELETES
    // --------------------------------------------------------------------------------------------

    #[wasm_bindgen(js_name = undoAccountStates)]
    pub fn js_undo_account_states(db_id: &str, account_hashes: Vec<String>);
}
