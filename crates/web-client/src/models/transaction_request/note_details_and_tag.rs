use miden_client::note::{NoteDetails as NativeNoteDetails, NoteTag as NativeNoteTag};
use js_export_macro::js_export;

use crate::models::NoteTag;
use crate::models::miden_arrays::NoteDetailsAndTagArray;
use crate::models::note_details::NoteDetails;

/// Pair of note details and tag used when declaring expected notes.
#[derive(Clone)]
#[js_export]
pub struct NoteDetailsAndTag {
    note_details: NoteDetails,
    tag: NoteTag,
}

#[js_export]
impl NoteDetailsAndTag {
    /// Creates a new pair from note details and tag.
    #[js_export(constructor)]
    pub fn new(note_details: NoteDetails, tag: NoteTag) -> NoteDetailsAndTag {
        NoteDetailsAndTag { note_details, tag }
    }

    /// Returns the note details.
    #[js_export(getter, js_name = "noteDetails")]
    pub fn note_details(&self) -> NoteDetails {
        self.note_details.clone()
    }

    /// Returns the note tag.
    #[js_export(getter)]
    pub fn tag(&self) -> NoteTag {
        self.tag
    }
}

impl From<NoteDetailsAndTag> for (NativeNoteDetails, NativeNoteTag) {
    fn from(note_details_and_args: NoteDetailsAndTag) -> Self {
        let native_note_details: NativeNoteDetails = note_details_and_args.note_details.into();
        let native_tag: NativeNoteTag = note_details_and_args.tag.into();
        (native_note_details, native_tag)
    }
}

impl From<&NoteDetailsAndTag> for (NativeNoteDetails, NativeNoteTag) {
    fn from(note_details_and_args: &NoteDetailsAndTag) -> Self {
        let native_note_details: NativeNoteDetails =
            note_details_and_args.note_details.clone().into();
        let native_tag: NativeNoteTag = note_details_and_args.tag.into();
        (native_note_details, native_tag)
    }
}

impl From<NoteDetailsAndTagArray> for Vec<(NativeNoteDetails, NativeNoteTag)> {
    fn from(note_details_and_tag_array: NoteDetailsAndTagArray) -> Self {
        let items: Vec<NoteDetailsAndTag> = note_details_and_tag_array.into();
        items.into_iter().map(Into::into).collect()
    }
}

impl From<&NoteDetailsAndTagArray> for Vec<(NativeNoteDetails, NativeNoteTag)> {
    fn from(note_details_and_tag_array: &NoteDetailsAndTagArray) -> Self {
        let items: Vec<NoteDetailsAndTag> = note_details_and_tag_array.into();
        items.into_iter().map(Into::into).collect()
    }
}

impl_napi_from_value!(NoteDetailsAndTag);
