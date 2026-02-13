use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::sync::Arc;

use idxdb_store::auth::{
    get_account_auth_by_pub_key_commitment,
    get_key_commitments_by_account_id,
    insert_account_auth,
    insert_account_key_mapping,
    remove_account_auth,
    remove_all_mappings_for_key,
};
use miden_client::account::AccountId;
use miden_client::auth::{
    AuthSecretKey,
    PublicKey,
    PublicKeyCommitment,
    Signature,
    SigningInputs,
    TransactionAuthenticator,
};
use miden_client::keystore::{KeyStoreError, Keystore};
use miden_client::utils::{RwLock, Serializable};
use miden_client::{AuthenticationError, Word as NativeWord};
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
    /// The database ID for `IndexedDB` operations.
    db_id: String,
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
    pub fn new(rng: R, db_id: String) -> Self {
        WebKeyStore {
            rng: Arc::new(RwLock::new(rng)),
            callbacks: Arc::new(JsCallbacks {
                get_key: None,
                insert_key: None,
                sign: None,
            }),
            db_id,
        }
    }

    /// Creates a new instance with optional JavaScript callbacks.
    /// When provided, these callbacks override the default `IndexedDB` storage and local signing.
    pub fn new_with_callbacks(
        rng: R,
        db_id: String,
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
            db_id,
        }
    }

    /// Adds a secret key to the keystore without updating account mappings.
    ///
    /// This is an internal method. Use [`Keystore::add_key`] instead.
    async fn add_key_without_account(&self, key: &AuthSecretKey) -> Result<(), KeyStoreError> {
        if let Some(insert_key_cb) = &self.callbacks.as_ref().insert_key {
            let sk = WebAuthSecretKey::from(key.clone());
            insert_key_cb.insert_key(&sk).await?;
            return Ok(());
        }

        let pub_key_commitment = NativeWord::from(key.public_key().to_commitment()).to_hex();
        let secret_key_hex = hex::encode(key.to_bytes());

        insert_account_auth(&self.db_id, pub_key_commitment, secret_key_hex)
            .await
            .map_err(|_| {
                KeyStoreError::StorageError("Failed to insert item into IndexedDB".to_string())
            })?;

        Ok(())
    }
}

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
            .get_key(pub_key)
            .await
            .map_err(|err| AuthenticationError::other(err.to_string()))?;

        let mut rng = self.rng.write();

        let signature = match secret_key {
            Some(AuthSecretKey::Falcon512Rpo(k)) => {
                Signature::Falcon512Rpo(k.sign_with_rng(message, &mut rng))
            },
            Some(AuthSecretKey::EcdsaK256Keccak(k)) => Signature::EcdsaK256Keccak(k.sign(message)),
            Some(other_k) => other_k.sign(message),
            None => return Err(AuthenticationError::UnknownPublicKey(pub_key)),
        };

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

#[async_trait::async_trait(?Send)]
impl<R: Rng> Keystore for WebKeyStore<R> {
    async fn add_key(
        &self,
        key: &AuthSecretKey,
        account_id: AccountId,
    ) -> Result<(), KeyStoreError> {
        let pub_key_commitment = key.public_key().to_commitment();

        // Store the key
        self.add_key_without_account(key).await?;

        // Store the mapping
        let account_id_hex = account_id.to_hex();
        let pub_key_hex = NativeWord::from(pub_key_commitment).to_hex();

        insert_account_key_mapping(&self.db_id, account_id_hex, pub_key_hex)
            .await
            .map_err(|_| {
                KeyStoreError::StorageError(
                    "Failed to insert account key mapping into IndexedDB".to_string(),
                )
            })?;

        Ok(())
    }

    async fn remove_key(&self, pub_key: PublicKeyCommitment) -> Result<(), KeyStoreError> {
        let pub_key_hex = NativeWord::from(pub_key).to_hex();

        // Remove all account-key mappings for this key
        remove_all_mappings_for_key(&self.db_id, pub_key_hex.clone())
            .await
            .map_err(|_| {
                KeyStoreError::StorageError(
                    "Failed to remove account key mappings from IndexedDB".to_string(),
                )
            })?;

        // Remove the key itself
        remove_account_auth(&self.db_id, pub_key_hex).await.map_err(|_| {
            KeyStoreError::StorageError("Failed to remove key from IndexedDB".to_string())
        })?;

        Ok(())
    }

    /// Retrieves a secret key from the keystore given the commitment of a public key.
    async fn get_key(
        &self,
        pub_key: PublicKeyCommitment,
    ) -> Result<Option<AuthSecretKey>, KeyStoreError> {
        if let Some(get_key_cb) = &self.callbacks.as_ref().get_key {
            return get_key_cb.get_secret_key(pub_key).await;
        }
        let pub_key_commitment = NativeWord::from(pub_key).to_hex();
        let secret_key_hex =
            get_account_auth_by_pub_key_commitment(&self.db_id, pub_key_commitment)
                .await
                .map_err(|_| {
                    KeyStoreError::StorageError(
                        "Failed to get secret key from IndexedDB".to_string(),
                    )
                })?;

        let Some(secret_key_hex) = secret_key_hex else {
            return Ok(None);
        };

        let secret_key_bytes = hex::decode(secret_key_hex).map_err(|err| {
            KeyStoreError::DecodingError(format!("error decoding secret key hex: {err:?}"))
        })?;

        let secret_key = decode_secret_key_from_bytes(&secret_key_bytes)?;
        Ok(Some(secret_key))
    }

    async fn get_account_key_commitments(
        &self,
        account_id: &AccountId,
    ) -> Result<BTreeSet<PublicKeyCommitment>, KeyStoreError> {
        let account_id_hex = account_id.to_hex();

        let commitment_hexes = get_key_commitments_by_account_id(&self.db_id, account_id_hex)
            .await
            .map_err(|_| {
                KeyStoreError::StorageError(
                    "Failed to get key commitments from IndexedDB".to_string(),
                )
            })?;

        let commitments = commitment_hexes
            .into_iter()
            .filter_map(|hex| {
                NativeWord::try_from(hex.as_str()).ok().map(PublicKeyCommitment::from)
            })
            .collect();

        Ok(commitments)
    }
}
