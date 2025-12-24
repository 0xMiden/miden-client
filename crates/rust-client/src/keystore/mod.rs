use alloc::boxed::Box;
use alloc::string::String;

use miden_objects::address::Address;
use miden_objects::crypto::ies::UnsealingKey;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("storage error: {0}")]
    StorageError(String),
    #[error("decoding error: {0}")]
    DecodingError(String),
}

/// Trait for storing and retrieving encryption keys by address.
///
/// Encryption keys are used for end-to-end encryption of private notes.
/// Each address has an associated encryption key stored in the keystore.
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait EncryptionKeyStore: Send + Sync {
    /// Stores an encryption secret key (unsealing key) for an address.
    async fn add_encryption_key(
        &self,
        address: &Address,
        key: &UnsealingKey,
    ) -> Result<(), KeyStoreError>;

    /// Retrieves the encryption secret key (unsealing key) for an address.
    async fn get_encryption_key(
        &self,
        address: &Address,
    ) -> Result<Option<UnsealingKey>, KeyStoreError>;
}

#[cfg(feature = "std")]
mod fs_keystore;
#[cfg(feature = "std")]
pub use fs_keystore::FilesystemKeyStore;
