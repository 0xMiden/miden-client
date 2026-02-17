use alloc::string::String;

use thiserror::Error;

use crate::errors::ErrorCode;

#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("storage error: {0}")]
    StorageError(String),
    #[error("decoding error: {0}")]
    DecodingError(String),
}

impl ErrorCode for KeyStoreError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::StorageError(_) => "MIDEN-KS-001",
            Self::DecodingError(_) => "MIDEN-KS-002",
        }
    }
}

#[cfg(feature = "std")]
mod fs_keystore;
#[cfg(feature = "std")]
pub use fs_keystore::FilesystemKeyStore;
