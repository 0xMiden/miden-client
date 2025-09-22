use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "getValue")]
    pub async fn get_value(&mut self, key: String) -> Result<Option<JsValue>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let result: Option<Vec<u8>> = client.get_value(key).await.map_err(|err| {
                js_error_with_context(err, "failed to get setting value from the store")
            })?;
            let deserialized_result = result
                .map(|bytes| {
                    to_value(&bytes).map_err(|err| {
                        js_error_with_context(
                            err,
                            "failed to deserialize setting value into a JsValue",
                        )
                    })
                })
                .transpose()?;
            Ok(deserialized_result)
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "setValue")]
    pub async fn set_value(&mut self, key: String, value: JsValue) -> Result<(), JsValue> {
        let value_bytes: Vec<u8> = from_value(value).map_err(|err| {
            js_error_with_context(err, "failed to serialize given value into bytes")
        })?;
        if let Some(client) = self.get_mut_inner() {
            client.set_value(key, value_bytes).await.map_err(|err| {
                js_error_with_context(err, "failed to set setting value in the store")
            })?;
            Ok(())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "removeValue")]
    pub async fn remove_value(&mut self, key: String) -> Result<(), JsValue> {
        if let Some(client) = self.get_mut_inner() {
            client.remove_value(key).await.map_err(|err| {
                js_error_with_context(err, "failed to delete setting value in the store")
            })?;
            Ok(())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "listKeys")]
    pub async fn list_keys(&mut self) -> Result<Vec<String>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            client.list_keys().await.map_err(|err| {
                js_error_with_context(err, "failed to list setting keys in the store")
            })
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }
}
