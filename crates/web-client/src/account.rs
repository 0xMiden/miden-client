use miden_client::Word as NativeWord;

use crate::prelude::*;

#[cfg(feature = "napi")]
use miden_client::auth::PublicKeyCommitment;

use crate::WebClient;
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

// Shared methods (client-only access)
#[bindings]
impl WebClient {
    #[bindings(js_name = "getAccounts")]
    pub async fn get_accounts(&self) -> platform::JsResult<Vec<AccountHeader>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let result = client
            .get_account_headers()
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get accounts"))?;

        Ok(result.into_iter().map(|(header, _)| header.into()).collect())
    }

    /// Retrieves the full account data for the given account ID, returning `null` if not found.
    ///
    /// This method loads the complete account state including vault, storage, and code.
    #[bindings(js_name = "getAccount")]
    pub async fn get_account(
        &self,
        account_id: &AccountId,
    ) -> platform::JsResult<Option<Account>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .get_account(account_id.into())
            .await
            .map(|opt| opt.map(Into::into))
            .map_err(|err| platform::error_with_context(err, "failed to get account"))
    }

    /// Retrieves the asset vault for a specific account.
    ///
    /// To check the balance for a single asset, use `accountReader` instead.
    #[bindings(js_name = "getAccountVault")]
    pub async fn get_account_vault(
        &self,
        account_id: &AccountId,
    ) -> platform::JsResult<AssetVault> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .get_account_vault(account_id.into())
            .await
            .map(Into::into)
            .map_err(|err| platform::error_with_context(err, "failed to get account vault"))
    }

    /// Retrieves the storage for a specific account.
    ///
    /// To only load a specific slot, use `accountReader` instead.
    #[bindings(js_name = "getAccountStorage")]
    pub async fn get_account_storage(
        &self,
        account_id: &AccountId,
    ) -> platform::JsResult<AccountStorage> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .get_account_storage(account_id.into())
            .await
            .map(Into::into)
            .map_err(|err| platform::error_with_context(err, "failed to get account storage"))
    }

    /// Retrieves the account code for a specific account.
    ///
    /// Returns `null` if the account is not found.
    #[bindings(js_name = "getAccountCode")]
    pub async fn get_account_code(
        &self,
        account_id: &AccountId,
    ) -> platform::JsResult<Option<AccountCode>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .get_account_code(account_id.into())
            .await
            .map(|opt| opt.map(Into::into))
            .map_err(|err| platform::error_with_context(err, "failed to get account code"))
    }

    /// Returns all public key commitments associated with the given account ID.
    ///
    /// These commitments can be used with [`getAccountAuthByPubKeyCommitment`]
    /// to retrieve the corresponding secret keys from the keystore.
    #[bindings(js_name = "getPublicKeyCommitmentsOfAccount")]
    pub async fn get_public_key_commitments_of(
        &self,
        account_id: &AccountId,
    ) -> platform::JsResult<Vec<Word>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        Ok(client
            .get_account_public_key_commitments(account_id.as_native())
            .await
            .map_err(|err| {
                platform::error_with_context(
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

    #[bindings(js_name = "insertAccountAddress")]
    pub async fn insert_account_address(
        &self,
        account_id: &AccountId,
        address: &Address,
    ) -> platform::JsResult<()> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .add_address(address.into(), account_id.into())
            .await
            .map_err(|err| platform::error_with_context(err, "failed to add address to account"))?;
        Ok(())
    }

    #[bindings(js_name = "removeAccountAddress")]
    pub async fn remove_account_address(
        &self,
        account_id: &AccountId,
        address: &Address,
    ) -> platform::JsResult<()> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .remove_address(address.into(), account_id.into())
            .await
            .map_err(|err| {
                platform::error_with_context(err, "failed to remove address from account")
            })?;
        Ok(())
    }
}

// account_reader — uses store directly (different types per platform)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WebClient {
    /// Creates a new `AccountReader` for lazy access to account data.
    ///
    /// The `AccountReader` executes queries lazily - each method call fetches fresh data
    /// from storage, ensuring you always see the current state.
    #[wasm_bindgen(js_name = "accountReader")]
    pub fn account_reader(&self, account_id: &AccountId) -> platform::JsResult<AccountReader> {
        let store =
            self.store.clone().ok_or(platform::error_from_string("Store not initialized"))?;
        Ok(AccountReader::new(store, account_id.into()))
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl WebClient {
    /// Creates a new `AccountReader` for lazy access to account data.
    ///
    /// The `AccountReader` executes queries lazily - each method call fetches fresh data
    /// from storage, ensuring you always see the current state.
    pub async fn account_reader(&self, account_id: &AccountId) -> platform::JsResult<AccountReader> {
        let store = lock_store!(self)?;
        Ok(AccountReader::new(store, account_id.into()))
    }
}

// get_account_auth_secret_key_by_pub_key_commitment — different keystore APIs
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WebClient {
    /// Retrieves an authentication secret key from the keystore given a public key commitment.
    #[wasm_bindgen(js_name = "getAccountAuthByPubKeyCommitment")]
    pub async fn get_account_auth_secret_key_by_pub_key_commitment(
        &self,
        pub_key_commitment: &Word,
    ) -> platform::JsResult<AuthSecretKey> {
        let keystore = self.keystore.clone().expect("Keystore not initialized");

        let auth_secret_key = keystore
            .get_secret_key((*pub_key_commitment.as_native()).into())
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get auth key for account"))?
            .ok_or(platform::error_from_string("Auth not found for account"))?;

        Ok(auth_secret_key.into())
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl WebClient {
    /// Retrieves an authentication secret key from the keystore given a public key commitment.
    #[napi_derive::napi(js_name = "getAccountAuthByPubKeyCommitment")]
    pub async fn get_account_auth_secret_key_by_pub_key_commitment(
        &self,
        pub_key_commitment: &Word,
    ) -> platform::JsResult<AuthSecretKey> {
        let keystore = lock_keystore!(self)?;

        let native_word: NativeWord = pub_key_commitment.into();
        let commitment = PublicKeyCommitment::from(native_word);

        let auth_secret_key = keystore
            .get_key(commitment)
            .map_err(|err| platform::error_with_context(err, "failed to get auth key for account"))?
            .ok_or_else(|| platform::error_from_string("Auth not found for account"))?;

        Ok(auth_secret_key.into())
    }
}
