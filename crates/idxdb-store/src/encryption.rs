use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::{JsFuture, js_sys};

// WEB ENCRYPTION KEYSTORE HELPER
// ================================================================================================

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionKeyIdxdbObject {
    pub key: String,
}

#[wasm_bindgen(module = "/src/js/accounts.js")]
extern "C" {
    #[wasm_bindgen(js_name = insertEncryptionKey)]
    pub fn idxdb_insert_encryption_key(address_hash: String, key_hex: String) -> js_sys::Promise;

    #[wasm_bindgen(js_name = getEncryptionKeyByAddressHash)]
    pub fn idxdb_get_encryption_key(address_hash: String) -> js_sys::Promise;
}

pub async fn insert_encryption_key(address_hash: String, key_hex: String) -> Result<(), JsValue> {
    let promise = idxdb_insert_encryption_key(address_hash, key_hex);
    JsFuture::from(promise).await?;

    Ok(())
}

pub async fn get_encryption_key(address_hash: String) -> Result<Option<String>, JsValue> {
    let promise = idxdb_get_encryption_key(address_hash);
    let js_key = JsFuture::from(promise).await?;

    let encryption_key_idxdb: Option<EncryptionKeyIdxdbObject> =
        from_value(js_key).map_err(|err| {
            JsValue::from_str(&format!("Error: failed to deserialize encryption key: {err}"))
        })?;

    Ok(encryption_key_idxdb.map(|k| k.key))
}
