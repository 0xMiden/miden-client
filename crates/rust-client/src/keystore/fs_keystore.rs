use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::string::ToString;
use std::sync::Arc;

use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::account::auth::{AuthSecretKey, PublicKey, PublicKeyCommitment, Signature};
use miden_tx::AuthenticationError;
use miden_tx::auth::{SigningInputs, TransactionAuthenticator};
use miden_tx::utils::sync::RwLock;
use miden_tx::utils::{Deserializable, Serializable};
use serde::{Deserialize, Serialize};

use super::encryption::{KeyEncryptor, is_encrypted};
use super::{KeyStoreError, Keystore};

// INDEX FILE
// ================================================================================================

const INDEX_FILE_NAME: &str = "key_index.json";
const INDEX_VERSION: u32 = 1;

/// The structure of the key index file that maps account IDs to public key commitments.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct KeyIndex {
    version: u32,
    /// Maps account ID (hex) to a set of public key commitment (hex).
    mappings: BTreeMap<String, BTreeSet<String>>,
}

impl KeyIndex {
    fn new() -> Self {
        Self {
            version: INDEX_VERSION,
            mappings: BTreeMap::new(),
        }
    }

    /// Adds a mapping from account ID to public key commitment.
    fn add_mapping(&mut self, account_id: &AccountId, pub_key_commitment: PublicKeyCommitment) {
        let account_id_hex = account_id.to_hex();
        let pub_key_hex = Word::from(pub_key_commitment).to_hex();

        self.mappings.entry(account_id_hex).or_default().insert(pub_key_hex);
    }

    /// Removes all mappings for a given public key commitment.
    fn remove_all_mappings_for_key(&mut self, pub_key_commitment: PublicKeyCommitment) {
        let pub_key_hex = Word::from(pub_key_commitment).to_hex();

        // Remove the key from all account mappings
        self.mappings.retain(|_, commitments| {
            commitments.remove(&pub_key_hex);
            !commitments.is_empty()
        });
    }

    /// Loads the index from disk, or creates a new one if it doesn't exist.
    fn read_from_file(keys_directory: &Path) -> Result<Self, KeyStoreError> {
        let index_path = keys_directory.join(INDEX_FILE_NAME);

        if !index_path.exists() {
            return Ok(Self::new());
        }

        let contents =
            fs::read_to_string(&index_path).map_err(keystore_error("error reading index file"))?;

        serde_json::from_str(&contents).map_err(|err| {
            KeyStoreError::DecodingError(format!("error parsing index file: {err:?}"))
        })
    }

    /// Saves the index to disk atomically (write to temp file, then rename).
    fn write_to_file(&self, keys_directory: &Path) -> Result<(), KeyStoreError> {
        let index_path = keys_directory.join(INDEX_FILE_NAME);
        let temp_path = std::env::temp_dir().join(INDEX_FILE_NAME);

        let contents = serde_json::to_string_pretty(self).map_err(|err| {
            KeyStoreError::StorageError(format!("error serializing index: {err:?}"))
        })?;

        // Write to temp file
        let mut file = fs::File::create(&temp_path)
            .map_err(keystore_error("error creating temp index file"))?;
        file.write_all(contents.as_bytes())
            .map_err(keystore_error("error writing temp index file"))?;
        file.sync_all().map_err(keystore_error("error syncing temp index file"))?;

        // Atomically rename
        fs::rename(&temp_path, &index_path).map_err(keystore_error("error renaming index file"))
    }

    /// Returns the account ID associated with a given public key commitment hex.
    ///
    /// Iterates over all mappings to find which account contains the commitment.
    /// Returns `None` if no account is found.
    fn get_account_id(&self, pub_key_commitment: PublicKeyCommitment) -> Option<AccountId> {
        let pub_key_hex = Word::from(pub_key_commitment).to_hex();

        for (account_id_hex, commitments) in &self.mappings {
            if commitments.contains(&pub_key_hex) {
                return AccountId::from_hex(account_id_hex).ok();
            }
        }

        None
    }

    /// Gets all public key commitments for an account ID.
    fn get_commitments(
        &self,
        account_id: &AccountId,
    ) -> Result<BTreeSet<PublicKeyCommitment>, KeyStoreError> {
        let account_id_hex = account_id.to_hex();

        self.mappings
            .get(&account_id_hex)
            .map(|commitments| {
                commitments
                    .iter()
                    .filter_map(|hex| {
                        Word::try_from(hex.as_str()).ok().map(PublicKeyCommitment::from)
                    })
                    .collect()
            })
            .ok_or_else(|| {
                KeyStoreError::StorageError(format!("account not found {account_id_hex}"))
            })
    }
}

// FILESYSTEM KEYSTORE
// ================================================================================================

/// A filesystem-based keystore that stores keys in separate files and provides transaction
/// authentication functionality. The public key is hashed and the result is used as the filename
/// and the contents of the file are the serialized public and secret key.
///
/// Account-to-key mappings are stored in a separate JSON index file.
///
/// Optionally supports encrypting secret key files at rest via a [`KeyEncryptor`]. When an
/// encryptor is set (see [`with_encryption`](Self::with_encryption)), keys are encrypted on
/// write and decrypted on read. Existing plaintext key files are auto-detected and can still be
/// read even when encryption is enabled.
pub struct FilesystemKeyStore {
    /// The directory where the keys are stored and read from.
    pub keys_directory: PathBuf,
    /// The in-memory index of account-to-key mappings.
    index: RwLock<KeyIndex>,
    /// Optional encryptor for secret key files.
    encryptor: Option<Arc<dyn KeyEncryptor>>,
}

impl std::fmt::Debug for FilesystemKeyStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilesystemKeyStore")
            .field("keys_directory", &self.keys_directory)
            .field("encrypted", &self.encryptor.is_some())
            .finish()
    }
}

impl Clone for FilesystemKeyStore {
    fn clone(&self) -> Self {
        let index = self.index.read().clone();
        Self {
            keys_directory: self.keys_directory.clone(),
            index: RwLock::new(index),
            encryptor: self.encryptor.clone(),
        }
    }
}

impl FilesystemKeyStore {
    /// Creates a [`FilesystemKeyStore`] on a specific directory.
    ///
    /// Keys are stored and read as plaintext. To enable encryption, chain
    /// [`with_encryption`](Self::with_encryption) after construction.
    pub fn new(keys_directory: PathBuf) -> Result<Self, KeyStoreError> {
        if !keys_directory.exists() {
            fs::create_dir_all(&keys_directory)
                .map_err(keystore_error("error creating keys directory"))?;
        }

        let index = KeyIndex::read_from_file(&keys_directory)?;

        Ok(FilesystemKeyStore {
            keys_directory,
            index: RwLock::new(index),
            encryptor: None,
        })
    }

    /// Enables encryption for this keystore using the given encryptor.
    ///
    /// When set, new keys are encrypted on write. On read, encrypted files are decrypted
    /// automatically, and plaintext files are still readable (backward compatible).
    #[must_use]
    pub fn with_encryption(mut self, encryptor: impl KeyEncryptor + 'static) -> Self {
        self.encryptor = Some(Arc::new(encryptor));
        self
    }

    /// Re-encrypts all existing plaintext key files using the configured encryptor.
    ///
    /// Returns the number of files that were migrated. Files that are already encrypted are
    /// skipped. Deduplicates file paths to avoid re-encrypting the same file twice (a key
    /// commitment can appear under multiple accounts).
    ///
    /// # Errors
    ///
    /// Returns an error if no encryptor is configured, or if any file cannot be read/written.
    pub fn migrate_to_encrypted(&self) -> Result<usize, KeyStoreError> {
        let encryptor = self.encryptor.as_ref().ok_or_else(|| {
            KeyStoreError::StorageError("no encryptor configured for migration".into())
        })?;

        let index = self.index.read();
        let mut migrated = 0usize;

        // Collect unique file paths from the index to avoid processing duplicates
        let mut seen_paths = BTreeSet::new();
        for commitments in index.mappings.values() {
            for hex in commitments {
                if let Ok(word) = Word::try_from(hex.as_str()) {
                    let commitment = PublicKeyCommitment::from(word);
                    let file_path = key_file_path(&self.keys_directory, commitment);
                    if file_path.exists() {
                        seen_paths.insert(file_path);
                    }
                }
            }
        }

        for file_path in seen_paths {
            let bytes =
                fs::read(&file_path).map_err(keystore_error("error reading key file for migration"))?;

            // Skip files that are already encrypted
            if is_encrypted(&bytes) {
                continue;
            }

            let encrypted = encryptor.encrypt(&bytes)?;
            fs::write(&file_path, encrypted)
                .map_err(keystore_error("error writing encrypted key file"))?;
            migrated += 1;
        }

        Ok(migrated)
    }

    /// Adds a secret key to the keystore without updating account mappings.
    ///
    /// This is an internal method. Use [`Keystore::add_key`] instead.
    fn add_key_without_account(&self, key: &AuthSecretKey) -> Result<(), KeyStoreError> {
        let pub_key_commitment = key.public_key().to_commitment();
        let file_path = key_file_path(&self.keys_directory, pub_key_commitment);
        self.write_secret_key_file(&file_path, key)
    }

    /// Retrieves a secret key from the keystore given the commitment of a public key.
    pub fn get_key_sync(
        &self,
        pub_key: PublicKeyCommitment,
    ) -> Result<Option<AuthSecretKey>, KeyStoreError> {
        let file_path = key_file_path(&self.keys_directory, pub_key);
        if !file_path.exists() {
            return Ok(None);
        }

        let secret_key = self.read_secret_key_file(&file_path)?;
        Ok(Some(secret_key))
    }

    /// Saves the index to disk.
    fn save_index(&self) -> Result<(), KeyStoreError> {
        let index = self.index.read();
        index.write_to_file(&self.keys_directory)
    }

    /// Reads a secret key from a file, auto-detecting encryption.
    ///
    /// If the file starts with the `MENC` magic header, it is decrypted using the configured
    /// encryptor. If the file is plaintext, it is deserialized directly. Attempting to read an
    /// encrypted file without a configured encryptor returns an error.
    fn read_secret_key_file(&self, file_path: &Path) -> Result<AuthSecretKey, KeyStoreError> {
        let bytes =
            fs::read(file_path).map_err(keystore_error("error reading secret key file"))?;

        let key_bytes = if is_encrypted(&bytes) {
            let encryptor = self.encryptor.as_ref().ok_or_else(|| {
                KeyStoreError::DecodingError(
                    "key file is encrypted but no encryptor is configured; \
                     set MIDEN_KEYSTORE_PASSWORD or configure an encryptor"
                        .into(),
                )
            })?;
            encryptor.decrypt(&bytes)?
        } else {
            bytes
        };

        AuthSecretKey::read_from_bytes(key_bytes.as_slice()).map_err(|err| {
            KeyStoreError::DecodingError(format!("error reading secret key from file: {err:?}"))
        })
    }

    /// Writes a secret key to a file, encrypting if an encryptor is configured.
    fn write_secret_key_file(
        &self,
        file_path: &Path,
        key: &AuthSecretKey,
    ) -> Result<(), KeyStoreError> {
        let plaintext = key.to_bytes();

        let data = if let Some(encryptor) = &self.encryptor {
            encryptor.encrypt(&plaintext)?
        } else {
            plaintext.to_vec()
        };

        fs::write(file_path, data).map_err(keystore_error("error writing secret key file"))
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
            .get_key_sync(pub_key)
            .map_err(|err| {
                AuthenticationError::other_with_source("failed to load secret key", err)
            })?
            .ok_or(AuthenticationError::UnknownPublicKey(pub_key))?;

        let signature = secret_key.sign(message);

        Ok(signature)
    }

    /// Retrieves a public key for a specific public key commitment.
    async fn get_public_key(
        &self,
        pub_key_commitment: PublicKeyCommitment,
    ) -> Option<Arc<PublicKey>> {
        self.get_key(pub_key_commitment)
            .await
            .ok()
            .flatten()
            .map(|key| Arc::new(key.public_key()))
    }
}

#[async_trait::async_trait]
impl Keystore for FilesystemKeyStore {
    async fn add_key(
        &self,
        key: &AuthSecretKey,
        account_id: AccountId,
    ) -> Result<(), KeyStoreError> {
        let pub_key_commitment = key.public_key().to_commitment();

        // Write the key file
        self.add_key_without_account(key)?;

        // Update the index
        {
            let mut index = self.index.write();
            index.add_mapping(&account_id, pub_key_commitment);
        }

        // Persist the index
        self.save_index()?;

        Ok(())
    }

    async fn remove_key(&self, pub_key: PublicKeyCommitment) -> Result<(), KeyStoreError> {
        // Remove from index first
        {
            let mut index = self.index.write();
            index.remove_all_mappings_for_key(pub_key);
        }

        // Persist the index
        self.save_index()?;

        // Remove the key file
        let file_path = key_file_path(&self.keys_directory, pub_key);
        if file_path.exists() {
            fs::remove_file(file_path).map_err(keystore_error("error removing secret key file"))?;
        }

        Ok(())
    }

    async fn get_key(
        &self,
        pub_key: PublicKeyCommitment,
    ) -> Result<Option<AuthSecretKey>, KeyStoreError> {
        self.get_key_sync(pub_key)
    }

    async fn get_account_id_by_key_commitment(
        &self,
        pub_key_commitment: PublicKeyCommitment,
    ) -> Result<Option<AccountId>, KeyStoreError> {
        let index = self.index.read();
        Ok(index.get_account_id(pub_key_commitment))
    }

    async fn get_account_key_commitments(
        &self,
        account_id: &AccountId,
    ) -> Result<BTreeSet<PublicKeyCommitment>, KeyStoreError> {
        let index = self.index.read();
        index.get_commitments(account_id)
    }
}

// HELPERS
// ================================================================================================

/// Returns the file path that belongs to the public key commitment
fn key_file_path(keys_directory: &Path, pub_key: PublicKeyCommitment) -> PathBuf {
    let filename = hash_pub_key(pub_key.into());
    keys_directory.join(filename)
}

fn keystore_error(context: &str) -> impl FnOnce(std::io::Error) -> KeyStoreError {
    move |err| KeyStoreError::StorageError(format!("{context}: {err:?}"))
}

/// Hashes a public key to a string representation.
fn hash_pub_key(pub_key: Word) -> String {
    let pub_key = pub_key.to_hex();
    let mut hasher = DefaultHasher::new();
    pub_key.hash(&mut hasher);
    hasher.finish().to_string()
}

