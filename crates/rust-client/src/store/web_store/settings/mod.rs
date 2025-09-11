use alloc::string::String;
use alloc::vec::Vec;

use serde_wasm_bindgen::from_value;
use wasm_bindgen_futures::JsFuture;

use crate::store::StoreError;
use crate::store::web_store::WebStore;
use crate::store::web_store::settings::models::SettingValueIdxdbObject;

mod js_bindings;
mod models;

use js_bindings::{
    idxdb_delete_setting_value,
    idxdb_get_setting_value,
    idxdb_insert_setting_value,
};

impl WebStore {
    pub(crate) async fn set_setting_value(
        &self,
        key: String,
        value: Vec<u8>,
    ) -> Result<(), StoreError> {
        let promise = idxdb_insert_setting_value(key, value);
        JsFuture::from(promise).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to set setting value: {js_error:?}",))
        })?;
        Ok(())
    }

    pub(crate) async fn get_setting_value(
        &self,
        key: String,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let promise = idxdb_get_setting_value(key);
        let value = JsFuture::from(promise).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to get setting value from idxdb: {js_error:?}",))
        })?;
        let setting: Option<SettingValueIdxdbObject> = from_value(value)
            .map_err(|err| StoreError::DatabaseError(format!("failed to deserialize value from idxdb: {err:?}")))?;
        Ok(setting.map(|setting| setting.value))
    }

    pub(crate) async fn delete_setting_value(&self, key: String) -> Result<(), StoreError> {
        let promise = idxdb_delete_setting_value(key);
        JsFuture::from(promise).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to delete setting value: {js_error:?}",))
        })?;
        Ok(())
    }
}
