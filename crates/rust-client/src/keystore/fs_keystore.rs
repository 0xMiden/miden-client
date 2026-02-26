use alloc::string::String;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::string::ToString;

use miden_protocol::Word;
use miden_protocol::account::auth::{AuthSecretKey, PublicKey, PublicKeyCommitment, Signature};
use miden_tx::AuthenticationError;
use miden_tx::auth::{SigningInputs, TransactionAuthenticator};
use miden_tx::utils::{Deserializable, Serializable};

use super::KeyStoreError;

/// A filesystem-based keystore that stores keys in separate files and provides transaction
/// authentication functionality. The public key is hashed and the result is used as the filename
/// and the contents of the file are the serialized public and secret key.
#[derive(Debug, Clone)]
pub struct FilesystemKeyStore {
    /// The directory where the keys are stored and read from.
    pub keys_directory: PathBuf,
}

impl FilesystemKeyStore {
    /// Creates a [`FilesystemKeyStore`] on a specific directory.
    pub fn new(keys_directory: PathBuf) -> Result<Self, KeyStoreError> {
        if !keys_directory.exists() {
            fs::create_dir_all(&keys_directory)
                .map_err(keystore_error("error creating keys directory"))?;
        }

        Ok(FilesystemKeyStore { keys_directory })
    }

    /// Adds a secret key to the keystore.
    pub fn add_key(&self, key: &AuthSecretKey) -> Result<(), KeyStoreError> {
        let pub_key_commitment = key.public_key().to_commitment();
        let file_path = key_file_path(&self.keys_directory, pub_key_commitment);
        write_secret_key_file(&file_path, key)
    }

    /// Removes a secret key from the keystore, given the commitment of a public key.
    pub fn remove_key(&self, pub_key: PublicKeyCommitment) -> Result<(), KeyStoreError> {
        let file_path = key_file_path(&self.keys_directory, pub_key);
        match fs::remove_file(file_path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(keystore_error("error removing secret key file")(e)),
        }
    }

    /// Retrieves a secret key from the keystore given the commitment of a public key.
    pub fn get_key(
        &self,
        pub_key: PublicKeyCommitment,
    ) -> Result<Option<AuthSecretKey>, KeyStoreError> {
        let file_path = key_file_path(&self.keys_directory, pub_key);
        match fs::read(&file_path) {
            Ok(bytes) => {
                let key = AuthSecretKey::read_from_bytes(&bytes).map_err(|err| {
                    KeyStoreError::DecodingError(format!(
                        "error reading secret key from file: {err:?}"
                    ))
                })?;
                Ok(Some(key))
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(keystore_error("error reading secret key file")(e)),
        }
    }
}

impl TransactionAuthenticator for FilesystemKeyStore {
    /// Gets a signature over a message, given a public key.
    ///
    /// The public key should correspond to one of the keys tracked by the keystore.
    ///
    /// # Errors
    /// If the public key isn't found in the store, [`AuthenticationError::UnknownPublicKey`] is
    /// returned.
    async fn get_signature(
        &self,
        pub_key: PublicKeyCommitment,
        signing_info: &SigningInputs,
    ) -> Result<Signature, AuthenticationError> {
        let message = signing_info.to_commitment();

        let secret_key = self
            .get_key(pub_key)
            .map_err(|err| {
                AuthenticationError::other_with_source("failed to load secret key", err)
            })?
            .ok_or(AuthenticationError::UnknownPublicKey(pub_key))?;

        let signature = secret_key.sign(message);

        Ok(signature)
    }

    async fn get_public_key(&self, _pub_key_commitment: PublicKeyCommitment) -> Option<&PublicKey> {
        None
    }
}

// HELPERS
// ================================================================================================

/// Returns the file path that belongs to the public key commitment.
///
/// Uses the hex representation of the public key as the filename. Falls back to legacy
/// `DefaultHasher`-based filenames and migrates them automatically.
fn key_file_path(keys_directory: &Path, pub_key: PublicKeyCommitment) -> PathBuf {
    let new_filename = Word::from(pub_key).to_hex();
    let new_path = keys_directory.join(&new_filename);
    if new_path.exists() {
        return new_path;
    }
    // Legacy fallback: try old DefaultHasher-based filename
    let legacy_filename = legacy_hash_pub_key(pub_key.into());
    let legacy_path = keys_directory.join(&legacy_filename);
    if legacy_path.exists() {
        // Migrate: rename to new convention; fall back to legacy path if rename fails
        if fs::rename(&legacy_path, &new_path).is_ok() {
            return new_path;
        }
        return legacy_path;
    }
    new_path // default to new path for new keys
}

/// Writes an [`AuthSecretKey`] into a file with restrictive permissions (0600 on Unix).
#[cfg(unix)]
fn write_secret_key_file(file_path: &Path, key: &AuthSecretKey) -> Result<(), KeyStoreError> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(file_path)
        .map_err(keystore_error("error writing secret key file"))?;
    file.write_all(&key.to_bytes())
        .map_err(keystore_error("error writing secret key file"))
}

/// Writes an [`AuthSecretKey`] into a file.
#[cfg(not(unix))]
fn write_secret_key_file(file_path: &Path, key: &AuthSecretKey) -> Result<(), KeyStoreError> {
    fs::write(file_path, key.to_bytes()).map_err(keystore_error("error writing secret key file"))
}

fn keystore_error(context: &str) -> impl FnOnce(std::io::Error) -> KeyStoreError {
    move |err| KeyStoreError::StorageError(format!("{context}: {err:?}"))
}

/// Legacy hash function for backward compatibility with existing key files.
/// Uses `DefaultHasher` which was the original filename derivation method.
/// This can be removed in a future release once all key files have been migrated.
fn legacy_hash_pub_key(pub_key: Word) -> String {
    let pub_key = pub_key.to_hex();
    let mut hasher = DefaultHasher::new();
    pub_key.hash(&mut hasher);
    hasher.finish().to_string()
}
