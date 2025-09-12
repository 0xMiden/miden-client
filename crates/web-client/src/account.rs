use miden_client::store::AccountRecord;
use miden_objects::account::{Account as NativeAccount, AuthSecretKey as NativeAuthSecretKey};
use wasm_bindgen::prelude::*;

use crate::models::account::Account;
use crate::models::account_header::AccountHeader;
use crate::models::account_id::AccountId;
use crate::models::secret_key::SecretKey;
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
            let account: Option<NativeAccount> = result.map(AccountRecord::into);

            Ok(account.map(miden_client::account::Account::into))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "getAccountAuthByPubKey")]
    pub async fn get_account_secret_key_by_pub_key(
        &mut self,
        pub_key: &Word,
    ) -> Result<SecretKey, JsValue> {
        let keystore = self.keystore.clone().expect("Keystore not initialized");

        let auth_secret_key = keystore
            .get_key(pub_key.into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get public key for account"))?
            .ok_or(JsValue::from_str("Auth not found for account"))?;
        let NativeAuthSecretKey::RpoFalcon512(secret_key) = auth_secret_key;

        Ok(secret_key.into())
    }
}
