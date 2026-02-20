use miden_client::account::{AccountFile as NativeAccountFile, AccountId as NativeAccountId};

use crate::prelude::*;
use crate::helpers::generate_wallet;
use crate::models::account::Account;
use crate::models::account_file::AccountFile;
use crate::models::account_id::AccountId as JsAccountId;
use crate::models::account_storage_mode::AccountStorageMode;
use crate::models::auth::AuthScheme;
use crate::models::note_file::NoteFile;
use crate::models::note_id::NoteId;
use crate::WebClient;

// Shared methods (client-only, no keystore access)
#[bindings]
impl WebClient {
    #[bindings(js_name = "importAccountById")]
    pub async fn import_account_by_id(
        &self,
        account_id: &JsAccountId,
    ) -> platform::JsResult<()> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let native_id: NativeAccountId = account_id.into();

        client
            .import_account_by_id(native_id)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to import public account"))?;

        Ok(())
    }

    #[bindings(js_name = "importNoteFile")]
    pub async fn import_note_file(&self, note_file: &NoteFile) -> platform::JsResult<NoteId> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let result = client
            .import_notes(core::slice::from_ref(&note_file.inner))
            .await
            .map_err(|err| platform::error_with_context(err, "failed to import note"))?;

        Ok(result[0].into())
    }

    #[bindings(js_name = "importAccountFile")]
    pub async fn import_account_file(
        &self,
        account_file: &AccountFile,
    ) -> platform::JsResult<String> {
        let account_data: NativeAccountFile = account_file.clone().into();
        let account_id = account_data.account.id().to_string();

        let NativeAccountFile { account, auth_secret_keys } = account_data;

        // Add keys to keystore
        {
            let keystore = lock_keystore!(self)?;
            for key in &auth_secret_keys {
                #[cfg(feature = "wasm")]
                keystore.add_secret_key(key).await.map_err(|err| {
                    platform::error_with_context(err, "failed to add secret key to keystore")
                })?;
                #[cfg(feature = "napi")]
                keystore.add_key(key).map_err(|err| {
                    platform::error_with_context(err, "failed to add secret key to keystore")
                })?;
            }
        }

        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .add_account(&account.clone(), false)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to import account"))?;

        let pub_keys: Vec<_> = auth_secret_keys
            .iter()
            .map(miden_client::auth::AuthSecretKey::public_key)
            .collect();
        client
            .register_account_public_key_commitments(&account.id(), &pub_keys)
            .await
            .map_err(|err| {
                platform::error_with_context(err, "failed to map account to public keys")
            })?;

        Ok(format!("Imported account with ID: {account_id}"))
    }

    #[bindings(js_name = "importPublicAccountFromSeed")]
    pub async fn import_public_account_from_seed(
        &self,
        init_seed: Vec<u8>,
        mutable: bool,
        auth_scheme: AuthScheme,
    ) -> platform::JsResult<Account> {
        let (generated_acct, key_pair) =
            generate_wallet(&AccountStorageMode::public(), mutable, Some(init_seed), auth_scheme)?;

        // Add key to keystore
        {
            let keystore = lock_keystore!(self)?;
            #[cfg(feature = "wasm")]
            keystore
                .add_secret_key(&key_pair)
                .await
                .map_err(|err| {
                    platform::error_with_context(err, "failed to add secret key to keystore")
                })?;
            #[cfg(feature = "napi")]
            keystore
                .add_key(&key_pair)
                .map_err(|err| {
                    platform::error_with_context(err, "failed to add secret key to keystore")
                })?;
        }

        let native_id = generated_acct.id();
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .import_account_by_id(native_id)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to import public account"))?;

        client
            .register_account_public_key_commitments(&native_id, &[key_pair.public_key()])
            .await
            .map_err(|err| {
                platform::error_with_context(err, "failed to map account to public keys")
            })?;

        Ok(Account::from(generated_acct))
    }
}

// wasm-only: force_import_store (no napi equivalent)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "forceImportStore")]
    pub async fn force_import_store(
        &mut self,
        store_dump: JsValue,
        _store_name: &str,
    ) -> Result<JsValue, JsValue> {
        let store =
            self.store.as_ref().ok_or(platform::error_from_string("Store not initialized"))?;
        store
            .force_import_store(store_dump)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to force import store"))?;

        Ok(JsValue::from_str("Store imported successfully"))
    }
}
