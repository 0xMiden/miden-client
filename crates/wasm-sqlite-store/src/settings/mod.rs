use alloc::string::String;
use alloc::vec::Vec;

use miden_client::store::StoreError;

mod js_bindings;
mod models;

use js_bindings::{js_get_setting, js_insert_setting, js_list_setting_keys, js_remove_setting};

use crate::WasmSqliteStore;
use crate::settings::models::SettingValueObject;

impl WasmSqliteStore {
    #[allow(clippy::unused_async)]
    pub(crate) async fn set_setting(&self, key: String, value: Vec<u8>) -> Result<(), StoreError> {
        js_insert_setting(self.db_id(), key, value);
        Ok(())
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_setting(&self, key: String) -> Result<Option<Vec<u8>>, StoreError> {
        let js_value = js_get_setting(self.db_id(), key);
        if js_value.is_null() || js_value.is_undefined() {
            return Ok(None);
        }
        let setting: SettingValueObject =
            serde_wasm_bindgen::from_value(js_value).map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize setting: {err:?}"))
            })?;
        Ok(Some(setting.value))
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn remove_setting(&self, key: String) -> Result<(), StoreError> {
        js_remove_setting(self.db_id(), key);
        Ok(())
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn list_setting_keys(&self) -> Result<Vec<String>, StoreError> {
        let js_value = js_list_setting_keys(self.db_id());
        let keys: Vec<String> = serde_wasm_bindgen::from_value(js_value).map_err(|err| {
            StoreError::DatabaseError(format!("failed to deserialize setting keys: {err:?}"))
        })?;
        Ok(keys)
    }
}
