use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use idxdb_store::auth::{
    idxdb_get_account_auth_by_pub_key_commitment,
    idxdb_get_account_id_by_key_commitment,
    idxdb_get_key_commitments_by_account_id,
    idxdb_insert_account_auth,
    idxdb_insert_account_key_mapping,
    idxdb_remove_account_auth,
    idxdb_remove_all_mappings_for_key,
};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AccountAuthIdxdbObject {
    pub secret_key: String,
}

pub(crate) async fn insert_account_auth(
    db_id: &str,
    pub_key_commitment_hex: String,
    secret_key: String,
) -> Result<(), JsValue> {
    let promise = idxdb_insert_account_auth(db_id, pub_key_commitment_hex, secret_key);
    JsFuture::from(promise).await?;

    Ok(())
}

pub(crate) async fn get_account_auth_by_pub_key_commitment(
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

pub(crate) async fn remove_account_auth(
    db_id: &str,
    pub_key_commitment_hex: String,
) -> Result<(), JsValue> {
    let promise = idxdb_remove_account_auth(db_id, pub_key_commitment_hex);
    JsFuture::from(promise).await?;
    Ok(())
}

pub(crate) async fn insert_account_key_mapping(
    db_id: &str,
    account_id_hex: String,
    pub_key_commitment_hex: String,
) -> Result<(), JsValue> {
    let promise = idxdb_insert_account_key_mapping(db_id, account_id_hex, pub_key_commitment_hex);
    JsFuture::from(promise).await?;
    Ok(())
}

pub(crate) async fn get_key_commitments_by_account_id(
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

pub(crate) async fn remove_all_mappings_for_key(
    db_id: &str,
    pub_key_commitment_hex: String,
) -> Result<(), JsValue> {
    let promise = idxdb_remove_all_mappings_for_key(db_id, pub_key_commitment_hex);
    JsFuture::from(promise).await?;
    Ok(())
}

pub(crate) async fn get_account_id_by_key_commitment(
    db_id: &str,
    pub_key_commitment_hex: String,
) -> Result<Option<String>, JsValue> {
    let promise = idxdb_get_account_id_by_key_commitment(db_id, pub_key_commitment_hex);
    let js_account_id = JsFuture::from(promise).await?;
    Ok(js_account_id.as_string())
}
