use alloc::string::String;
use std::fs::OpenOptions;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
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
            std::fs::create_dir_all(&keys_directory).map_err(|err| {
                KeyStoreError::StorageError(format!("error creating keys directory: {err:?}"))
            })?;
        }

        Ok(FilesystemKeyStore { keys_directory })
    }

    /// Adds a secret key to the keystore.
    pub fn add_key(&self, key: &AuthSecretKey) -> Result<(), KeyStoreError> {
        let public_key = key.public_key();
        let pub_key_commitment = public_key.to_commitment();

        let filename = hash_pub_key(pub_key_commitment.into());

        let file_path = self.keys_directory.join(filename);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .map_err(|err| {
                KeyStoreError::StorageError(format!("error opening secret key file: {err:?}"))
            })?;

        let mut writer = BufWriter::new(file);
        let key_pair_hex = hex::encode(key.to_bytes());
        writer.write_all(key_pair_hex.as_bytes()).map_err(|err| {
            KeyStoreError::StorageError(format!("error writing secret key file: {err:?}"))
        })?;

        Ok(())
    }

    /// Retrieves a secret key from the keystore given the commitment of a public key.
    pub fn get_key(
        &self,
        pub_key: PublicKeyCommitment,
    ) -> Result<Option<AuthSecretKey>, KeyStoreError> {
        let filename = hash_pub_key(pub_key.into());

        let file_path = self.keys_directory.join(filename);
        if !file_path.exists() {
            return Ok(None);
        }

        let file = OpenOptions::new().read(true).open(file_path).map_err(|err| {
            KeyStoreError::StorageError(format!("error opening secret key file: {err:?}"))
        })?;
        let mut reader = BufReader::new(file);
        let mut key_pair_hex = String::new();
        reader.read_line(&mut key_pair_hex).map_err(|err| {
            KeyStoreError::StorageError(format!("error reading secret key file: {err:?}"))
        })?;

        let secret_key_bytes = hex::decode(key_pair_hex.trim()).map_err(|err| {
            KeyStoreError::DecodingError(format!("error decoding secret key hex: {err:?}"))
        })?;
        let secret_key =
            AuthSecretKey::read_from_bytes(secret_key_bytes.as_slice()).map_err(|err| {
                KeyStoreError::DecodingError(format!(
                    "error reading secret key from bytes: {err:?}"
                ))
            })?;

        Ok(Some(secret_key))
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

/// Hashes a public key to a string representation.
fn hash_pub_key(pub_key: Word) -> String {
    let pub_key = pub_key.to_hex();
    let mut hasher = DefaultHasher::new();
    pub_key.hash(&mut hasher);
    hasher.finish().to_string()
}
