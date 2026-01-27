use miden_client::Word as NativeWord;
use miden_client::account::Account as NativeAccount;
use miden_client::auth::PublicKey as NativePublicKey;
use wasm_bindgen::prelude::*;

use crate::models::account::Account;
use crate::models::account_header::AccountHeader;
use crate::models::account_id::AccountId;
use crate::models::address::Address;
use crate::models::auth_secret_key::AuthSecretKey;
use crate::models::public_key::PublicKey;
use crate::models::word::Word;
use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "getAccounts")]
    pub async fn get_accounts(&mut self) -> Result<Vec<AccountHeader>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let result = client
                .get_account_headers()
                .await
                .map_err(|err| js_error_with_context(err, "failed to get accounts"))?;

            Ok(result.into_iter().map(|(header, _)| header.into()).collect())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "getAccount")]
    pub async fn get_account(
        &mut self,
        account_id: &AccountId,
    ) -> Result<Option<Account>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let result = client
                .get_account(account_id.into())
                .await
                .map_err(|err| js_error_with_context(err, "failed to get account"))?;

            if let Some(account_record) = result {
                // TODO: add partial account support for web client
                let native_account: NativeAccount = account_record
                    .try_into()
                    .map_err(|_| JsValue::from_str("retrieval of partial account unsupported"))?;
                Ok(Some(native_account.into()))
            } else {
                Ok(None)
            }
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    /// Retrieves an authentication secret key from the keystore given a public key commitment.
    ///
    /// The public key commitment should correspond to one of the keys tracked by the keystore.
    /// Returns the associated [`AuthSecretKey`] if found, or an error if not found.
    #[wasm_bindgen(js_name = "getAccountAuthByPubKeyCommitment")]
    pub async fn get_account_auth_secret_key_by_pub_key_commitment(
        &mut self,
        pub_key_commitment: &Word,
    ) -> Result<AuthSecretKey, JsValue> {
        let keystore = self.keystore.clone().expect("Keystore not initialized");

        let auth_secret_key = keystore
            .get_key((*pub_key_commitment.as_native()).into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get auth key for account"))?
            .ok_or(JsValue::from_str("Auth not found for account"))?;

        Ok(auth_secret_key.into())
    }

    /// Returns all public key commitments associated with the given account ID.
    ///
    /// These commitments can be used with [`getAccountAuthByPubKeyCommitment`]
    /// to retrieve the corresponding secret keys from the keystore.
    #[wasm_bindgen(js_name = "getPublicKeyCommitmentsOfAccount")]
    pub async fn get_public_key_commitments_of(
        &mut self,
        account_id: &AccountId,
    ) -> Result<Vec<Word>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            Ok(client
                .get_account_public_key_commitments(account_id.as_native())
                .await
                .map_err(|err| {
                    js_error_with_context(
                        err,
                        &format!(
                            "failed to fetch public key commitments for account: {}",
                            account_id.as_native()
                        ),
                    )
                })?
                .into_iter()
                .map(NativeWord::from)
                .map(Into::into)
                .collect())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "insertAccountAddress")]
    pub async fn insert_account_address(
        &mut self,
        account_id: &AccountId,
        address: &Address,
    ) -> Result<(), JsValue> {
        if let Some(client) = self.get_mut_inner() {
            client
                .add_address(address.into(), account_id.into())
                .await
                .map_err(|err| js_error_with_context(err, "failed to add address to account"))?;
            Ok(())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "removeAccountAddress")]
    pub async fn remove_account_address(
        &mut self,
        account_id: &AccountId,
        address: &Address,
    ) -> Result<(), JsValue> {
        if let Some(client) = self.get_mut_inner() {
            client.remove_address(address.into(), account_id.into()).await.map_err(|err| {
                js_error_with_context(err, "failed to remove address from account")
            })?;
            Ok(())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "getAccountByPublicKey")]
    pub async fn get_account_by_public_key(
        &mut self,
        pub_key: &PublicKey,
    ) -> Result<Option<Account>, JsValue> {
        let store =
            self.store.as_ref().ok_or_else(|| JsValue::from_str("Store not initialized"))?;

        let native_pub_key: NativePublicKey = pub_key.into();
        let commitment = native_pub_key.to_commitment();

        let account_id = store
            .get_account_id_by_public_key(commitment)
            .await
            .map_err(|err| js_error_with_context(err, "failed to get account by public key"))?;

        match account_id {
            Some(id) => self.get_account(&id.into()).await,
            None => Ok(None),
        }
    }
}
