use miden_client::Word;
use miden_client::note::NoteId;

use crate::prelude::*;
use crate::WebClient;
use crate::models::account_id::AccountId;
use crate::models::consumable_note_record::ConsumableNoteRecord;
use crate::models::input_note_record::InputNoteRecord;
use crate::models::note_filter::NoteFilter;
use crate::models::output_note_record::OutputNoteRecord;

#[bindings]
impl WebClient {
    #[bindings(js_name = "getInputNotes")]
    pub async fn get_input_notes(
        &self,
        filter: &NoteFilter,
    ) -> platform::JsResult<Vec<InputNoteRecord>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let result = client
            .get_input_notes(filter.into())
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get input notes"))?;
        Ok(result.into_iter().map(Into::into).collect())
    }

    #[bindings(js_name = "getInputNote")]
    pub async fn get_input_note(
        &self,
        note_id: String,
    ) -> platform::JsResult<Option<InputNoteRecord>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let note_id: NoteId = NoteId::from_raw(
            Word::try_from(note_id)
                .map_err(|err| platform::error_with_context(err, "failed to parse input note id"))?,
        );
        let result = client
            .get_input_note(note_id)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get input note"))?;

        Ok(result.map(Into::into))
    }

    #[bindings(js_name = "getOutputNotes")]
    pub async fn get_output_notes(
        &self,
        filter: &NoteFilter,
    ) -> platform::JsResult<Vec<OutputNoteRecord>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let notes = client
            .get_output_notes(filter.into())
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get output notes"))?;
        Ok(notes.into_iter().map(Into::into).collect())
    }

    #[bindings(js_name = "getOutputNote")]
    pub async fn get_output_note(&self, note_id: String) -> platform::JsResult<OutputNoteRecord> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let note_id: NoteId = NoteId::from_raw(
            Word::try_from(note_id)
                .map_err(|err| platform::error_with_context(err, "failed to parse output note id"))?,
        );
        let note = client
            .get_output_note(note_id)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get output note"))?
            .ok_or_else(|| platform::error_from_string("Note not found"))?;

        Ok(note.into())
    }

    #[bindings(js_name = "getConsumableNotes")]
    pub async fn get_consumable_notes(
        &self,
        account_id: Option<AccountId>,
    ) -> platform::JsResult<Vec<ConsumableNoteRecord>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let native_account_id = account_id.map(|id| id.into());
        let result = Box::pin(client.get_consumable_notes(native_account_id))
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get consumable notes"))?;

        Ok(result.into_iter().map(Into::into).collect())
    }
}
