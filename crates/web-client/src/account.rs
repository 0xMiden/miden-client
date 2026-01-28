use miden_client::Word as NativeWord;
use miden_client::account::{Account as NativeAccount, StorageSlotName};
use wasm_bindgen::prelude::*;

use crate::models::account::Account;
use crate::models::account_header::AccountHeader;
use crate::models::account_id::AccountId;
use crate::models::address::Address;
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

    /// Retrieves a storage slot value by name for the given account.
    ///
    /// This method fetches fresh data from storage on each call, providing lazy access
    /// to account storage without needing to fetch the full account first.
    ///
    /// For `Value` slots, returns the stored word.
    /// For `Map` slots, returns the map root.
    ///
    /// # Arguments
    /// * `account_id` - The ID of the account to read storage from.
    /// * `slot_name` - The name of the storage slot.
    ///
    /// # Errors
    /// Returns an error if the account or slot is not found.
    #[wasm_bindgen(js_name = "getStorageItem")]
    pub async fn get_storage_item(
        &mut self,
        account_id: &AccountId,
        slot_name: &str,
    ) -> Result<Word, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let slot_name = StorageSlotName::new(slot_name)
                .map_err(|err| js_error_with_context(err, "invalid slot name"))?;

            let value = client
                .storage(account_id.into())
                .get_item(slot_name)
                .await
                .map_err(|err| js_error_with_context(err, "failed to get storage item"))?;

            Ok(value.into())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    /// Retrieves a value from a storage map slot by name and key.
    ///
    /// This method fetches fresh data from storage on each call, providing lazy access
    /// to account storage maps without needing to fetch the full account first.
    ///
    /// # Arguments
    /// * `account_id` - The ID of the account to read storage from.
    /// * `slot_name` - The name of the storage map slot.
    /// * `key` - The key within the map.
    ///
    /// # Errors
    /// Returns an error if the account or slot is not found, or if the slot is not a map.
    #[wasm_bindgen(js_name = "getStorageMapItem")]
    pub async fn get_storage_map_item(
        &mut self,
        account_id: &AccountId,
        slot_name: &str,
        key: &Word,
    ) -> Result<Word, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let slot_name = StorageSlotName::new(slot_name)
                .map_err(|err| js_error_with_context(err, "invalid slot name"))?;

            let value = client
                .storage(account_id.into())
                .get_map_item(slot_name, (*key.as_native()).into())
                .await
                .map_err(|err| js_error_with_context(err, "failed to get storage map item"))?;

            Ok(value.into())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }
}
