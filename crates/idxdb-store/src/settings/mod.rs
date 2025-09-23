use alloc::string::String;
use alloc::vec::Vec;

use miden_client::store::StoreError;
use serde_wasm_bindgen::from_value;
use wasm_bindgen_futures::JsFuture;

mod js_bindings;
mod models;

use js_bindings::{idxdb_get_value, idxdb_insert_value, idxdb_list_keys, idxdb_remove_value};

use crate::WebStore;
use crate::settings::models::SettingValueIdxdbObject;

impl WebStore {
    pub(crate) async fn set_value(&self, key: String, value: Vec<u8>) -> Result<(), StoreError> {
        let promise = idxdb_insert_value(key, value);
        JsFuture::from(promise).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to set setting value: {js_error:?}",))
        })?;
        Ok(())
    }

    pub(crate) async fn get_value(&self, key: String) -> Result<Option<Vec<u8>>, StoreError> {
        let promise = idxdb_get_value(key);
        let value = JsFuture::from(promise).await.map_err(|js_error| {
            StoreError::DatabaseError(format!(
                "failed to get setting value from idxdb: {js_error:?}",
            ))
        })?;
        let setting: Option<SettingValueIdxdbObject> = from_value(value).map_err(|err| {
            StoreError::DatabaseError(format!("failed to deserialize value from idxdb: {err:?}"))
        })?;
        Ok(setting.map(|setting| setting.value))
    }

    pub(crate) async fn remove_value(&self, key: String) -> Result<(), StoreError> {
        let promise = idxdb_remove_value(key);
        JsFuture::from(promise).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to delete setting value: {js_error:?}",))
        })?;
        Ok(())
    }

    pub(crate) async fn list_keys(&self) -> Result<Vec<String>, StoreError> {
        let promise = idxdb_list_keys();
        let keys = JsFuture::from(promise).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to list setting keys: {js_error:?}",))
        })?;
        let keys: Vec<String> = from_value(keys).map_err(|err| {
            StoreError::DatabaseError(format!("failed to deserialize setting keys: {err:?}"))
        })?;
        Ok(keys)
    }
}
