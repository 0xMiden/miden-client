use miden_client::note::NoteTag;

use crate::prelude::*;
use crate::WebClient;

// Shared methods
#[bindings]
impl WebClient {
    #[bindings(js_name = "addTag")]
    pub async fn add_tag(&self, tag: String) -> platform::JsResult<()> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let note_tag_as_u32 = tag
            .parse::<u32>()
            .map_err(|err| platform::error_with_context(err, "failed to parse input note tag"))?;

        let note_tag: NoteTag = note_tag_as_u32.into();
        client
            .add_note_tag(note_tag)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to add note tag"))?;

        Ok(())
    }

    #[bindings(js_name = "removeTag")]
    pub async fn remove_tag(&self, tag: String) -> platform::JsResult<()> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let note_tag_as_u32 = tag
            .parse::<u32>()
            .map_err(|err| platform::error_with_context(err, "failed to parse input note tag"))?;

        let note_tag: NoteTag = note_tag_as_u32.into();
        client
            .remove_note_tag(note_tag)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to remove note tag"))?;

        Ok(())
    }

    #[bindings(js_name = "listTags")]
    pub async fn list_tags(&self) -> platform::JsResult<Vec<String>> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let tags: Vec<NoteTag> = client
            .get_note_tags()
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get note tags"))?
            .into_iter()
            .map(|tag_record| tag_record.tag)
            .collect();

        Ok(tags.iter().map(ToString::to_string).collect::<Vec<String>>())
    }
}
