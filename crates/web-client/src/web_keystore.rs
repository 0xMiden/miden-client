use alloc::string::ToString;
use alloc::sync::Arc;

use idxdb_store::auth::{get_account_auth_by_pub_key, insert_account_auth};
use miden_client::auth::{
    AuthSecretKey,
    PublicKeyCommitment,
    Signature,
    SigningInputs,
    TransactionAuthenticator,
};
use miden_client::keystore::KeyStoreError;
use miden_client::utils::{RwLock, Serializable};
use miden_client::{AuthenticationError, Word as NativeWord};
use rand::Rng;
use wasm_bindgen_futures::js_sys::Function;

use crate::models::secret_key::SecretKey;
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
            let secret_key = match &key {
                AuthSecretKey::RpoFalcon512(k) => SecretKey::from(k.clone()),
            };
            insert_key_cb.insert_key(&secret_key).await?;
            return Ok(());
        }
        let pub_key = match &key {
            AuthSecretKey::RpoFalcon512(k) => k.public_key().to_commitment().to_hex(),
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
            let _prepared_signature = sign_cb.sign(pub_key.into(), signing_inputs).await?;
            todo!("can't create Signature from preparated Signature")
        }
        let message = signing_inputs.to_commitment();

        let secret_key = self
            .get_key(pub_key.into())
            .await
            .map_err(|err| AuthenticationError::other(err.to_string()))?;

        let mut rng = self.rng.write();

        let AuthSecretKey::RpoFalcon512(k) = secret_key.ok_or(
            AuthenticationError::UnknownPublicKey(Into::<NativeWord>::into(pub_key).to_hex()),
        )?;

        let signature = Signature::RpoFalcon512(k.sign_with_rng(message, &mut rng));
        Ok(signature)
    }
}
