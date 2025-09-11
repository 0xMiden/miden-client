//! The `settings` module provides methods for managing arbitrary setting values that are persisted
//! in the client's store.

use alloc::string::{String, ToString};

use miden_objects::account::AccountId;
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

    /// Sets a setting value in the store.
    pub async fn set_setting_value<T: Serializable>(
        &mut self,
        key: String,
        value: T,
    ) -> Result<(), ClientError> {
        self.store.set_setting_value(key, value.to_bytes()).await.map_err(Into::into)
    }

    /// Retrieves a setting value from the store.
    pub async fn get_setting_value<T: Deserializable>(
        &self,
        key: String,
    ) -> Result<Option<T>, ClientError> {
        self.store
            .get_setting_value(key)
            .await
            .map(|value| value.map(|value| Deserializable::read_from_bytes(&value)))?
            .transpose()
            .map_err(Into::into)
    }

    /// Deletes the setting value from the store.
    pub async fn delete_setting_value(&mut self, key: String) -> Result<(), ClientError> {
        self.store.delete_setting_value(key).await.map_err(Into::into)
    }

    // DEFAULT ACCOUNT ID
    // --------------------------------------------------------------------------------------------

    /// Retrieves the default account ID from the store.
    pub async fn get_default_account_id(&self) -> Result<Option<AccountId>, ClientError> {
        self.get_setting_value("default_account_id".to_string()).await
    }

    /// Sets the default account ID in the store.
    pub async fn set_default_account_id(
        &mut self,
        account_id: AccountId,
    ) -> Result<(), ClientError> {
        self.set_setting_value("default_account_id".to_string(), account_id).await
    }

    /// Deletes the default account ID from the store.
    pub async fn delete_default_account_id(&mut self) -> Result<(), ClientError> {
        self.delete_setting_value("default_account_id".to_string()).await
    }
}
