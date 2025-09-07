use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_lib::utils::{Deserializable, Serializable};
use miden_tx::auth::SigningInputs;
use rand::Rng;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::js_sys::{Array, Function, Promise};

use super::KeyStoreError;
use crate::auth::{AuthSecretKey, TransactionAuthenticator};
use crate::store::web_store::account::utils::{get_account_auth_by_pub_key, insert_account_auth};
use crate::utils::RwLock;
use crate::{AuthenticationError, Felt, Word};

/// A web-based keystore that stores keys in [browser's local storage](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API)
/// and provides transaction authentication functionality.
#[derive(Clone)]
pub struct WebKeyStore<R: Rng> {
    /// The random number generator used to generate signatures.
    rng: Arc<RwLock<R>>,
    /// Optional JavaScript callbacks for remote key storage and signing.
    callbacks: Option<Arc<JsCallbacks>>,
}

struct JsCallbacks {
    get_key_cb: Option<Function>,
    insert_key_cb: Option<Function>,
    sign_cb: Option<Function>,
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
            callbacks: None,
        }
    }

    /// Creates a new instance with optional JavaScript callbacks.
    /// When provided, these callbacks override the default IndexedDB storage and local signing.
    pub fn new_with_callbacks(
        rng: R,
        get_key_cb: Option<Function>,
        insert_key_cb: Option<Function>,
        sign_cb: Option<Function>,
    ) -> Self {
        WebKeyStore {
            rng: Arc::new(RwLock::new(rng)),
            callbacks: Some(Arc::new(JsCallbacks { get_key_cb, insert_key_cb, sign_cb })),
        }
    }

    pub async fn add_key(&self, key: &AuthSecretKey) -> Result<(), KeyStoreError> {
        if let Some(callbacks) = &self.callbacks {
            if let Some(insert_key_cb) = &callbacks.insert_key_cb {
                let pub_key = match &key {
                    AuthSecretKey::RpoFalcon512(k) => Word::from(k.public_key()).to_hex(),
                };
                let secret_key_hex = hex::encode(key.to_bytes());

                let result = insert_key_cb
                    .call2(&JsValue::NULL, &JsValue::from(pub_key), &JsValue::from(secret_key_hex))
                    .map_err(|err| {
                        KeyStoreError::StorageError(format!("JS insertKey threw: {err:?}"))
                    })?;

                if let Some(promise) = result.dyn_ref::<Promise>() {
                    JsFuture::from(promise.clone()).await.map_err(|_| {
                        KeyStoreError::StorageError("Failed to insert key via callback".to_string())
                    })?;
                }

                return Ok(());
            }
        }

        let pub_key = match &key {
            AuthSecretKey::RpoFalcon512(k) => Word::from(k.public_key()).to_hex(),
        };
        let secret_key_hex = hex::encode(key.to_bytes());

        insert_account_auth(pub_key, secret_key_hex).await.map_err(|_| {
            KeyStoreError::StorageError("Failed to insert item into local storage".to_string())
        })?;

        Ok(())
    }

    pub async fn get_key(&self, pub_key: Word) -> Result<Option<AuthSecretKey>, KeyStoreError> {
        if let Some(callbacks) = &self.callbacks {
            if let Some(get_key_cb) = &callbacks.get_key_cb {
                let pub_key_str = pub_key.to_hex();
                let call_result =
                    get_key_cb.call1(&JsValue::NULL, &JsValue::from(pub_key_str)).map_err(
                        |err| KeyStoreError::StorageError(format!("JS getKey threw: {err:?}")),
                    )?;

                let resolved = if let Some(promise) = call_result.dyn_ref::<Promise>() {
                    JsFuture::from(promise.clone()).await.map_err(|_| {
                        KeyStoreError::StorageError("Failed to get key via callback".to_string())
                    })?
                } else {
                    call_result
                };

                if resolved.is_null() || resolved.is_undefined() {
                    return Ok(None);
                }

                let secret_key_hex = resolved.as_string().ok_or_else(|| {
                    KeyStoreError::DecodingError("Expected secret key hex string".to_string())
                })?;

                let secret_key_bytes = hex::decode(secret_key_hex).map_err(|err| {
                    KeyStoreError::DecodingError(format!("error decoding secret key hex: {err:?}"))
                })?;

                let secret_key =
                    AuthSecretKey::read_from_bytes(&secret_key_bytes).map_err(|err| {
                        KeyStoreError::DecodingError(format!("error reading secret key: {err:?}"))
                    })?;

                return Ok(Some(secret_key));
            }
        }

        let pub_key_str = pub_key.to_hex();
        let secret_key_hex = get_account_auth_by_pub_key(pub_key_str).await.map_err(|err| {
            KeyStoreError::StorageError(format!("failed to get item from local storage: {err:?}"))
        })?;

        let secret_key_bytes = hex::decode(secret_key_hex).map_err(|err| {
            KeyStoreError::DecodingError(format!("error decoding secret key hex: {err:?}"))
        })?;

        let secret_key = AuthSecretKey::read_from_bytes(&secret_key_bytes).map_err(|err| {
            KeyStoreError::DecodingError(format!("error reading secret key: {err:?}"))
        })?;

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
        pub_key: Word,
        signing_inputs: &SigningInputs,
    ) -> Result<Vec<Felt>, AuthenticationError> {
        // If a JavaScript signing callback is provided, use it directly.
        if let Some(callbacks) = &self.callbacks {
            if let Some(sign_cb) = &callbacks.sign_cb {
                let commitment_hex = signing_inputs.to_commitment().to_hex();
                let pub_key_hex = pub_key.to_hex();

                let call_result = sign_cb
                    .call2(
                        &JsValue::NULL,
                        &JsValue::from(pub_key_hex),
                        &JsValue::from(commitment_hex),
                    )
                    .map_err(|err| AuthenticationError::other(format!("JS sign threw: {err:?}")))?;

                let resolved = if let Some(promise) = call_result.dyn_ref::<Promise>() {
                    JsFuture::from(promise.clone()).await.map_err(|err| {
                        AuthenticationError::other(format!("Failed to sign via callback: {err:?}"))
                    })?
                } else {
                    call_result
                };

                let arr = Array::is_array(&resolved).then(|| Array::from(&resolved)).ok_or_else(
                    || AuthenticationError::other("sign callback must return an array"),
                )?;

                let mut result: Vec<Felt> = Vec::with_capacity(arr.length() as usize);
                for value in arr.iter() {
                    if let Some(s) = value.as_string() {
                        let n = s.parse::<u64>().map_err(|_| {
                            AuthenticationError::other(
                                "failed to parse signature element string as u64",
                            )
                        })?;
                        result.push(Felt::new(n));
                    } else if let Some(f) = value.as_f64() {
                        result.push(Felt::new(f as u64));
                    } else {
                        return Err(AuthenticationError::other(
                            "signature elements must be numbers or numeric strings",
                        ));
                    }
                }

                return Ok(result);
            }
        }

        let message = signing_inputs.to_commitment();

        let secret_key = self
            .get_key(pub_key)
            .await
            .map_err(|err| AuthenticationError::other(err.to_string()))?;

        let mut rng = self.rng.write();

        let AuthSecretKey::RpoFalcon512(k) =
            secret_key.ok_or(AuthenticationError::UnknownPublicKey(pub_key.to_hex()))?;
        miden_tx::auth::signatures::get_falcon_signature(&k, message, &mut *rng)
    }
}
