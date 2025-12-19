use miden_client::account::{AccountFile as NativeAccountFile, AccountId as NativeAccountId};
use wasm_bindgen::prelude::*;

use crate::helpers::generate_wallet;
use crate::models::account::Account;
use crate::models::account_file::AccountFile;
use crate::models::account_id::AccountId as JsAccountId;
use crate::models::account_storage_mode::AccountStorageMode;
use crate::models::auth::AuthScheme;
use crate::models::note_file::NoteFile;
use crate::models::note_id::NoteId;
use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "importAccountFile")]
    pub async fn import_account_file(
        &mut self,
        account_file: AccountFile,
    ) -> Result<JsValue, JsValue> {
        let keystore = self.keystore.clone();
        if let Some(client) = self.get_mut_inner() {
            let account_data: NativeAccountFile = account_file.into();
            let account_id = account_data.account.id().to_string();

            let NativeAccountFile { account, auth_secret_keys } = account_data;

            client
                .add_account(&account.clone(), false)
                .await
                .map_err(|err| js_error_with_context(err, "failed to import account"))?;

            let keystore = keystore.expect("KeyStore should be initialized");
            for key in auth_secret_keys {
                keystore.add_key(&key, &account.id()).await.map_err(|err| err.to_string())?;
            }

            Ok(JsValue::from_str(&format!("Imported account with ID: {account_id}")))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "importPublicAccountFromSeed")]
    pub async fn import_public_account_from_seed(
        &mut self,
        init_seed: Vec<u8>,
        mutable: bool,
        auth_scheme: AuthScheme,
    ) -> Result<Account, JsValue> {
        let keystore = self.keystore.clone();
        let client = self.get_mut_inner().ok_or(JsValue::from_str("Client not initialized"))?;

        let (generated_acct, key_pair) =
            generate_wallet(&AccountStorageMode::public(), mutable, Some(init_seed), auth_scheme)
                .await?;

        let native_id = generated_acct.id();
        client
            .import_account_by_id(native_id)
            .await
            .map_err(|err| js_error_with_context(err, "failed to import public account"))?;

        keystore
            .expect("KeyStore should be initialized")
            .add_key(&key_pair, &native_id)
            .await
            .map_err(|err| err.to_string())?;

        Ok(Account::from(generated_acct))
    }

    #[wasm_bindgen(js_name = "importAccountById")]
    pub async fn import_account_by_id(
        &mut self,
        account_id: &JsAccountId,
    ) -> Result<JsValue, JsValue> {
        let client = self
            .get_mut_inner()
            .ok_or_else(|| JsValue::from_str("Client not initialized"))?;

        let native_id: NativeAccountId = account_id.into();

        client
            .import_account_by_id(native_id)
            .await
            .map(|_| JsValue::undefined())
            .map_err(|err| js_error_with_context(err, "failed to import public account"))
    }

    #[wasm_bindgen(js_name = "importNoteFile")]
    pub async fn import_note_file(&mut self, note_file: NoteFile) -> Result<NoteId, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            Ok(client
                .import_note(note_file.into())
                .await
                .map_err(|err| js_error_with_context(err, "failed to import note"))?
                .into())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "forceImportStore")]
    pub async fn force_import_store(&mut self, store_dump: JsValue) -> Result<JsValue, JsValue> {
        let store = self.store.as_ref().ok_or(JsValue::from_str("Store not initialized"))?;
        store
            .force_import_store(store_dump)
            .await
            .map_err(|err| js_error_with_context(err, "failed to force import store"))?;

        Ok(JsValue::from_str("Store imported successfully"))
    }
}
