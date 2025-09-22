use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;

use idxdb_store::auth::{get_account_auth_by_pub_key, insert_account_auth};
use miden_client::auth::{AuthSecretKey, SigningInputs, TransactionAuthenticator};
use miden_client::keystore::KeyStoreError;
use miden_client::utils::RwLock;
use miden_client::{AuthenticationError, Felt, Word};
use miden_lib::utils::{Deserializable, Serializable};
use rand::Rng;

/// A web-based keystore that stores keys in [browser's local storage](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API)
/// and provides transaction authentication functionality.
#[derive(Clone)]
pub struct WebKeyStore<R: Rng> {
    /// The random number generator used to generate signatures.
    rng: Arc<RwLock<R>>,
}

impl<R: Rng> WebKeyStore<R> {
    /// Creates a new instance of the web keystore with the provided RNG.
    pub fn new(rng: R) -> Self {
        WebKeyStore { rng: Arc::new(RwLock::new(rng)) }
    }

    pub async fn add_key(&self, key: &AuthSecretKey) -> Result<(), KeyStoreError> {
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
        let message = signing_inputs.to_commitment();

        let secret_key = self
            .get_key(pub_key)
            .await
            .map_err(|err| AuthenticationError::other(err.to_string()))?;

        let mut rng = self.rng.write();

        let AuthSecretKey::RpoFalcon512(k) =
            secret_key.ok_or(AuthenticationError::UnknownPublicKey(pub_key.to_hex()))?;
        miden_client::auth::get_falcon_signature(&k, message, &mut *rng)
    }
}
