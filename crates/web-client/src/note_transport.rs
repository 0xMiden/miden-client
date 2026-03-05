use js_export_macro::js_export;

use crate::platform::{JsErr, from_str_err};
use crate::{WebClient, js_error_with_context};

#[js_export]
impl WebClient {
    /// Send a private note via the note transport layer
    #[js_export(js_name = "sendPrivateNote")]
    pub async fn send_private_note(
        &self,
        note: crate::models::note::Note,
        address: crate::models::address::Address,
    ) -> Result<(), JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| {
            from_str_err("Client not initialized. Call createClient() first.")
        })?;

        client
            .send_private_note(note.into(), &address.into())
            .await
            .map_err(|e| js_error_with_context(e, "failed sending private note"))?;

        Ok(())
    }

    /// Fetch private notes from the note transport layer
    ///
    /// Uses an internal pagination mechanism to avoid fetching duplicate notes.
    #[js_export(js_name = "fetchPrivateNotes")]
    pub async fn fetch_private_notes(&self) -> Result<(), JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| {
            from_str_err("Client not initialized. Call createClient() first.")
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
    #[js_export(js_name = "fetchAllPrivateNotes")]
    pub async fn fetch_all_private_notes(&self) -> Result<(), JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| {
            from_str_err("Client not initialized. Call createClient() first.")
        })?;

        client
            .fetch_all_private_notes()
            .await
            .map_err(|e| js_error_with_context(e, "failed fetching all private notes"))?;

        Ok(())
    }
}
