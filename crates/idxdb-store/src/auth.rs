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
        pub_key: String,
        secret_key: String,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getAccountAuthByPubKey)]
    pub fn idxdb_get_account_auth_by_pub_key(db_id: &str, pub_key: String) -> js_sys::Promise;
}

pub async fn insert_account_auth(
    db_id: &str,
    pub_key: String,
    secret_key: String,
) -> Result<(), JsValue> {
    let promise = idxdb_insert_account_auth(db_id, pub_key, secret_key);
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn get_account_auth_by_pub_key(db_id: &str, pub_key: String) -> Result<String, JsValue> {
    let promise = idxdb_get_account_auth_by_pub_key(db_id, pub_key.clone());
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
