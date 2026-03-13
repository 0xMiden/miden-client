use js_export_macro::js_export;
use miden_client::account::AccountId as NativeAccountId;
use miden_client::note::NoteTag as NativeNoteTag;

use super::account_id::AccountId;
use crate::platform::{JsErr, from_str_err};

/// Note tags are 32-bits of data that serve as best-effort filters for notes.
///
/// Tags enable quick lookups for notes related to particular use cases, scripts, or account
/// prefixes.
#[derive(Clone, Copy)]
#[js_export]
pub struct NoteTag(pub(crate) NativeNoteTag);

#[js_export]
impl NoteTag {
    /// Creates a new `NoteTag` from an arbitrary u32.
    #[js_export(constructor)]
    pub fn new(tag: u32) -> NoteTag {
        NoteTag(NativeNoteTag::new(tag))
    }

    /// Constructs a note tag that targets the given account ID.
    #[js_export(js_name = "withAccountTarget")]
    pub fn with_account_target(account_id: &AccountId) -> NoteTag {
        let native_account_id: NativeAccountId = account_id.into();
        NoteTag(NativeNoteTag::with_account_target(native_account_id))
    }

    /// Constructs a note tag that targets the given account ID with a custom tag length.
    #[js_export(js_name = "withCustomAccountTarget")]
    pub fn with_custom_account_target(
        account_id: &AccountId,
        tag_len: u8,
    ) -> Result<NoteTag, JsErr> {
        let native_account_id: NativeAccountId = account_id.into();
        NativeNoteTag::with_custom_account_target(native_account_id, tag_len)
            .map(NoteTag)
            .map_err(|err| from_str_err(&err.to_string()))
    }

    /// Returns the inner u32 value of this tag.
    #[js_export(js_name = "asU32")]
    pub fn as_u32(&self) -> u32 {
        self.0.as_u32()
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

impl_napi_from_value!(NoteTag);
