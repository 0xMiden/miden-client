//! Provides convenient access to account storage slots.

use alloc::sync::Arc;

use miden_protocol::Word;
use miden_protocol::account::{AccountId, StorageMapWitness, StorageSlotName};

use crate::errors::ClientError;
use crate::store::Store;

/// Provides convenient, name-based access to account storage slots.
///
/// `StorageReader` executes queries lazily - each method call fetches only the
/// requested slot from storage, not the entire account.
///
/// # Example
/// ```ignore
/// // Get a storage reader for an account
/// let reader = client.storage(account_id);
///
/// // Read a value slot (fetches only this slot)
/// let metadata = reader.get_item("token_metadata").await?;
///
/// // Read from a map slot (fetches only this slot)
/// let balance = reader.get_map_item("balances", user_key).await?;
///
/// // Read with witness for proofs
/// let (balance, witness) = reader.get_map_witness("balances", user_key).await?;
/// ```
pub struct StorageReader {
    store: Arc<dyn Store>,
    account_id: AccountId,
}

impl StorageReader {
    /// Creates a new `StorageReader` for the given account.
    ///
    /// This is typically called via [`Client::storage`](crate::Client::storage).
    pub fn new(store: Arc<dyn Store>, account_id: AccountId) -> Self {
        Self { store, account_id }
    }

    /// Returns the account ID this reader is associated with.
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    /// Retrieves a storage slot value by name.
    ///
    /// This method fetches only the requested slot from storage, not the entire account.
    ///
    /// For `Value` slots, returns the stored word.
    /// For `Map` slots, returns the map root.
    pub async fn get_item(
        &self,
        slot_name: impl Into<StorageSlotName>,
    ) -> Result<Word, ClientError> {
        self.store
            .get_account_storage_item(self.account_id, slot_name.into())
            .await
            .map_err(ClientError::StoreError)
    }

    /// Retrieves a value from a storage map slot by name and key.
    ///
    /// This method fetches only the requested slot from storage, not the entire account.
    ///
    /// # Errors
    /// Returns an error if the slot is not found or is not a map.
    pub async fn get_map_item(
        &self,
        slot_name: impl Into<StorageSlotName>,
        key: Word,
    ) -> Result<Word, ClientError> {
        let (value, _witness) =
            self.store.get_account_map_item(self.account_id, slot_name.into(), key).await?;
        Ok(value)
    }

    /// Retrieves a value and its Merkle witness from a storage map slot.
    ///
    /// This method fetches only the requested slot from storage, not the entire account.
    /// The witness can be used in transaction proofs.
    ///
    /// # Errors
    /// Returns an error if the slot is not found or is not a map.
    pub async fn get_map_witness(
        &self,
        slot_name: impl Into<StorageSlotName>,
        key: Word,
    ) -> Result<(Word, StorageMapWitness), ClientError> {
        self.store
            .get_account_map_item(self.account_id, slot_name.into(), key)
            .await
            .map_err(ClientError::StoreError)
    }
}
