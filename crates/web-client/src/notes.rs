use miden_client::Word;
use miden_client::note::NoteId;

use js_export_macro::js_export;

use crate::models::account_id::AccountId;
use crate::models::consumable_note_record::ConsumableNoteRecord;
use crate::models::input_note_record::InputNoteRecord;
use crate::models::note_filter::NoteFilter;
use crate::models::output_note_record::OutputNoteRecord;
use crate::platform::{JsErr, from_str_err};
use crate::{WebClient, js_error_with_context};

#[js_export]
impl WebClient {
    #[js_export(js_name = "getInputNotes")]
    pub async fn get_input_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<InputNoteRecord>, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let result = client
            .get_input_notes(filter.into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get input notes"))?;
        Ok(result.into_iter().map(Into::into).collect())
    }

    #[js_export(js_name = "getInputNote")]
    pub async fn get_input_note(
        &self,
        note_id: String,
    ) -> Result<Option<InputNoteRecord>, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let note_id: NoteId = NoteId::from_raw(
            Word::try_from(note_id)
                .map_err(|err| js_error_with_context(err, "failed to parse input note id"))?,
        );
        let result = client
            .get_input_note(note_id)
            .await
            .map_err(|err| js_error_with_context(err, "failed to get input note"))?;

        Ok(result.map(Into::into))
    }

    #[js_export(js_name = "getOutputNotes")]
    pub async fn get_output_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<OutputNoteRecord>, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let notes = client
            .get_output_notes(filter.into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get output notes"))?;
        Ok(notes.into_iter().map(Into::into).collect())
    }

    #[js_export(js_name = "getOutputNote")]
    pub async fn get_output_note(&self, note_id: String) -> Result<OutputNoteRecord, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let note_id: NoteId = NoteId::from_raw(
            Word::try_from(note_id)
                .map_err(|err| js_error_with_context(err, "failed to parse output note id"))?,
        );
        let note = client
            .get_output_note(note_id)
            .await
            .map_err(|err| js_error_with_context(err, "failed to get output note"))?
            .ok_or_else(|| from_str_err("Note not found"))?;

        Ok(note.into())
    }

    #[js_export(js_name = "getConsumableNotes")]
    pub async fn get_consumable_notes(
        &self,
        account_id: Option<AccountId>,
    ) -> Result<Vec<ConsumableNoteRecord>, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let native_account_id = account_id.map(Into::into);
        let result = Box::pin(client.get_consumable_notes(native_account_id))
            .await
            .map_err(|err| js_error_with_context(err, "failed to get consumable notes"))?;

        Ok(result.into_iter().map(Into::into).collect())
    }
}
