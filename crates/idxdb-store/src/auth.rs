use miden_client::account::AccountId;
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
        pub_key_hex_commitment: String,
        secret_key_hex: String,
        account_id: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountAuthByPubKeyCommitment)]
    pub fn idxdb_get_account_auth_by_pub_key(pub_key_commitment_hex: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getPublicCommitmentsForAccountId)]
    pub fn idxdb_get_public_key_commitments_for_account(account_id_hex: String) -> js_sys::Promise;
}

pub async fn insert_account_auth(
    pub_key_commitment_hex: String,
    secret_key_hex: String,
    account_id: &AccountId,
) -> Result<(), JsValue> {
    let account_id_hex = account_id.to_hex();
    let promise = idxdb_insert_account_auth(pub_key_commitment_hex, secret_key_hex, account_id_hex);
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn get_account_auth_by_pub_key(pub_key: String) -> Result<String, JsValue> {
    let promise = idxdb_get_account_auth_by_pub_key(pub_key.clone());
    let js_secret_key = JsFuture::from(promise).await?;

    let account_auth_idxdb: Option<AccountAuthIdxdbObject> =
        from_value(js_secret_key).map_err(|err| {
            JsValue::from_str(&format!("Error: failed to deserialize secret key: {err}"))
        })?;

    match account_auth_idxdb {
        Some(account_auth) => Ok(account_auth.secret_key),
        None => Err(JsValue::from_str(&format!("Pub key {pub_key} not found in the store"))),
    }
}

pub async fn get_public_commitments_for_account(
    account_id: AccountId,
) -> Result<Vec<String>, JsValue> {
    let account_id_hex = account_id.to_hex();

    let promise = idxdb_get_public_key_commitments_for_account(account_id_hex);

    let raw_public_keys: Vec<String> = from_value(JsFuture::from(promise).await?).map_err(|err| {
        JsValue::from_str(&format!(
            "Error: failed to deserialize secret keys for account: {account_id}, got error: {err} "
        ))
    })?;

    Ok(raw_public_keys)
}
