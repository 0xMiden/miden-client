use wasm_bindgen::prelude::*;

use crate::models::account::Account;
use crate::models::account_code::AccountCode;
use crate::models::account_header::AccountHeader;
use crate::models::account_id::AccountId;
use crate::models::account_reader::AccountReader;
use crate::models::account_storage::AccountStorage;
use crate::models::address::Address;
use crate::models::asset_vault::AssetVault;
use crate::models::felt::Felt;
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
        Ok(AccountReader::from(self.get_inner()?.account_reader(account_id.into())))
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

    /// Prunes historical account states for the specified account up to the given nonce.
    ///
    /// Deletes all historical entries with `replaced_at_nonce <= up_to_nonce` and any
    /// orphaned account code.
    ///
    /// Returns the total number of rows deleted, including historical entries and orphaned
    /// account code.
    #[wasm_bindgen(js_name = "pruneAccountHistory")]
    pub async fn prune_account_history(
        &mut self,
        account_id: &AccountId,
        up_to_nonce: &Felt,
    ) -> Result<u32, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let deleted = client
                .prune_account_history(account_id.into(), up_to_nonce.into())
                .await
                .map_err(|err| js_error_with_context(err, "failed to prune account history"))?;
            // SAFETY: on wasm32 usize is 32 bits, so this conversion is infallible
            Ok(u32::try_from(deleted).expect("deleted count should fit in u32"))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }
}
