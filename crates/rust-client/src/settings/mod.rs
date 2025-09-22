//! The `settings` module provides methods for managing arbitrary setting values that are persisted
//! in the client's store.

use alloc::string::String;
use alloc::vec::Vec;

use miden_tx::utils::{Deserializable, Serializable};

use super::Client;
use crate::errors::ClientError;

// CLIENT METHODS
// ================================================================================================

/// This section of the [Client] contains methods for:
///
/// - **Settings accessors:** Methods to get, set, and delete setting values from the store.
/// - **Default account ID:** Methods to get, set, and delete the default account ID. This is a
///   wrapper around a specific setting value.
impl<AUTH> Client<AUTH> {
    // SETTINGS ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Sets a setting value in the store. It can then be retrieved using `get_value`.
    pub async fn set_value<T: Serializable>(
        &mut self,
        key: String,
        value: T,
    ) -> Result<(), ClientError> {
        self.store.set_value(key, value.to_bytes()).await.map_err(Into::into)
    }

    /// Retrieves the value for `key`, or `None` if it hasn’t been set.
    pub async fn get_value<T: Deserializable>(
        &self,
        key: String,
    ) -> Result<Option<T>, ClientError> {
        self.store
            .get_value(key)
            .await
            .map(|value| value.map(|value| Deserializable::read_from_bytes(&value)))?
            .transpose()
            .map_err(Into::into)
    }

    /// Deletes the setting value from the store.
    pub async fn remove_value(&mut self, key: String) -> Result<(), ClientError> {
        self.store.remove_value(key).await.map_err(Into::into)
    }

    /// Returns all the setting keys from the store.
    pub async fn list_keys(&self) -> Result<Vec<String>, ClientError> {
        self.store.list_keys().await.map_err(Into::into)
    }
}
