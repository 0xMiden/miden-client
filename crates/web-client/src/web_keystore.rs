use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use idxdb_store::auth::{get_account_auth_by_pub_key, insert_account_auth};
use idxdb_store::encryption::{get_encryption_key, insert_encryption_key};
use miden_client::account::Address;
use miden_client::auth::{
    AuthSecretKey,
    PublicKey,
    PublicKeyCommitment,
    Signature,
    SigningInputs,
    TransactionAuthenticator,
};
use miden_client::keystore::{EncryptionKeyStore, KeyStoreError};
use miden_client::utils::{RwLock, Serializable};
use miden_client::{AuthenticationError, Deserializable, Word, Word as NativeWord};
use miden_objects::crypto::dsa::ecdsa_k256_keccak::SecretKey as K256SecretKey;
use miden_objects::crypto::dsa::eddsa_25519::SecretKey as X25519SecretKey;
use miden_objects::crypto::ies::UnsealingKey;
use rand::Rng;
use wasm_bindgen_futures::js_sys::Function;

use crate::models::auth_secret_key::AuthSecretKey as WebAuthSecretKey;
use crate::web_keystore_callbacks::{
    GetKeyCallback,
    InsertKeyCallback,
    SignCallback,
    decode_secret_key_from_bytes,
};

/// A web-based keystore that stores keys in [browser's local storage](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API)
/// and provides transaction authentication functionality.
#[derive(Clone)]
pub struct WebKeyStore<R: Rng> {
    /// The random number generator used to generate signatures.
    rng: Arc<RwLock<R>>,
    callbacks: Arc<JsCallbacks>,
}

struct JsCallbacks {
    get_key: Option<GetKeyCallback>,
    insert_key: Option<InsertKeyCallback>,
    sign: Option<SignCallback>,
}

// Since Function is not Send/Sync, we need to explicitly mark our struct as Send + Sync
// This is safe in WASM because it's single-threaded
unsafe impl Send for JsCallbacks {}
unsafe impl Sync for JsCallbacks {}

impl<R: Rng> WebKeyStore<R> {
    /// Creates a new instance of the web keystore with the provided RNG.
    pub fn new(rng: R) -> Self {
        WebKeyStore {
            rng: Arc::new(RwLock::new(rng)),
            callbacks: Arc::new(JsCallbacks {
                get_key: None,
                insert_key: None,
                sign: None,
            }),
        }
    }

    /// Creates a new instance with optional JavaScript callbacks.
    /// When provided, these callbacks override the default `IndexedDB` storage and local signing.
    pub fn new_with_callbacks(
        rng: R,
        get_key: Option<Function>,
        insert_key: Option<Function>,
        sign: Option<Function>,
    ) -> Self {
        WebKeyStore {
            rng: Arc::new(RwLock::new(rng)),
            callbacks: Arc::new(JsCallbacks {
                get_key: get_key.map(GetKeyCallback),
                insert_key: insert_key.map(InsertKeyCallback),
                sign: sign.map(SignCallback),
            }),
        }
    }

    pub async fn add_key(&self, key: &AuthSecretKey) -> Result<(), KeyStoreError> {
        if let Some(insert_key_cb) = &self.callbacks.as_ref().insert_key {
            let sk = WebAuthSecretKey::from(key.clone());
            insert_key_cb.insert_key(&sk).await?;
            return Ok(());
        }
        let pub_key = match key {
            AuthSecretKey::RpoFalcon512(k) => k.public_key().to_commitment().to_hex(),
            AuthSecretKey::EcdsaK256Keccak(k) => k.public_key().to_commitment().to_hex(),
            other => {
                let commitment: Word = other.public_key().to_commitment().into();
                commitment.to_hex()
            },
        };
        let secret_key_hex = hex::encode(key.to_bytes());

        insert_account_auth(pub_key, secret_key_hex).await.map_err(|_| {
            KeyStoreError::StorageError("Failed to insert item into IndexedDB".to_string())
        })?;

        Ok(())
    }

    pub async fn get_key(
        &self,
        pub_key: NativeWord,
    ) -> Result<Option<AuthSecretKey>, KeyStoreError> {
        if let Some(get_key_cb) = &self.callbacks.as_ref().get_key {
            return get_key_cb.get_secret_key(pub_key).await;
        }
        let pub_key_str = pub_key.to_hex();
        let secret_key_hex = get_account_auth_by_pub_key(pub_key_str).await.map_err(|err| {
            KeyStoreError::StorageError(format!("failed to get item from IndexedDB: {err:?}"))
        })?;

        let secret_key_bytes = hex::decode(secret_key_hex).map_err(|err| {
            KeyStoreError::DecodingError(format!("error decoding secret key hex: {err:?}"))
        })?;

        let secret_key = decode_secret_key_from_bytes(&secret_key_bytes)?;

        Ok(Some(secret_key))
    }
}

// ENCRYPTION KEY STORE IMPLEMENTATION
// ================================================================================================

#[async_trait::async_trait(?Send)]
impl<R: Rng + Send + Sync> EncryptionKeyStore for WebKeyStore<R> {
    async fn add_encryption_key(
        &self,
        address: &Address,
        key: &UnsealingKey,
    ) -> Result<(), KeyStoreError> {
        let address_hash = hash_address(address);
        let key_bytes = serialize_unsealing_key(key);
        let key_hex = hex::encode(key_bytes);

        insert_encryption_key(address_hash, key_hex)
            .await
            .map_err(|e| KeyStoreError::StorageError(format!("{e:?}")))?;

        Ok(())
    }

    async fn get_encryption_key(
        &self,
        address: &Address,
    ) -> Result<Option<UnsealingKey>, KeyStoreError> {
        let address_hash = hash_address(address);

        let key_hex = get_encryption_key(address_hash)
            .await
            .map_err(|e| KeyStoreError::StorageError(format!("{e:?}")))?;

        match key_hex {
            Some(hex) => {
                let bytes =
                    hex::decode(hex).map_err(|e| KeyStoreError::DecodingError(format!("{e:?}")))?;
                let key = deserialize_unsealing_key(&bytes)?;
                Ok(Some(key))
            },
            None => Ok(None),
        }
    }
}

// TRANSACTION AUTHENTICATOR IMPLEMENTATION
// ================================================================================================

impl<R: Rng> TransactionAuthenticator for WebKeyStore<R> {
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
        signing_inputs: &SigningInputs,
    ) -> Result<Signature, AuthenticationError> {
        // If a JavaScript signing callback is provided, use it directly.
        if let Some(sign_cb) = &self.callbacks.as_ref().sign {
            return sign_cb.sign(pub_key.into(), signing_inputs).await;
        }
        let message = signing_inputs.to_commitment();

        let secret_key = self
            .get_key(pub_key.into())
            .await
            .map_err(|err| AuthenticationError::other(err.to_string()))?;

        let mut rng = self.rng.write();

        let signature = match secret_key {
            Some(AuthSecretKey::RpoFalcon512(k)) => {
                Signature::RpoFalcon512(k.sign_with_rng(message, &mut rng))
            },
            Some(AuthSecretKey::EcdsaK256Keccak(k)) => Signature::EcdsaK256Keccak(k.sign(message)),
            Some(other_k) => other_k.sign(message),
            None => return Err(AuthenticationError::UnknownPublicKey(pub_key)),
        };

        Ok(signature)
    }

    // TODO: add this (related to #1417)
    async fn get_public_key(&self, _pub_key_commitment: PublicKeyCommitment) -> Option<&PublicKey> {
        None
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Hashes an address to a string representation for use as a key.
fn hash_address(address: &Address) -> String {
    let address_bytes = address.to_bytes();
    let mut hasher = DefaultHasher::new();
    address_bytes.hash(&mut hasher);
    hasher.finish().to_string()
}

// UNSEALING KEY SERIALIZATION
// ================================================================================================

// IES scheme discriminants - must match miden-crypto's IesScheme enum order.
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
