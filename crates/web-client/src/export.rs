use miden_client::Word;
use miden_client::account::AccountFile as NativeAccountFile;
use miden_client::keystore::Keystore;
use miden_client::note::NoteId;

#[cfg(feature = "browser")]
use wasm_bindgen::prelude::*;
use js_export_macro::js_export;

use crate::models::account_file::AccountFile;
use crate::models::account_id::AccountId;
use crate::models::note_export_format::NoteExportFormat;
use crate::models::note_file::NoteFile;
use crate::platform::{JsErr, from_str_err};
use crate::{WebClient, js_error_with_context};

#[js_export]
impl WebClient {
    #[js_export(js_name = "exportNoteFile")]
    pub async fn export_note_file(
        &self,
        note_id: String,
        export_format: NoteExportFormat,
    ) -> Result<NoteFile, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let note_id = NoteId::from_raw(Word::try_from(note_id).map_err(|err| {
            js_error_with_context(
                err,
                "error exporting note file: failed to parse input note id",
            )
        })?);

        let output_note = client
            .get_output_note(note_id)
            .await
            .map_err(|err| {
                js_error_with_context(
                    err,
                    "error exporting note file: failed to get output notes",
                )
            })?
            .ok_or(from_str_err("No output note found"))?;

        let export_type = export_format.into();

        let note_file = output_note.into_note_file(&export_type).map_err(|err| {
            js_error_with_context(err, "failed to convert output note to note file")
        })?;

        Ok(note_file.into())
    }

    #[js_export(js_name = "exportAccountFile")]
    pub async fn export_account_file(
        &self,
        account_id: AccountId,
    ) -> Result<AccountFile, JsErr> {
        let keystore = self.keystore().await?;
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let account = client
            .get_account(account_id.into())
            .await
            .map_err(|err| {
                js_error_with_context(
                    err,
                    &format!(
                        "failed to get account for account id: {}",
                        account_id.to_string()
                    ),
                )
            })?
            .ok_or_else(|| {
                from_str_err(&format!(
                    "Account with ID {} not found",
                    account_id.to_string()
                ))
            })?;

        let key_pairs =
            keystore.get_keys_for_account(account_id.as_native()).await.map_err(|err| {
                js_error_with_context(
                    err,
                    &format!("failed to get keys for account: {}", &account_id.to_string()),
                )
            })?;

        let account_data = NativeAccountFile::new(account, key_pairs);

        Ok(AccountFile::from(account_data))
    }
}

/// Retrieves the entire underlying store and returns it as a `JsValue`
///
/// Meant to be used in conjunction with the `forceImportStore` method
#[cfg(feature = "browser")]
#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "exportStore")]
    pub async fn export_store(&self) -> Result<JsValue, JsValue> {
        let store_guard = self.store.lock().await;
        let store = store_guard.as_ref().ok_or(JsValue::from_str("Store not initialized"))?;

        let json_string = store
            .export_store()
            .await
            .map_err(|err| js_error_with_context(err, "failed to export store"))?;

        Ok(JsValue::from_str(&json_string))
    }
}
