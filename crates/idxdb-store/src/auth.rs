use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::{JsFuture, js_sys};

// WEB KEYSTORE HELPER
// ================================================================================================

// TODO: This functionality is not directly related to the webstore. As such, it should be moved
// into the webclient crate specifically.

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAuthIdxdbObject {
    pub secret_key: String,
}

#[wasm_bindgen(module = "/src/js/accounts.js")]
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

    #[wasm_bindgen(js_name = getAccountIdByKeyCommitment)]
    pub fn idxdb_get_account_id_by_key_commitment(
        db_id: &str,
        pub_key_commitment_hex: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = removeAllMappingsForKey)]
    pub fn idxdb_remove_all_mappings_for_key(
        db_id: &str,
        pub_key_commitment_hex: String,
    ) -> js_sys::Promise;
}

pub async fn insert_account_auth(
    db_id: &str,
    pub_key_commitment_hex: String,
    secret_key: String,
) -> Result<(), JsValue> {
    let promise = idxdb_insert_account_auth(db_id, pub_key_commitment_hex, secret_key);
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn get_account_auth_by_pub_key_commitment(
    db_id: &str,
    pub_key_commitment_hex: String,
) -> Result<Option<String>, JsValue> {
    let promise =
        idxdb_get_account_auth_by_pub_key_commitment(db_id, pub_key_commitment_hex.clone());
    let js_secret_key = JsFuture::from(promise).await?;

    let account_auth_idxdb: Option<AccountAuthIdxdbObject> =
        from_value(js_secret_key).map_err(|err| {
            JsValue::from_str(&format!("Error: failed to deserialize secret key: {err}"))
        })?;

    Ok(account_auth_idxdb.map(|auth| auth.secret_key))
}

pub async fn remove_account_auth(
    db_id: &str,
    pub_key_commitment_hex: String,
) -> Result<(), JsValue> {
    let promise = idxdb_remove_account_auth(db_id, pub_key_commitment_hex);
    JsFuture::from(promise).await?;
    Ok(())
}

pub async fn insert_account_key_mapping(
    db_id: &str,
    account_id_hex: String,
    pub_key_commitment_hex: String,
) -> Result<(), JsValue> {
    let promise = idxdb_insert_account_key_mapping(db_id, account_id_hex, pub_key_commitment_hex);
    JsFuture::from(promise).await?;
    Ok(())
}

pub async fn remove_account_key_mapping(
    db_id: &str,
    account_id_hex: String,
    pub_key_commitment_hex: String,
) -> Result<bool, JsValue> {
    let promise = idxdb_remove_account_key_mapping(db_id, account_id_hex, pub_key_commitment_hex);
    let result = JsFuture::from(promise).await?;
    Ok(result.as_bool().unwrap_or(false))
}

pub async fn get_key_commitments_by_account_id(
    db_id: &str,
    account_id_hex: String,
) -> Result<Vec<String>, JsValue> {
    let promise = idxdb_get_key_commitments_by_account_id(db_id, account_id_hex);
    let js_commitments = JsFuture::from(promise).await?;

    let commitments: Vec<String> = from_value(js_commitments).map_err(|err| {
        JsValue::from_str(&format!("Error: failed to deserialize key commitments: {err}"))
    })?;

    Ok(commitments)
}

pub async fn get_account_id_by_key_commitment(
    db_id: &str,
    pub_key_commitment_hex: String,
) -> Result<Option<String>, JsValue> {
    let promise = idxdb_get_account_id_by_key_commitment(db_id, pub_key_commitment_hex);
    let js_account_id = JsFuture::from(promise).await?;
    Ok(js_account_id.as_string())
}

pub async fn remove_all_mappings_for_key(
    db_id: &str,
    pub_key_commitment_hex: String,
) -> Result<(), JsValue> {
    let promise = idxdb_remove_all_mappings_for_key(db_id, pub_key_commitment_hex);
    JsFuture::from(promise).await?;
    Ok(())
}
