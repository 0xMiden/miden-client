use miden_client::account::Account as NativeAccount;
use miden_client::keystore;
use wasm_bindgen::prelude::*;

use crate::models::account::Account;
use crate::models::account_header::AccountHeader;
use crate::models::account_id::AccountId;
use crate::models::address::Address;
use crate::models::auth_secret_key::AuthSecretKey;
use crate::models::public_key::PublicKey;
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

    #[wasm_bindgen(js_name = "getAccountAuthByPubKey")]
    pub async fn get_account_secret_key_by_pub_key(
        &mut self,
        pub_key: &PublicKey,
    ) -> Result<AuthSecretKey, JsValue> {
        let keystore = self.keystore.clone().expect("Keystore not initialized");

        let auth_secret_key = keystore
            .get_key(pub_key.0.to_commitment())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get public key for account"))?
            .ok_or(JsValue::from_str("Auth not found for account"))?;

        Ok(auth_secret_key.into())
    }

    #[wasm_bindgen(js_name = "getPublicKeysOfAccount")]
    pub async fn get_public_keys_of(
        &mut self,
        account_id: &AccountId,
    ) -> Result<AuthSecretKey, JsValue> {
        let keystore = self.keystore.clone().ok_or(JsValue::from("keystore is not initialized"))?;

        Ok(todo!())

        // Ok(keystore
        //     .get_public_commitments_for_account(account_id.as_native())
        //     .await
        //     .unwrap())
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
}
