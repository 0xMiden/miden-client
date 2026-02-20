#[cfg(feature = "wasm")]
use serde_wasm_bindgen::{from_value, to_value};

#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;

use crate::prelude::*;
use crate::WebClient;

// Shared methods (remove_setting, list_setting_keys)
#[bindings]
impl WebClient {
    /// Deletes a setting key-value from the store.
    #[bindings(js_name = "removeSetting")]
    pub async fn remove_setting(&self, key: String) -> platform::JsResult<()> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client.remove_setting(key).await.map_err(|err| {
            platform::error_with_context(err, "failed to delete setting value in the store")
        })?;
        Ok(())
    }

    /// Returns all the existing setting keys from the store.
    #[bindings(js_name = "listSettingKeys")]
    pub async fn list_setting_keys(&self) -> platform::JsResult<Vec<String>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client.list_setting_keys().await.map_err(|err| {
            platform::error_with_context(err, "failed to list setting keys in the store")
        })
    }
}

// wasm-specific methods (get_setting, set_setting use JsValue)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WebClient {
    /// Retrieves the setting value for `key`, or `None` if it hasn't been set.
    #[wasm_bindgen(js_name = "getSetting")]
    pub async fn get_setting(&self, key: String) -> Result<Option<JsValue>, JsValue> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let result: Option<Vec<u8>> = client.get_setting(key).await.map_err(|err| {
            platform::error_with_context(err, "failed to get setting value from the store")
        })?;
        let deserialized_result = result
            .map(|bytes| {
                to_value(&bytes).map_err(|err| {
                    platform::error_with_context(
                        err,
                        "failed to deserialize setting value into a JsValue",
                    )
                })
            })
            .transpose()?;
        Ok(deserialized_result)
    }

    /// Sets a setting key-value in the store. It can then be retrieved using `get_setting`.
    #[wasm_bindgen(js_name = "setSetting")]
    pub async fn set_setting(&self, key: String, value: JsValue) -> platform::JsResult<()> {
        let value_bytes: Vec<u8> = from_value(value).map_err(|err| {
            platform::error_with_context(err, "failed to serialize given value into bytes")
        })?;
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client.set_setting(key, value_bytes).await.map_err(|err| {
            platform::error_with_context(err, "failed to set setting value in the store")
        })?;
        Ok(())
    }
}

// napi-specific methods (get_setting, set_setting use Buffer)
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl WebClient {
    /// Retrieves the setting value for `key`, or `None` if it hasn't been set.
    pub async fn get_setting(&self, key: String) -> platform::JsResult<Option<Buffer>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let result: Option<Vec<u8>> = client.get_setting(key).await.map_err(|err| {
            platform::error_with_context(err, "failed to get setting value from the store")
        })?;

        Ok(result.map(Buffer::from))
    }

    /// Sets a setting key-value in the store. It can then be retrieved using `get_setting`.
    pub async fn set_setting(&self, key: String, value: Buffer) -> platform::JsResult<()> {
        let value_bytes: Vec<u8> = value.into();

        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client.set_setting(key, value_bytes).await.map_err(|err| {
            platform::error_with_context(err, "failed to set setting value in the store")
        })?;

        Ok(())
    }
}
