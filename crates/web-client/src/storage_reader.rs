use alloc::sync::Arc;

use idxdb_store::WebStore;
use miden_client::account::StorageSlotName;
use miden_client::store::Store;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::account_id::AccountId;
use crate::models::word::Word;

/// Provides convenient, name-based access to account storage slots.
///
/// `StorageReader` executes queries lazily - each method call fetches only the
/// requested slot from storage.
///
/// # Example (JavaScript)
/// ```javascript
/// // Get a storage reader for an account
/// const reader = client.newStorageReader(accountId);
///
/// // Read a value slot
/// const metadata = await reader.getItem("token_metadata");
///
/// // Read from a map slot
/// const balance = await reader.getMapItem("balances", userKey);
/// ```
#[wasm_bindgen]
pub struct StorageReader {
    store: Arc<WebStore>,
    account_id: miden_client::account::AccountId,
}

#[wasm_bindgen]
impl StorageReader {
    /// Returns the account ID this reader is associated with.
    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.account_id.into()
    }

    /// Retrieves a storage slot value by name.
    ///
    /// For `Value` slots, returns the stored word.
    /// For `Map` slots, returns the map root.
    ///
    /// # Arguments
    /// * `slot_name` - The name of the storage slot.
    ///
    /// # Errors
    /// Returns an error if the slot is not found.
    #[wasm_bindgen(js_name = "getItem")]
    pub async fn get_item(&self, slot_name: &str) -> Result<Word, JsValue> {
        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| js_error_with_context(err, "invalid slot name"))?;

        let value = self
            .store
            .get_account_storage_item(self.account_id, slot_name)
            .await
            .map_err(|err| js_error_with_context(err, "failed to get storage item"))?;

        Ok(value.into())
    }

    /// Retrieves a value from a storage map slot by name and key.
    ///
    /// # Arguments
    /// * `slot_name` - The name of the storage map slot.
    /// * `key` - The key within the map.
    ///
    /// # Errors
    /// Returns an error if the slot is not found or is not a map.
    #[wasm_bindgen(js_name = "getMapItem")]
    pub async fn get_map_item(&self, slot_name: &str, key: &Word) -> Result<Word, JsValue> {
        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| js_error_with_context(err, "invalid slot name"))?;

        let (value, _witness) = self
            .store
            .get_account_map_item(self.account_id, slot_name, *key.as_native())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get storage map item"))?;

        Ok(value.into())
    }
}

impl StorageReader {
    /// Creates a new `StorageReader` for the given account.
    pub(crate) fn new(store: Arc<WebStore>, account_id: miden_client::account::AccountId) -> Self {
        Self { store, account_id }
    }
}
