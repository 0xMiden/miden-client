use wasm_bindgen::prelude::*;

use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    /// Send a private note via the note transport layer
    #[wasm_bindgen(js_name = "sendPrivateNote")]
    pub async fn send_private_note(
        &mut self,
        note: crate::models::note::Note,
        address: crate::models::address::Address,
    ) -> Result<(), JsValue> {
        let client = self.get_mut_inner().ok_or_else(|| {
            JsValue::from_str("Client not initialized. Call createClient() first.")
        })?;

        client
            .send_private_note(note.into(), &address.into())
            .await
            .map_err(|e| js_error_with_context(e, "failed sending private note"))?;

        Ok(())
    }

    /// Re-send a previously-sent private note by its ID. Looks the note up
    /// from the client's output-note store, reconstructs it, and hands it
    /// to the transport layer. Intended for recovery after a transport
    /// failure: the on-chain transaction committed but the blob never
    /// reached the recipient — without a resend they can never discover
    /// the note (it's private; no on-chain details), so the sender's
    /// asset is effectively lost.
    ///
    /// Fails with a clear error if the note isn't tracked locally (the
    /// caller wasn't the sender) or if the stored record lacks full
    /// recipient data.
    #[wasm_bindgen(js_name = "resendPrivateNoteById")]
    pub async fn resend_private_note_by_id(
        &mut self,
        note_id: crate::models::note_id::NoteId,
        address: crate::models::address::Address,
    ) -> Result<(), JsValue> {
        let client = self.get_mut_inner().ok_or_else(|| {
            JsValue::from_str("Client not initialized. Call createClient() first.")
        })?;

        client
            .resend_private_note_by_id(note_id.into(), &address.into())
            .await
            .map_err(|e| js_error_with_context(e, "failed resending private note by id"))?;

        Ok(())
    }

    /// Fetch private notes from the note transport layer
    ///
    /// Uses an internal pagination mechanism to avoid fetching duplicate notes.
    #[wasm_bindgen(js_name = "fetchPrivateNotes")]
    pub async fn fetch_private_notes(&mut self) -> Result<(), JsValue> {
        let client = self.get_mut_inner().ok_or_else(|| {
            JsValue::from_str("Client not initialized. Call createClient() first.")
        })?;

        client
            .fetch_private_notes()
            .await
            .map_err(|e| js_error_with_context(e, "failed fetching private notes"))?;

        Ok(())
    }

    /// Fetch all private notes from the note transport layer
    ///
    /// Fetches all notes stored in the transport layer, with no pagination.
    /// Prefer using [`WebClient::fetch_private_notes`] for a more efficient, on-going,
    /// fetching mechanism.
    #[wasm_bindgen(js_name = "fetchAllPrivateNotes")]
    pub async fn fetch_all_private_notes(&mut self) -> Result<(), JsValue> {
        let client = self.get_mut_inner().ok_or_else(|| {
            JsValue::from_str("Client not initialized. Call createClient() first.")
        })?;

        client
            .fetch_all_private_notes()
            .await
            .map_err(|e| js_error_with_context(e, "failed fetching all private notes"))?;

        Ok(())
    }
}
