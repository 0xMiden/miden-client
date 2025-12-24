use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use std::fs::OpenOptions;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::string::ToString;

use miden_objects::Word;
use miden_objects::account::auth::{AuthSecretKey, PublicKey, PublicKeyCommitment, Signature};
use miden_objects::address::Address;
use miden_objects::crypto::dsa::ecdsa_k256_keccak::SecretKey as K256SecretKey;
use miden_objects::crypto::dsa::eddsa_25519::SecretKey as X25519SecretKey;
use miden_objects::crypto::ies::UnsealingKey;
use miden_tx::AuthenticationError;
use miden_tx::auth::{SigningInputs, TransactionAuthenticator};
use miden_tx::utils::{Deserializable, Serializable};

use super::{EncryptionKeyStore, KeyStoreError};

/// A filesystem-based keystore that stores keys in separate files and provides transaction
/// authentication functionality. The public key is hashed and the result is used as the filename
/// and the contents of the file are the serialized public and secret key.
#[derive(Debug, Clone)]
pub struct FilesystemKeyStore {
    /// The directory where the keys are stored and read from.
    keys_directory: PathBuf,
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

    /// Retrieves a secret key from the keystore given its public key.
    pub fn get_key(&self, pub_key: Word) -> Result<Option<AuthSecretKey>, KeyStoreError> {
        let filename = hash_pub_key(pub_key);

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
            .get_key(pub_key.into())
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

#[async_trait::async_trait]
impl EncryptionKeyStore for FilesystemKeyStore {
    async fn add_encryption_key(
        &self,
        address: &Address,
        key: &UnsealingKey,
    ) -> Result<(), KeyStoreError> {
        let encryption_dir = self.keys_directory.join("encryption");
        if !encryption_dir.exists() {
            std::fs::create_dir_all(&encryption_dir).map_err(|err| {
                KeyStoreError::StorageError(format!(
                    "error creating encryption keys directory: {err:?}"
                ))
            })?;
        }

        // Use hash of address as filename
        let filename = hash_address(address);
        let file_path = encryption_dir.join(&filename);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .map_err(|err| {
                KeyStoreError::StorageError(format!("error opening encryption key file: {err:?}"))
            })?;

        let key_bytes = serialize_unsealing_key(key);
        let mut writer = BufWriter::new(file);
        let key_hex = hex::encode(key_bytes);
        writer.write_all(key_hex.as_bytes()).map_err(|err| {
            KeyStoreError::StorageError(format!("error writing encryption key file: {err:?}"))
        })?;

        Ok(())
    }

    async fn get_encryption_key(
        &self,
        address: &Address,
    ) -> Result<Option<UnsealingKey>, KeyStoreError> {
        let encryption_dir = self.keys_directory.join("encryption");
        let filename = hash_address(address);
        let file_path = encryption_dir.join(filename);

        if !file_path.exists() {
            return Ok(None);
        }

        let file = OpenOptions::new().read(true).open(file_path).map_err(|err| {
            KeyStoreError::StorageError(format!("error opening encryption key file: {err:?}"))
        })?;
        let mut reader = BufReader::new(file);
        let mut key_hex = String::new();
        reader.read_line(&mut key_hex).map_err(|err| {
            KeyStoreError::StorageError(format!("error reading encryption key file: {err:?}"))
        })?;

        let key_bytes = hex::decode(key_hex.trim()).map_err(|err| {
            KeyStoreError::DecodingError(format!("error decoding encryption key hex: {err:?}"))
        })?;
        let key = deserialize_unsealing_key(&key_bytes)?;

        Ok(Some(key))
    }
}

/// Hashes a public key to a string representation.
fn hash_pub_key(pub_key: Word) -> String {
    let pub_key = pub_key.to_hex();
    let mut hasher = DefaultHasher::new();
    pub_key.hash(&mut hasher);
    hasher.finish().to_string()
}

/// Hashes an address to a string representation for use as a filename.
fn hash_address(address: &Address) -> String {
    use miden_tx::utils::Serializable;

    let address_bytes = address.to_bytes();
    let mut hasher = DefaultHasher::new();
    address_bytes.hash(&mut hasher);
    hasher.finish().to_string()
}

// UNSEALING KEY SERIALIZATION
// ================================================================================================

// IES scheme discriminants - must match miden-crypto's IesScheme enum order.
// TODO: Remove these helpers once miden-crypto 0.19 is available, which provides
// Serializable/Deserializable impls for UnsealingKey.
const IES_SCHEME_K256_XCHACHA20_POLY1305: u8 = 0;
const IES_SCHEME_X25519_XCHACHA20_POLY1305: u8 = 1;
const IES_SCHEME_K256_AEAD_RPO: u8 = 2;
const IES_SCHEME_X25519_AEAD_RPO: u8 = 3;

/// Serializes an [`UnsealingKey`] to bytes.
fn serialize_unsealing_key(key: &UnsealingKey) -> Vec<u8> {
    let mut bytes = Vec::new();

    match key {
        UnsealingKey::K256XChaCha20Poly1305(secret_key) => {
            bytes.push(IES_SCHEME_K256_XCHACHA20_POLY1305);
            bytes.extend_from_slice(&secret_key.to_bytes());
        },
        UnsealingKey::X25519XChaCha20Poly1305(secret_key) => {
            bytes.push(IES_SCHEME_X25519_XCHACHA20_POLY1305);
            bytes.extend_from_slice(&secret_key.to_bytes());
        },
        UnsealingKey::K256AeadRpo(secret_key) => {
            bytes.push(IES_SCHEME_K256_AEAD_RPO);
            bytes.extend_from_slice(&secret_key.to_bytes());
        },
        UnsealingKey::X25519AeadRpo(secret_key) => {
            bytes.push(IES_SCHEME_X25519_AEAD_RPO);
            bytes.extend_from_slice(&secret_key.to_bytes());
        },
    }

    bytes
}

/// Deserializes an [`UnsealingKey`] from bytes.
fn deserialize_unsealing_key(bytes: &[u8]) -> Result<UnsealingKey, KeyStoreError> {
    if bytes.is_empty() {
        return Err(KeyStoreError::DecodingError("empty bytes for unsealing key".to_string()));
    }

    let scheme = bytes[0];
    let key_bytes = &bytes[1..];

    match scheme {
        IES_SCHEME_K256_XCHACHA20_POLY1305 => {
            let secret_key = K256SecretKey::read_from_bytes(key_bytes).map_err(|e| {
                KeyStoreError::DecodingError(format!(
                    "failed to deserialize K256 secret key: {e:?}"
                ))
            })?;
            Ok(UnsealingKey::K256XChaCha20Poly1305(secret_key))
        },
        IES_SCHEME_X25519_XCHACHA20_POLY1305 => {
            let secret_key = X25519SecretKey::read_from_bytes(key_bytes).map_err(|e| {
                KeyStoreError::DecodingError(format!(
                    "failed to deserialize X25519 secret key: {e:?}"
                ))
            })?;
            Ok(UnsealingKey::X25519XChaCha20Poly1305(secret_key))
        },
        IES_SCHEME_K256_AEAD_RPO => {
            let secret_key = K256SecretKey::read_from_bytes(key_bytes).map_err(|e| {
                KeyStoreError::DecodingError(format!(
                    "failed to deserialize K256 secret key: {e:?}"
                ))
            })?;
            Ok(UnsealingKey::K256AeadRpo(secret_key))
        },
        IES_SCHEME_X25519_AEAD_RPO => {
            let secret_key = X25519SecretKey::read_from_bytes(key_bytes).map_err(|e| {
                KeyStoreError::DecodingError(format!(
                    "failed to deserialize X25519 secret key: {e:?}"
                ))
            })?;
            Ok(UnsealingKey::X25519AeadRpo(secret_key))
        },
        _ => Err(KeyStoreError::DecodingError(format!("unsupported IES scheme: {scheme}"))),
    }
}
