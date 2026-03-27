use js_export_macro::js_export;
use miden_client::Word as NativeWord;
use miden_client::keystore::Keystore;

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
use crate::platform::{JsErr, from_str_err};
use crate::{WebClient, js_error_with_context};

#[js_export]
impl WebClient {
    #[js_export(js_name = "getAccounts")]
    pub async fn get_accounts(&self) -> Result<Vec<AccountHeader>, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let result = client
            .get_account_headers()
            .await
            .map_err(|err| js_error_with_context(err, "failed to get accounts"))?;

        Ok(result.into_iter().map(|(header, _)| header.into()).collect())
    }

    /// Retrieves the full account data for the given account ID, returning `null` if not found.
    ///
    /// This method loads the complete account state including vault, storage, and code.
    #[js_export(js_name = "getAccount")]
    pub async fn get_account(&self, account_id: &AccountId) -> Result<Option<Account>, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        client
            .get_account(account_id.into())
            .await
            .map(|opt| opt.map(Into::into))
            .map_err(|err| js_error_with_context(err, "failed to get account"))
    }

    /// Retrieves the asset vault for a specific account.
    ///
    /// To check the balance for a single asset, use `accountReader` instead.
    #[js_export(js_name = "getAccountVault")]
    pub async fn get_account_vault(&self, account_id: &AccountId) -> Result<AssetVault, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        client
            .get_account_vault(account_id.into())
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get account vault"))
    }

    /// Retrieves the storage for a specific account.
    ///
    /// To only load a specific slot, use `accountReader` instead.
    #[js_export(js_name = "getAccountStorage")]
    pub async fn get_account_storage(
        &self,
        account_id: &AccountId,
    ) -> Result<AccountStorage, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        client
            .get_account_storage(account_id.into())
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to get account storage"))
    }

    /// Retrieves the account code for a specific account.
    ///
    /// Returns `null` if the account is not found.
    #[js_export(js_name = "getAccountCode")]
    pub async fn get_account_code(
        &self,
        account_id: &AccountId,
    ) -> Result<Option<AccountCode>, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        client
            .get_account_code(account_id.into())
            .await
            .map(|opt| opt.map(Into::into))
            .map_err(|err| js_error_with_context(err, "failed to get account code"))
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
    #[js_export(js_name = "accountReader")]
    pub async fn account_reader(&self, account_id: &AccountId) -> Result<AccountReader, JsErr> {
        let guard = self.inner.lock().await;
        let client = guard.as_ref().ok_or_else(|| from_str_err("Client not initialized"))?;
        Ok(AccountReader::from(client.account_reader(account_id.into())))
    }

    /// Retrieves an authentication secret key from the keystore given a public key commitment.
    ///
    /// The public key commitment should correspond to one of the keys tracked by the keystore.
    /// Returns the associated [`AuthSecretKey`] if found, or an error if not found.
    #[js_export(js_name = "getAccountAuthByPubKeyCommitment")]
    pub async fn get_account_auth_secret_key_by_pub_key_commitment(
        &self,
        pub_key_commitment: &Word,
    ) -> Result<AuthSecretKey, JsErr> {
        let keystore = self.keystore().await?;

        let auth_secret_key = keystore
            .get_key((*pub_key_commitment.as_native()).into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get auth key for account"))?
            .ok_or(from_str_err("Auth not found for account"))?;

        Ok(auth_secret_key.into())
    }

    /// Returns all public key commitments associated with the given account ID.
    ///
    /// These commitments can be used with [`getAccountAuthByPubKeyCommitment`]
    /// to retrieve the corresponding secret keys from the keystore.
    #[js_export(js_name = "getPublicKeyCommitmentsOfAccount")]
    pub async fn get_public_key_commitments_of(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<Word>, JsErr> {
        let keystore = self.keystore().await?;
        Ok(keystore
            .get_account_key_commitments(account_id.as_native())
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
    }

    #[js_export(js_name = "insertAccountAddress")]
    pub async fn insert_account_address(
        &self,
        account_id: &AccountId,
        address: &Address,
    ) -> Result<(), JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        client
            .add_address(address.into(), account_id.into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to add address to account"))?;
        Ok(())
    }

    #[js_export(js_name = "removeAccountAddress")]
    pub async fn remove_account_address(
        &self,
        account_id: &AccountId,
        address: &Address,
    ) -> Result<(), JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        client
            .remove_address(address.into(), account_id.into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to remove address from account"))?;
        Ok(())
    }

    /// Retrieves the full account data for the account associated with the given public key
    /// commitment, returning `null` if no account is found.
    #[js_export(js_name = "getAccountByKeyCommitment")]
    pub async fn get_account_by_key_commitment(
        &self,
        pub_key_commitment: &Word,
    ) -> Result<Option<Account>, JsErr> {
        let keystore = self.keystore().await?;

        let account_id = keystore
            .get_account_id_by_key_commitment((*pub_key_commitment.as_native()).into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get account by key commitment"))?;

        match account_id {
            Some(id) => self.get_account(&id.into()).await,
            None => Ok(None),
        }
    }
}
