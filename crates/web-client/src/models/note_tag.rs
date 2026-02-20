use miden_client::account::AccountId as NativeAccountId;
use miden_client::note::NoteTag as NativeNoteTag;
use crate::prelude::*;

use super::account_id::AccountId;

/// Note tags are 32-bits of data that serve as best-effort filters for notes.
///
/// Tags enable quick lookups for notes related to particular use cases, scripts, or account
/// prefixes.
#[bindings]
#[derive(Clone, Copy)]
pub struct NoteTag(pub(crate) NativeNoteTag);

#[bindings]
impl NoteTag {
    #[bindings(constructor)]
    pub fn new(tag: u32) -> NoteTag {
        NoteTag(NativeNoteTag::new(tag))
    }

    pub fn as_u32(&self) -> u32 {
        self.0.as_u32()
    }

    #[bindings(factory)]
    pub fn with_account_target(account_id: &AccountId) -> NoteTag {
        let native_account_id: NativeAccountId = account_id.into();
        NoteTag(NativeNoteTag::with_account_target(native_account_id))
    }

    #[bindings(factory)]
    pub fn with_custom_account_target(
        account_id: &AccountId,
        tag_len: u8,
    ) -> JsResult<NoteTag> {
        let native_account_id: NativeAccountId = account_id.into();
        NativeNoteTag::with_custom_account_target(native_account_id, tag_len)
            .map(NoteTag)
            .map_err(|err| platform::error_with_context(err, "failed to create note tag with custom account target"))
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteTag> for NoteTag {
    fn from(native_note_tag: NativeNoteTag) -> Self {
        NoteTag(native_note_tag)
    }
}

impl From<&NativeNoteTag> for NoteTag {
    fn from(native_note_tag: &NativeNoteTag) -> Self {
        NoteTag(*native_note_tag)
    }
}

impl From<NoteTag> for NativeNoteTag {
    fn from(note_tag: NoteTag) -> Self {
        note_tag.0
    }
}

impl From<&NoteTag> for NativeNoteTag {
    fn from(note_tag: &NoteTag) -> Self {
        note_tag.0
    }
}
