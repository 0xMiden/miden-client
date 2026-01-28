use miden_client::Word;
use miden_client::note::NoteId;
use wasm_bindgen::prelude::*;

use crate::models::account_id::AccountId;
use crate::models::consumable_note_record::ConsumableNoteRecord;
use crate::models::input_note_record::InputNoteRecord;
use crate::models::note_filter::NoteFilter;
use crate::models::output_note_record::OutputNoteRecord;
use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "getInputNotes")]
    pub async fn get_input_notes(
        &mut self,
        filter: NoteFilter,
    ) -> Result<Vec<InputNoteRecord>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let result = client
                .get_input_notes(filter.into())
                .await
                .map_err(|err| js_error_with_context(err, "failed to get input notes"))?;
            Ok(result.into_iter().map(Into::into).collect())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "getInputNote")]
    pub async fn get_input_note(
        &mut self,
        note_id: String,
    ) -> Result<Option<InputNoteRecord>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let note_id: NoteId = NoteId::from_raw(
                Word::try_from(note_id)
                    .map_err(|err| js_error_with_context(err, "failed to parse input note id"))?,
            );
            let result = client
                .get_input_note(note_id)
                .await
                .map_err(|err| js_error_with_context(err, "failed to get input note"))?;

            Ok(result.map(Into::into))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "getOutputNotes")]
    pub async fn get_output_notes(
        &mut self,
        filter: NoteFilter,
    ) -> Result<Vec<OutputNoteRecord>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let notes = client
                .get_output_notes(filter.into())
                .await
                .map_err(|err| js_error_with_context(err, "failed to get output notes"))?;
            Ok(notes.into_iter().map(Into::into).collect())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "getOutputNote")]
    pub async fn get_output_note(&mut self, note_id: String) -> Result<OutputNoteRecord, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let note_id: NoteId = NoteId::from_raw(
                Word::try_from(note_id)
                    .map_err(|err| js_error_with_context(err, "failed to parse output note id"))?,
            );
            let note = client
                .get_output_note(note_id)
                .await
                .map_err(|err| js_error_with_context(err, "failed to get output note"))?
                .ok_or_else(|| JsValue::from_str("Note not found"))?;

            Ok(note.into())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "getConsumableNotes")]
    pub async fn get_consumable_notes(
        &mut self,
        account_id: Option<AccountId>,
    ) -> Result<Vec<ConsumableNoteRecord>, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let native_account_id = account_id.map(Into::into);
            let result = Box::pin(client.get_consumable_notes(native_account_id))
                .await
                .map_err(|err| js_error_with_context(err, "failed to get consumable notes"))?;

            Ok(result.into_iter().map(Into::into).collect())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }
}
