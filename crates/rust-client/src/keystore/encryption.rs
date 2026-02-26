use alloc::vec::Vec;

use argon2::Argon2;
use chacha20poly1305::ChaCha20Poly1305;
use chacha20poly1305::aead::{Aead, KeyInit};
use zeroize::Zeroizing;

use super::KeyStoreError;

// CONSTANTS
// ================================================================================================

/// Magic bytes at the start of every encrypted key file.
const ENCRYPTED_MAGIC: &[u8; 4] = b"MENC";

/// Current encrypted file format version.
const ENCRYPTED_VERSION: u8 = 0x01;

/// Salt length for Argon2id key derivation (16 bytes).
const SALT_LEN: usize = 16;

/// Nonce length for ChaCha20-Poly1305 (12 bytes).
const NONCE_LEN: usize = 12;

/// Length of the fixed-size header: magic (4) + version (1) + salt (16) + nonce (12) = 33 bytes.
const HEADER_LEN: usize = 4 + 1 + SALT_LEN + NONCE_LEN;

/// Argon2id memory cost in KiB (19 MiB — OWASP recommendation).
const ARGON2_M_COST: u32 = 19 * 1024;

/// Argon2id time cost (iterations).
const ARGON2_T_COST: u32 = 2;

/// Argon2id parallelism.
const ARGON2_P_COST: u32 = 1;

/// Derived key length for ChaCha20-Poly1305 (32 bytes).
const KEY_LEN: usize = 32;

// KEY ENCRYPTOR TRAIT
// ================================================================================================

/// Trait for encrypting and decrypting secret key bytes.
///
/// Implementations are used by [`FilesystemKeyStore`](super::FilesystemKeyStore) to encrypt keys
/// at rest. The trait is object-safe so it can be stored as `Arc<dyn KeyEncryptor>`.
pub trait KeyEncryptor: Send + Sync {
    /// Encrypts plaintext key bytes, returning the ciphertext (including any headers/metadata
    /// needed for decryption).
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, KeyStoreError>;

    /// Decrypts ciphertext previously produced by [`encrypt`](Self::encrypt), returning the
    /// original plaintext key bytes.
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, KeyStoreError>;
}

// PASSWORD ENCRYPTOR
// ================================================================================================

/// Password-based key encryptor using Argon2id for key derivation and ChaCha20-Poly1305 for
/// authenticated encryption.
///
/// Each call to [`encrypt`](KeyEncryptor::encrypt) generates a unique random salt and nonce, so
/// the same plaintext encrypted twice will produce different ciphertexts.
///
/// ## Encrypted file format
///
/// ```text
/// [4B: "MENC"] [1B: version=0x01] [16B: salt] [12B: nonce] [NB: ciphertext + 16B auth tag]
/// ```
pub struct PasswordEncryptor {
    password: Zeroizing<Vec<u8>>,
}

impl PasswordEncryptor {
    /// Creates a new `PasswordEncryptor` from a password.
    ///
    /// The password is stored in a [`Zeroizing`] wrapper that clears memory on drop.
    pub fn new(password: impl Into<Vec<u8>>) -> Self {
        Self {
            password: Zeroizing::new(password.into()),
        }
    }

    /// Derives a 256-bit key from the password and salt using Argon2id.
    ///
    /// The derived key is wrapped in [`Zeroizing`] so it is cleared from memory after use.
    fn derive_key(&self, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_LEN]>, KeyStoreError> {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(KEY_LEN))
                .map_err(|e| KeyStoreError::StorageError(format!("argon2 params error: {e}")))?,
        );

        let mut key = Zeroizing::new([0u8; KEY_LEN]);
        argon2
            .hash_password_into(self.password.as_slice(), salt, &mut *key)
            .map_err(|e| KeyStoreError::StorageError(format!("key derivation error: {e}")))?;

        Ok(key)
    }
}

impl KeyEncryptor for PasswordEncryptor {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, KeyStoreError> {
        // Generate random salt and nonce
        let salt: [u8; SALT_LEN] = rand::random();
        let nonce: [u8; NONCE_LEN] = rand::random();
        let nonce = chacha20poly1305::Nonce::from(nonce);

        // Derive encryption key (zeroized on drop)
        let key = self.derive_key(&salt)?;
        let cipher = ChaCha20Poly1305::new_from_slice(&*key)
            .map_err(|e| KeyStoreError::StorageError(format!("cipher init error: {e}")))?;

        // Encrypt
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| KeyStoreError::StorageError(format!("encryption error: {e}")))?;

        // Assemble: magic + version + salt + nonce + ciphertext
        let mut output = Vec::with_capacity(HEADER_LEN + ciphertext.len());
        output.extend_from_slice(ENCRYPTED_MAGIC);
        output.push(ENCRYPTED_VERSION);
        output.extend_from_slice(&salt);
        output.extend_from_slice(&nonce);
        output.extend_from_slice(&ciphertext);

        Ok(output)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, KeyStoreError> {
        if data.len() < HEADER_LEN {
            return Err(KeyStoreError::DecodingError("encrypted file too short".into()));
        }

        // Validate magic
        if &data[..4] != ENCRYPTED_MAGIC {
            return Err(KeyStoreError::DecodingError("invalid encrypted file magic".into()));
        }

        // Validate version
        let version = data[4];
        if version != ENCRYPTED_VERSION {
            return Err(KeyStoreError::DecodingError(format!(
                "unsupported encrypted file version: {version}"
            )));
        }

        // Extract salt, nonce, ciphertext
        let salt = &data[5..5 + SALT_LEN];
        let nonce_bytes = &data[5 + SALT_LEN..5 + SALT_LEN + NONCE_LEN];
        let ciphertext = &data[HEADER_LEN..];

        let nonce = chacha20poly1305::Nonce::from_slice(nonce_bytes);

        // Derive key and decrypt (key is zeroized on drop)
        let key = self.derive_key(salt)?;
        let cipher = ChaCha20Poly1305::new_from_slice(&*key)
            .map_err(|e| KeyStoreError::StorageError(format!("cipher init error: {e}")))?;

        cipher.decrypt(nonce, ciphertext).map_err(|_| {
            KeyStoreError::DecodingError(
                "decryption failed: wrong password or corrupted data".into(),
            )
        })
    }
}

/// Returns `true` if `data` starts with the encrypted file magic header (`MENC`).
///
/// # Safety assumption
///
/// `AuthSecretKey::to_bytes()` starts with a variant discriminant (`0x00` or `0x01`), so
/// plaintext key files will never start with `b"MENC"` (`[0x4D, 0x45, 0x4E, 0x43]`). This
/// makes magic-based detection reliable for distinguishing encrypted from plaintext files.
pub(crate) fn is_encrypted(data: &[u8]) -> bool {
    data.len() >= 4 && &data[..4] == ENCRYPTED_MAGIC
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let encryptor = PasswordEncryptor::new("test-password");
        let plaintext = b"secret key data here";

        let encrypted = encryptor.encrypt(plaintext).unwrap();
        assert!(is_encrypted(&encrypted));

        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_password_fails() {
        let encryptor = PasswordEncryptor::new("correct-password");
        let plaintext = b"secret key data";

        let encrypted = encryptor.encrypt(plaintext).unwrap();

        let wrong_encryptor = PasswordEncryptor::new("wrong-password");
        let result = wrong_encryptor.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn unique_ciphertexts() {
        let encryptor = PasswordEncryptor::new("password");
        let plaintext = b"same data";

        let encrypted1 = encryptor.encrypt(plaintext).unwrap();
        let encrypted2 = encryptor.encrypt(plaintext).unwrap();

        // Each encryption should produce different output (different salt + nonce)
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same plaintext
        assert_eq!(encryptor.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(encryptor.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn truncated_data_fails() {
        let encryptor = PasswordEncryptor::new("password");
        let plaintext = b"data";

        let encrypted = encryptor.encrypt(plaintext).unwrap();

        // Truncate to less than header length
        let result = encryptor.decrypt(&encrypted[..HEADER_LEN - 1]);
        assert!(result.is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let encryptor = PasswordEncryptor::new("password");
        let plaintext = b"important data";

        let mut encrypted = encryptor.encrypt(plaintext).unwrap();

        // Flip a bit in the ciphertext portion
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0x01;

        let result = encryptor.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn plaintext_not_detected_as_encrypted() {
        // AuthSecretKey discriminants are 0x00 or 0x01, never starts with "MENC"
        assert!(!is_encrypted(&[0x00, 0x01, 0x02, 0x03]));
        assert!(!is_encrypted(&[0x01, 0x00, 0x00, 0x00]));
        assert!(!is_encrypted(&[]));
        assert!(!is_encrypted(&[0x4d])); // just 'M'
    }

    #[test]
    fn encrypted_detected_correctly() {
        assert!(is_encrypted(b"MENC\x01some_data_here"));
    }
}
