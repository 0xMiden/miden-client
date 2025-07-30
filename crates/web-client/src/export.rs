use miden_client::Word;
use miden_client::account::{Account, AccountFile, AccountId};
use miden_client::store::NoteExportType;
use miden_client::transaction::AccountInterface;
use miden_client::utils::Serializable;
use miden_lib::AuthScheme;
use miden_lib::account::auth::NoAuth;
use wasm_bindgen::prelude::*;

use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "exportNoteFile")]
    pub async fn export_note_file(
        &mut self,
        note_id: String,
        export_type: String,
    ) -> Result<JsValue, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let note_id = Word::try_from(note_id)
                .map_err(|err| js_error_with_context(err, "failed to parse input note id"))?
                .into();

            let output_note = client
                .get_output_note(note_id)
                .await
                .map_err(|err| js_error_with_context(err, "failed to get output notes"))?
                .ok_or(JsValue::from_str("No output note found"))?;

            let export_type = match export_type.as_str() {
                "Id" => NoteExportType::NoteId,
                "Full" => NoteExportType::NoteWithProof,
                _ => NoteExportType::NoteDetails,
            };

            let note_file = output_note.into_note_file(&export_type).map_err(|err| {
                js_error_with_context(err, "failed to convert output note to note file")
            })?;

            let input_note_bytes = note_file.to_bytes();

            let serialized_input_note_bytes = serde_wasm_bindgen::to_value(&input_note_bytes)
                .map_err(|_| JsValue::from_str("Serialization error"))?;

            Ok(serialized_input_note_bytes)
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

    #[wasm_bindgen(js_name = "exportAccount")]
    pub async fn export_account(&mut self, account_id: String) -> Result<JsValue, JsValue> {
        let keystore = self.keystore.clone();
        if let Some(client) = self.get_mut_inner() {
            let account_id = if account_id.starts_with("0x") {
                AccountId::from_hex(&account_id)
                    .map_err(|err| js_error_with_context(err, "failed to parse the account id"))?
            } else {
                AccountId::from_bech32(&account_id)
                    .map_err(|err| js_error_with_context(err, "failed to parse the account id"))?
                    .1
            };

            let account = client
                .get_account(account_id)
                .await
                .map_err(|err| js_error_with_context(err, "failed to get account for account id"))?
                .ok_or(JsValue::from_str("No account found"))?;

            let account_seed = account.seed().copied();

            let account: Account = account.into();

            let keystore = keystore.expect("KeyStore should be initialised");

            let mut key_pairs = vec![];

            for pub_key in get_public_keys_from_account(&account) {
                key_pairs.push(
                    keystore
                        .get_key(pub_key)
                        .map_err(|err| {
                            js_error_with_context(err, "failed to get public key for account")
                        })?
                        .ok_or(JsValue::from_str("Auth not found for account"))?,
                );
            }

            let account_data = AccountFile::new(account, account_seed, key_pairs);
            let mut account_file_bytes = vec![];
            account_data.write_into(&mut account_file_bytes);

            let serialized_input_note_bytes = serde_wasm_bindgen::to_value(&account_file_bytes)
                .map_err(|_| JsValue::from_str("Serialization error"))?;

            Ok(serialized_input_note_bytes)
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }
}

/// Gets the public key from the storage of an account. This will only work if the account is
/// created by the CLI as it expects the account to have the `RpoFalcon512` authentication scheme.
pub fn get_public_keys_from_account(account: &Account) -> Vec<Word> {
    let mut pub_keys = vec![];
    let interface: AccountInterface = account.into();

    for auth in interface.auth() {
        match auth {
            AuthScheme::RpoFalcon512 { pub_key } => pub_keys.push(Word::from(*pub_key)),
            NoAuth => {},
        }
    }

    pub_keys
}
