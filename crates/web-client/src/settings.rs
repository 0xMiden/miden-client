use js_export_macro::js_export;
#[cfg(feature = "browser")]
use wasm_bindgen::prelude::*;

use crate::platform::{JsErr, from_str_err};
use crate::{WebClient, js_error_with_context};

// Browser-specific settings methods that need serde_wasm_bindgen for JsValue serialization
#[cfg(feature = "browser")]
#[wasm_bindgen]
impl WebClient {
    /// Retrieves the setting value for `key`, or `None` if it hasn't been set.
    #[wasm_bindgen(js_name = "getSetting")]
    pub async fn get_setting(&self, key: String) -> Result<Option<JsValue>, JsValue> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| JsValue::from_str("Client not initialized"))?;
        let result: Option<Vec<u8>> = client.get_setting(key).await.map_err(|err| {
            js_error_with_context(err, "failed to get setting value from the store")
        })?;
        let deserialized_result = result
            .map(|bytes| {
                serde_wasm_bindgen::to_value(&bytes).map_err(|err| {
                    js_error_with_context(err, "failed to deserialize setting value into a JsValue")
                })
            })
            .transpose()?;
        Ok(deserialized_result)
    }

    /// Sets a setting key-value in the store. It can then be retrieved using `get_setting`.
    #[wasm_bindgen(js_name = "setSetting")]
    pub async fn set_setting(&self, key: String, value: JsValue) -> Result<(), JsValue> {
        let value_bytes: Vec<u8> = serde_wasm_bindgen::from_value(value).map_err(|err| {
            js_error_with_context(err, "failed to serialize given value into bytes")
        })?;
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| JsValue::from_str("Client not initialized"))?;
        client.set_setting(key, value_bytes).await.map_err(|err| {
            js_error_with_context(err, "failed to set setting value in the store")
        })?;
        Ok(())
    }
}

// Node.js-specific settings methods that use Vec<u8> directly
#[cfg(feature = "nodejs")]
#[napi_derive::napi]
impl WebClient {
    /// Retrieves the setting value for `key`, or `None` if it hasn't been set.
    #[napi(js_name = "getSetting")]
    pub async fn get_setting(&self, key: String) -> Result<Option<Vec<u8>>, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        client
            .get_setting(key)
            .await
            .map_err(|err| js_error_with_context(err, "failed to get setting value from the store"))
    }

    /// Sets a setting key-value in the store.
    #[napi(js_name = "setSetting")]
    pub async fn set_setting(&self, key: String, value: Vec<u8>) -> Result<(), JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        client.set_setting(key, value).await.map_err(|err| {
            js_error_with_context(err, "failed to set setting value in the store")
        })?;
        Ok(())
    }
}

// Shared settings methods (identical logic for both platforms)
#[js_export]
impl WebClient {
    /// Deletes a setting key-value from the store.
    #[js_export(js_name = "removeSetting")]
    pub async fn remove_setting(&self, key: String) -> Result<(), JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        client.remove_setting(key).await.map_err(|err| {
            js_error_with_context(err, "failed to delete setting value in the store")
        })?;
        Ok(())
    }

    /// Returns all the existing setting keys from the store.
    #[js_export(js_name = "listSettingKeys")]
    pub async fn list_setting_keys(&self) -> Result<Vec<String>, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        client
            .list_setting_keys()
            .await
            .map_err(|err| js_error_with_context(err, "failed to list setting keys in the store"))
    }
}
