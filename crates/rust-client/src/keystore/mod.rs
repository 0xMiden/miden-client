use alloc::string::String;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("storage error: {0}")]
    StorageError(String),
    #[error("decoding error: {0}")]
    DecodingError(String),
}

mod common;
#[cfg(feature = "std")]
mod fs_keystore;
#[cfg(feature = "std")]
pub use fs_keystore::FilesystemKeyStore;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
mod web_keystore;
#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use web_keystore::WebKeyStore;
