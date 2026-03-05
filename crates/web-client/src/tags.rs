use miden_client::note::NoteTag;

use js_export_macro::js_export;

use crate::platform::{JsErr, from_str_err};
use crate::{WebClient, js_error_with_context};

#[js_export]
impl WebClient {
    #[js_export(js_name = "addTag")]
    pub async fn add_tag(&self, tag: String) -> Result<(), JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let note_tag_as_u32 = tag
            .parse::<u32>()
            .map_err(|err| js_error_with_context(err, "failed to parse input note tag"))?;

        let note_tag: NoteTag = note_tag_as_u32.into();
        client
            .add_note_tag(note_tag)
            .await
            .map_err(|err| js_error_with_context(err, "failed to add note tag"))?;

        Ok(())
    }

    #[js_export(js_name = "removeTag")]
    pub async fn remove_tag(&self, tag: String) -> Result<(), JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let note_tag_as_u32 = tag
            .parse::<u32>()
            .map_err(|err| js_error_with_context(err, "failed to parse input note tag"))?;

        let note_tag: NoteTag = note_tag_as_u32.into();
        client
            .remove_note_tag(note_tag)
            .await
            .map_err(|err| js_error_with_context(err, "failed to remove note tag"))?;

        Ok(())
    }

    #[js_export(js_name = "listTags")]
    pub async fn list_tags(&self) -> Result<Vec<String>, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let tags: Vec<NoteTag> = client
            .get_note_tags()
            .await
            .map_err(|err| js_error_with_context(err, "failed to get note tags"))?
            .into_iter()
            .map(|tag_record| tag_record.tag)
            .collect();

        Ok(tags.iter().map(ToString::to_string).collect::<Vec<String>>())
    }
}
