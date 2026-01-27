//! Provides convenient access to account storage slots.

use miden_protocol::account::{
    AccountStorage, StorageMapWitness, StorageSlot, StorageSlotContent, StorageSlotName,
};
use miden_protocol::Word;

use crate::errors::AccountError;

/// Provides convenient, name-based access to account storage slots.
///
/// # Example
/// ```ignore
/// let account = client.get_account(account_id).await?.unwrap();
/// let reader = StorageReader::new(account.account().storage().clone());
///
/// // Read a value slot
/// let metadata = reader.get_item("token_metadata")?;
///
/// // Read from a map slot
/// let balance = reader.get_map_item("balances", user_key)?;
///
/// // Read with witness for proofs
/// let (balance, witness) = reader.get_map_witness("balances", user_key)?;
/// ```
pub struct StorageReader {
    storage: AccountStorage,
}

impl StorageReader {
    /// Creates a new `StorageReader` from an `AccountStorage`.
    pub fn new(storage: AccountStorage) -> Self {
        Self { storage }
    }

    /// Creates a new `StorageReader` from an `Account`.
    pub fn from_account(account: &miden_protocol::account::Account) -> Self {
        Self::new(account.storage().clone())
    }

    /// Returns a reference to the underlying storage.
    pub fn storage(&self) -> &AccountStorage {
        &self.storage
    }

    /// Returns the storage commitment.
    pub fn commitment(&self) -> Word {
        self.storage.to_commitment()
    }

    /// Retrieves a storage slot value by name.
    ///
    /// For `Value` slots, returns the stored word.
    /// For `Map` slots, returns the map root.
    pub fn get_item(&self, slot_name: impl Into<StorageSlotName>) -> Result<Word, AccountError> {
        let slot_name = slot_name.into();
        self.storage
            .get(&slot_name)
            .map(StorageSlot::value)
            .ok_or(AccountError::StorageSlotNameNotFound { slot_name })
    }

    /// Retrieves a value from a storage map slot by name and key.
    ///
    /// # Errors
    /// Returns an error if the slot is not found or is not a map.
    pub fn get_map_item(
        &self,
        slot_name: impl Into<StorageSlotName>,
        key: Word,
    ) -> Result<Word, AccountError> {
        let slot_name = slot_name.into();
        match self.storage.get(&slot_name).map(StorageSlot::content) {
            Some(StorageSlotContent::Map(map)) => Ok(map.get(&key)),
            Some(StorageSlotContent::Value(_)) => Err(AccountError::StorageSlotNotMap(slot_name)),
            None => Err(AccountError::StorageSlotNameNotFound { slot_name }),
        }
    }

    /// Retrieves a value and its Merkle witness from a storage map slot.
    ///
    /// The witness can be used in transaction proofs.
    ///
    /// # Errors
    /// Returns an error if the slot is not found or is not a map.
    pub fn get_map_witness(
        &self,
        slot_name: impl Into<StorageSlotName>,
        key: Word,
    ) -> Result<(Word, StorageMapWitness), AccountError> {
        let slot_name = slot_name.into();
        match self.storage.get(&slot_name).map(StorageSlot::content) {
            Some(StorageSlotContent::Map(map)) => {
                let value = map.get(&key);
                let witness = map.open(&key);
                Ok((value, witness))
            },
            Some(StorageSlotContent::Value(_)) => Err(AccountError::StorageSlotNotMap(slot_name)),
            None => Err(AccountError::StorageSlotNameNotFound { slot_name }),
        }
    }

    /// Returns the storage slot with the given name, if it exists.
    pub fn get_slot(&self, slot_name: impl Into<StorageSlotName>) -> Option<&StorageSlot> {
        self.storage.get(&slot_name.into())
    }

    /// Returns an iterator over all storage slots.
    pub fn slots(&self) -> impl Iterator<Item = &StorageSlot> {
        self.storage.slots().iter()
    }
}

impl From<AccountStorage> for StorageReader {
    fn from(storage: AccountStorage) -> Self {
        Self::new(storage)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use miden_protocol::account::{AccountStorage, StorageMap, StorageSlot, StorageSlotName};

    use super::*;

    fn value_slot_name() -> StorageSlotName {
        StorageSlotName::new("miden::testing::storage_reader::value").expect("valid slot name")
    }

    fn map_slot_name() -> StorageSlotName {
        StorageSlotName::new("miden::testing::storage_reader::map").expect("valid slot name")
    }

    fn test_value() -> Word {
        Word::from([1u32, 1, 1, 1])
    }

    fn test_key() -> Word {
        Word::from([1u32, 2, 3, 4])
    }

    fn test_map_value() -> Word {
        Word::from([10u32, 20, 30, 40])
    }

    fn empty_word() -> Word {
        Word::default()
    }

    fn create_test_storage() -> AccountStorage {
        let mut storage_map = StorageMap::new();
        storage_map.insert(test_key(), test_map_value()).expect("insert should succeed");

        let slots = vec![
            StorageSlot::with_value(value_slot_name(), test_value()),
            StorageSlot::with_map(map_slot_name(), storage_map),
        ];

        AccountStorage::new(slots).expect("storage creation should succeed")
    }

    #[test]
    fn test_new_and_storage() {
        let storage = create_test_storage();
        let reader = StorageReader::new(storage.clone());

        assert_eq!(reader.storage().to_commitment(), storage.to_commitment());
    }

    #[test]
    fn test_from_account_storage() {
        let storage = create_test_storage();
        let reader: StorageReader = storage.clone().into();

        assert_eq!(reader.storage().to_commitment(), storage.to_commitment());
    }

    #[test]
    fn test_commitment() {
        let storage = create_test_storage();
        let reader = StorageReader::new(storage.clone());

        assert_eq!(reader.commitment(), storage.to_commitment());
    }

    #[test]
    fn test_get_item_value_slot() {
        let reader = StorageReader::new(create_test_storage());

        let result = reader.get_item(value_slot_name());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_value());
    }

    #[test]
    fn test_get_item_map_slot_returns_root() {
        let reader = StorageReader::new(create_test_storage());

        // For map slots, get_item returns the map root
        let result = reader.get_item(map_slot_name());
        assert!(result.is_ok());
        // The result should be a valid word (the map root)
        let root = result.unwrap();
        assert_ne!(root, empty_word());
    }

    #[test]
    fn test_get_item_not_found() {
        let reader = StorageReader::new(create_test_storage());
        let missing_name =
            StorageSlotName::new("miden::testing::storage_reader::missing").expect("valid name");

        let result = reader.get_item(missing_name.clone());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AccountError::StorageSlotNameNotFound { slot_name } if slot_name == missing_name
        ));
    }

    #[test]
    fn test_get_map_item_success() {
        let reader = StorageReader::new(create_test_storage());

        let result = reader.get_map_item(map_slot_name(), test_key());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_map_value());
    }

    #[test]
    fn test_get_map_item_missing_key_returns_empty() {
        let reader = StorageReader::new(create_test_storage());
        let missing_key = Word::from([99u32, 99, 99, 99]);

        // Missing keys in a storage map return empty word (not an error)
        let result = reader.get_map_item(map_slot_name(), missing_key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), empty_word());
    }

    #[test]
    fn test_get_map_item_value_slot_error() {
        let reader = StorageReader::new(create_test_storage());

        let result = reader.get_map_item(value_slot_name(), test_key());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AccountError::StorageSlotNotMap(name) if name == value_slot_name()
        ));
    }

    #[test]
    fn test_get_map_item_not_found() {
        let reader = StorageReader::new(create_test_storage());
        let missing_name =
            StorageSlotName::new("miden::testing::storage_reader::missing").expect("valid name");

        let result = reader.get_map_item(missing_name.clone(), test_key());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AccountError::StorageSlotNameNotFound { slot_name } if slot_name == missing_name
        ));
    }

    #[test]
    fn test_get_map_witness_success() {
        let reader = StorageReader::new(create_test_storage());

        let result = reader.get_map_witness(map_slot_name(), test_key());
        assert!(result.is_ok());
        let (value, _witness) = result.unwrap();
        assert_eq!(value, test_map_value());
    }

    #[test]
    fn test_get_map_witness_value_slot_error() {
        let reader = StorageReader::new(create_test_storage());

        let result = reader.get_map_witness(value_slot_name(), test_key());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AccountError::StorageSlotNotMap(name) if name == value_slot_name()
        ));
    }

    #[test]
    fn test_get_map_witness_not_found() {
        let reader = StorageReader::new(create_test_storage());
        let missing_name =
            StorageSlotName::new("miden::testing::storage_reader::missing").expect("valid name");

        let result = reader.get_map_witness(missing_name.clone(), test_key());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AccountError::StorageSlotNameNotFound { slot_name } if slot_name == missing_name
        ));
    }

    #[test]
    fn test_get_slot_exists() {
        let reader = StorageReader::new(create_test_storage());

        let slot = reader.get_slot(value_slot_name());
        assert!(slot.is_some());
        assert_eq!(slot.unwrap().value(), test_value());
    }

    #[test]
    fn test_get_slot_not_found() {
        let reader = StorageReader::new(create_test_storage());
        let missing_name =
            StorageSlotName::new("miden::testing::storage_reader::missing").expect("valid name");

        let slot = reader.get_slot(missing_name);
        assert!(slot.is_none());
    }

    #[test]
    fn test_slots_iterator() {
        let reader = StorageReader::new(create_test_storage());

        let slots: Vec<_> = reader.slots().collect();
        assert_eq!(slots.len(), 2);
    }
}
