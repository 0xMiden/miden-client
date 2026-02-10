use miden_client::Word as NativeWord;
use wasm_bindgen::prelude::*;

use crate::models::account::Account;
use crate::models::account_code::AccountCode;
use crate::models::account_header::AccountHeader;
use crate::models::account_id::AccountId;
use crate::models::account_reader::AccountReader;
use crate::models::account_storage::AccountStorage;
use crate::models::address::Address;
use crate::models::asset_vault::AssetVault;
use crate::models::auth_secret_key::AuthSecretKey;
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

    /// Retrieves the full account data for the given account ID, returning `null` if not found.
    ///
    /// This method loads the complete account state including vault, storage, and code.
    #[wasm_bindgen(js_name = "getAccount")]
    pub async fn get_account(
        &mut self,
        account_id: &AccountId,
    ) -> Result<Option<Account>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            client
                .get_account(account_id.into())
                .await
                .map(|opt| opt.map(Into::into))
                .map_err(|err| js_error_with_context(err, "failed to get account"))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    /// Retrieves the asset vault for a specific account.
    ///
    /// To check the balance for a single asset, use `accountReader` instead.
    #[wasm_bindgen(js_name = "getAccountVault")]
    pub async fn get_account_vault(
        &mut self,
        account_id: &AccountId,
    ) -> Result<AssetVault, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            client
                .get_account_vault(account_id.into())
                .await
                .map(Into::into)
                .map_err(|err| js_error_with_context(err, "failed to get account vault"))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    /// Retrieves the storage for a specific account.
    ///
    /// To only load a specific slot, use `accountReader` instead.
    #[wasm_bindgen(js_name = "getAccountStorage")]
    pub async fn get_account_storage(
        &mut self,
        account_id: &AccountId,
    ) -> Result<AccountStorage, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            client
                .get_account_storage(account_id.into())
                .await
                .map(Into::into)
                .map_err(|err| js_error_with_context(err, "failed to get account storage"))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    /// Retrieves the account code for a specific account.
    ///
    /// Returns `null` if the account is not found.
    #[wasm_bindgen(js_name = "getAccountCode")]
    pub async fn get_account_code(
        &mut self,
        account_id: &AccountId,
    ) -> Result<Option<AccountCode>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            client
                .get_account_code(account_id.into())
                .await
                .map(|opt| opt.map(Into::into))
                .map_err(|err| js_error_with_context(err, "failed to get account code"))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    /// Creates a new `AccountReader` for lazy access to account data.
    ///
    /// The `AccountReader` executes queries lazily - each method call fetches fresh data
    /// from storage, ensuring you always see the current state.
    ///
    /// # Arguments
    /// * `account_id` - The ID of the account to read.
    ///
    /// # Example
    /// ```javascript
    /// const reader = client.accountReader(accountId);
    /// const nonce = await reader.nonce();
    /// const balance = await reader.getBalance(faucetId);
    /// ```
    #[wasm_bindgen(js_name = "accountReader")]
    pub fn account_reader(&self, account_id: &AccountId) -> Result<AccountReader, JsValue> {
        let store = self.store.clone().ok_or(JsValue::from_str("Store not initialized"))?;
        Ok(AccountReader::new(store, account_id.into()))
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
            .get_secret_key((*pub_key_commitment.as_native()).into())
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
}
