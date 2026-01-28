use miden_client::Word;
use miden_client::account::AccountFile as NativeAccountFile;
use miden_client::note::NoteId;
use miden_client::store::NoteExportType;
use wasm_bindgen::prelude::*;

use crate::models::account_file::AccountFile;
use crate::models::account_id::AccountId;
use crate::models::note_file::NoteFile;
use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "exportNoteFile")]
    pub async fn export_note_file(
        &mut self,
        note_id: String,
        export_type: String,
    ) -> Result<NoteFile, JsValue> {
        if let Some(client) = self.get_mut_inner() {
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
                .ok_or(JsValue::from_str("No output note found"))?;

            let export_type = match export_type.as_str() {
                "Id" => NoteExportType::NoteId,
                "Full" => NoteExportType::NoteWithProof,
                "Details" => NoteExportType::NoteDetails,
                // Fail fast on unspecified/invalid export type instead of defaulting
                other => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid export type: {}. Expected one of: Id | Full | Details",
                        if other.is_empty() { "<empty>" } else { other }
                    )));
                },
            };

            let note_file = output_note.into_note_file(&export_type).map_err(|err| {
                js_error_with_context(err, "failed to convert output note to note file")
            })?;

            Ok(note_file.into())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    /// Retrieves the entire underlying web store and returns it as a `JsValue`
    ///
    /// Meant to be used in conjunction with the `force_import_store` method
    #[wasm_bindgen(js_name = "exportStore")]
    pub async fn export_store(&mut self) -> Result<JsValue, JsValue> {
        let store = self.store.as_ref().ok_or(JsValue::from_str("Store not initialized"))?;
        let export = store
            .export_store()
            .await
            .map_err(|err| js_error_with_context(err, "failed to export store"))?;

        Ok(export)
    }

    #[wasm_bindgen(js_name = "exportAccountFile")]
    pub async fn export_account_file(
        &mut self,
        account_id: AccountId,
    ) -> Result<AccountFile, JsValue> {
        let keystore = self.keystore.clone().expect("Keystore not initialized");
        if let Some(client) = self.get_mut_inner() {
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
                .ok_or(JsValue::from_str("No account found"))?;
            let account = account
                .try_into()
                .map_err(|_| JsValue::from_str("partial accounts are still unsupported"))?;

            let mut key_pairs = vec![];

            let commitments = client
                .get_account_public_key_commitments(account_id.as_native())
                .await
                .map_err(|err| {
                    js_error_with_context(
                        err,
                        &format!(
                            "failed to get public keys for account: {}",
                            &account_id.to_string()
                        ),
                    )
                })?;

            for commitment in commitments {
                key_pairs.push(
                    keystore
                        .get_key(commitment)
                        .await
                        .map_err(|err| {
                            js_error_with_context(err, "failed to get public key for account")
                        })?
                        .ok_or(JsValue::from_str("Auth not found for account"))?,
                );
            }

            let account_data = NativeAccountFile::new(account, key_pairs);

            Ok(AccountFile::from(account_data))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }
}
