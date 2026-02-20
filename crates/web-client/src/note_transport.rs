use crate::prelude::*;
use crate::WebClient;

#[bindings]
impl WebClient {
    /// Send a private note via the note transport layer
    #[bindings(js_name = "sendPrivateNote")]
    pub async fn send_private_note(
        &self,
        note: crate::models::note::Note,
        address: crate::models::address::Address,
    ) -> platform::JsResult<()> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .send_private_note(note.into(), &address.into())
            .await
            .map_err(|e| platform::error_with_context(e, "failed sending private note"))?;

        Ok(())
    }

    /// Fetch private notes from the note transport layer
    ///
    /// Uses an internal pagination mechanism to avoid fetching duplicate notes.
    #[bindings(js_name = "fetchPrivateNotes")]
    pub async fn fetch_private_notes(&self) -> platform::JsResult<()> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .fetch_private_notes()
            .await
            .map_err(|e| platform::error_with_context(e, "failed fetching private notes"))?;

        Ok(())
    }

    /// Fetch all private notes from the note transport layer
    ///
    /// Fetches all notes stored in the transport layer, with no pagination.
    /// Prefer using [`WebClient::fetch_private_notes`] for a more efficient, on-going,
    /// fetching mechanism.
    #[bindings(js_name = "fetchAllPrivateNotes")]
    pub async fn fetch_all_private_notes(&self) -> platform::JsResult<()> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .fetch_all_private_notes()
            .await
            .map_err(|e| platform::error_with_context(e, "failed fetching all private notes"))?;

        Ok(())
    }
}
