use miden_client::{Word, store::NoteExportType, utils::Serializable};
use wasm_bindgen::prelude::*;

use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "exportNote")]
    pub async fn export_note(
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

    /// Retrieves the entire underlying web store and returns it as a JsValue
    ///
    /// Meant to be used in conjunction with the force_import_store method
    #[wasm_bindgen(js_name = "exportStore")]
    pub async fn export_store(&mut self) -> Result<JsValue, JsValue> {
        let store = self.store.as_ref().ok_or(JsValue::from_str("Store not initialized"))?;
        let export = store
            .export_store()
            .await
            .map_err(|err| js_error_with_context(err, "failed to export store"))?;

        Ok(export)
    }
}
