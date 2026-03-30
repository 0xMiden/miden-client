use alloc::sync::Arc;

use miden_client::Word as NativeWord;
use miden_client::keystore::Keystore;
use wasm_bindgen::prelude::*;

use crate::models::account_id::AccountId;
use crate::models::auth_secret_key::AuthSecretKey;
use crate::models::word::Word;
use crate::{ClientAuth, js_error_with_context};

/// A handle to the keystore, exposed to JavaScript as `client.keystore`.
///
/// Provides methods for managing secret keys: inserting, retrieving, removing,
/// and querying key-account mappings.
#[wasm_bindgen]
pub struct WebKeystoreApi {
    keystore: Arc<ClientAuth>,
}

impl WebKeystoreApi {
    pub(crate) fn new(keystore: Arc<ClientAuth>) -> Self {
        Self { keystore }
    }
}

#[wasm_bindgen]
impl WebKeystoreApi {
    /// Inserts a secret key into the keystore, associating it with the given account ID.
    #[wasm_bindgen]
    pub async fn insert(
        &self,
        account_id: &AccountId,
        secret_key: &AuthSecretKey,
    ) -> Result<(), JsValue> {
        let native_secret_key: miden_client::auth::AuthSecretKey = secret_key.into();
        let native_account_id = account_id.into();

        self.keystore
            .add_key(&native_secret_key, native_account_id)
            .await
            .map_err(|err| js_error_with_context(err, "failed to insert key into keystore"))
    }

    /// Retrieves a secret key from the keystore given a public key commitment.
    ///
    /// Returns the associated `AuthSecretKey` if found, or `null` if not found.
    #[wasm_bindgen]
    pub async fn get(
        &self,
        pub_key_commitment: &Word,
    ) -> Result<Option<AuthSecretKey>, JsValue> {
        let key = self
            .keystore
            .get_key((*pub_key_commitment.as_native()).into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get key from keystore"))?;

        Ok(key.map(Into::into))
    }

    /// Removes a key from the keystore by its public key commitment.
    #[wasm_bindgen]
    pub async fn remove(
        &self,
        pub_key_commitment: &Word,
    ) -> Result<(), JsValue> {
        self.keystore
            .remove_key((*pub_key_commitment.as_native()).into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to remove key from keystore"))
    }

    /// Returns all public key commitments associated with the given account ID.
    #[wasm_bindgen(js_name = "getCommitments")]
    pub async fn get_commitments(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<Word>, JsValue> {
        Ok(self
            .keystore
            .get_account_key_commitments(account_id.as_native())
            .await
            .map_err(|err| {
                js_error_with_context(
                    err,
                    &format!(
                        "failed to get key commitments for account: {}",
                        account_id.as_native()
                    ),
                )
            })?
            .into_iter()
            .map(NativeWord::from)
            .map(Into::into)
            .collect())
    }

    /// Returns the account ID associated with a given public key commitment,
    /// or `null` if no account is found.
    #[wasm_bindgen(js_name = "getAccountId")]
    pub async fn get_account_id(
        &self,
        pub_key_commitment: &Word,
    ) -> Result<Option<AccountId>, JsValue> {
        let account_id = self
            .keystore
            .get_account_id_by_key_commitment((*pub_key_commitment.as_native()).into())
            .await
            .map_err(|err| {
                js_error_with_context(err, "failed to get account by key commitment")
            })?;

        Ok(account_id.map(Into::into))
    }
}
