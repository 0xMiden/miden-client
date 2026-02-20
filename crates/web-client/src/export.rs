use miden_client::Word;
use miden_client::account::AccountFile as NativeAccountFile;
use miden_client::note::NoteId;
use miden_client::store::NoteExportType;

use crate::prelude::*;
use crate::WebClient;
use crate::models::account_file::AccountFile;
use crate::models::account_id::AccountId;
use crate::models::note_file::NoteFile;

// Shared: export_note_file (client-only, identical logic)
#[bindings]
impl WebClient {
    #[bindings(js_name = "exportNoteFile")]
    pub async fn export_note_file(
        &self,
        note_id: String,
        export_type: String,
    ) -> platform::JsResult<NoteFile> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let note_id = NoteId::from_raw(Word::try_from(note_id).map_err(|err| {
            platform::error_with_context(
                err,
                "error exporting note file: failed to parse input note id",
            )
        })?);

        let output_note = client
            .get_output_note(note_id)
            .await
            .map_err(|err| {
                platform::error_with_context(
                    err,
                    "error exporting note file: failed to get output notes",
                )
            })?
            .ok_or_else(|| platform::error_from_string("No output note found"))?;

        let export_type = match export_type.as_str() {
            "Id" => NoteExportType::NoteId,
            "Full" => NoteExportType::NoteWithProof,
            "Details" => NoteExportType::NoteDetails,
            other => {
                return Err(platform::error_from_string(&format!(
                    "Invalid export type: {}. Expected one of: Id | Full | Details",
                    if other.is_empty() { "<empty>" } else { other }
                )));
            },
        };

        let note_file = output_note.into_note_file(&export_type).map_err(|err| {
            platform::error_with_context(err, "failed to convert output note to note file")
        })?;

        Ok(note_file.into())
    }

    #[bindings(js_name = "exportAccountFile")]
    pub async fn export_account_file(
        &self,
        account_id: &AccountId,
    ) -> platform::JsResult<AccountFile> {
        let native_id: miden_client::account::AccountId = account_id.into();

        let mut key_pairs = vec![];

        // Fetch keys from keystore
        {
            let mut guard = lock_client!(self);
            let client = guard
                .as_mut()
                .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

            let commitments = client
                .get_account_public_key_commitments(account_id.as_native())
                .await
                .map_err(|err| {
                    platform::error_with_context(
                        err,
                        &format!("failed to get public keys for account: {}", native_id),
                    )
                })?;

            // Drop client guard before locking keystore
            drop(guard);

            #[cfg(feature = "wasm")]
            {
                let keystore = self.keystore.clone().expect("Keystore not initialized");
                for commitment in commitments {
                    key_pairs.push(
                        keystore
                            .get_secret_key(commitment)
                            .await
                            .map_err(|err| {
                                platform::error_with_context(
                                    err,
                                    "failed to get public key for account",
                                )
                            })?
                            .ok_or(platform::error_from_string("Auth not found for account"))?,
                    );
                }
            }

            #[cfg(feature = "napi")]
            {
                let keystore = lock_keystore!(self)?;
                for commitment in commitments {
                    key_pairs.push(
                        keystore
                            .get_key(commitment)
                            .map_err(|err| {
                                platform::error_with_context(
                                    err,
                                    "failed to get public key for account",
                                )
                            })?
                            .ok_or_else(|| {
                                platform::error_from_string("Auth not found for account")
                            })?,
                    );
                }
            }
        }

        // Re-lock client to get account
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let account = client
            .get_account(native_id)
            .await
            .map_err(|err| {
                platform::error_with_context(
                    err,
                    &format!("failed to get account for account id: {}", native_id),
                )
            })?
            .ok_or_else(|| {
                platform::error_from_string(&format!("Account with ID {} not found", native_id))
            })?;

        let account_data = NativeAccountFile::new(account, key_pairs);

        Ok(AccountFile::from(account_data))
    }
}

// wasm-only: export_store (no napi equivalent)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WebClient {
    /// Retrieves the entire underlying web store and returns it as a `JsValue`
    ///
    /// Meant to be used in conjunction with the `force_import_store` method
    #[wasm_bindgen(js_name = "exportStore")]
    pub async fn export_store(&self) -> Result<JsValue, JsValue> {
        let store = self.store.as_ref().ok_or(platform::error_from_string("Store not initialized"))?;
        let export = store
            .export_store()
            .await
            .map_err(|err| platform::error_with_context(err, "failed to export store"))?;

        Ok(export)
    }
}
