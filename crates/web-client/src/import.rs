use js_export_macro::js_export;
use miden_client::account::{AccountFile as NativeAccountFile, AccountId as NativeAccountId};
use miden_client::keystore::Keystore;
#[cfg(feature = "browser")]
use wasm_bindgen::prelude::*;

use crate::helpers::generate_wallet;
use crate::models::account::Account;
use crate::models::account_file::AccountFile;
use crate::models::account_id::AccountId as JsAccountId;
use crate::models::account_storage_mode::AccountStorageMode;
use crate::models::auth::AuthScheme;
use crate::models::note_file::NoteFile;
use crate::models::note_id::NoteId;
use crate::platform::{JsErr, from_str_err};
use crate::{WebClient, js_error_with_context};

#[js_export]
impl WebClient {
    #[js_export(js_name = "importAccountFile")]
    pub async fn import_account_file(&self, account_file: AccountFile) -> Result<String, JsErr> {
        let keystore = self.get_keystore().await?;
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let account_data: NativeAccountFile = account_file.into();
        let account_id = account_data.account.id().to_string();

        let NativeAccountFile { account, auth_secret_keys } = account_data;

        client
            .add_account(&account.clone(), false)
            .await
            .map_err(|err| js_error_with_context(err, "failed to import account"))?;

        for key in &auth_secret_keys {
            keystore
                .add_key(key, account.id())
                .await
                .map_err(|err| from_str_err(&err.to_string()))?;
        }

        Ok(format!("Imported account with ID: {account_id}"))
    }

    #[js_export(js_name = "importPublicAccountFromSeed")]
    pub async fn import_public_account_from_seed(
        &self,
        init_seed: Vec<u8>,
        mutable: bool,
        auth_scheme: AuthScheme,
    ) -> Result<Account, JsErr> {
        let keystore = self.get_keystore().await?;

        let (generated_acct, key_pair) =
            generate_wallet(&AccountStorageMode::public(), mutable, Some(init_seed), auth_scheme)
                .await?;

        let native_id = generated_acct.id();

        {
            let mut guard = self.get_mut_inner().await;
            let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
            client
                .import_account_by_id(native_id)
                .await
                .map_err(|err| js_error_with_context(err, "failed to import public account"))?;
        }

        keystore
            .add_key(&key_pair, native_id)
            .await
            .map_err(|err| from_str_err(&err.to_string()))?;

        Ok(Account::from(generated_acct))
    }

    #[js_export(js_name = "importAccountById")]
    pub async fn import_account_by_id(&self, account_id: &JsAccountId) -> Result<(), JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;

        let native_id: NativeAccountId = account_id.into();

        client
            .import_account_by_id(native_id)
            .await
            .map_err(|err| js_error_with_context(err, "failed to import public account"))?;

        Ok(())
    }

    #[js_export(js_name = "importNoteFile")]
    pub async fn import_note_file(&self, note_file: NoteFile) -> Result<NoteId, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        Ok(client
            .import_notes(&[note_file.into()])
            .await
            .map_err(|err| js_error_with_context(err, "failed to import note"))?[0]
            .into())
    }
}

/// Imports store contents from a JSON string, replacing all existing data.
///
/// Use together with [`export_store`].
#[cfg(feature = "browser")]
#[wasm_bindgen(js_name = "importStore")]
pub async fn import_store(store_name: &str, store_dump: &str) -> Result<(), JsValue> {
    let store = idxdb_store::IdxdbStore::new(store_name.into())
        .await
        .map_err(|err| JsValue::from_str(&format!("failed to open store: {err:?}")))?;

    store
        .import_store(store_dump.to_string())
        .await
        .map_err(|err| JsValue::from_str(&format!("failed to import store: {err:?}")))?;

    Ok(())
}
