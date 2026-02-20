use miden_client::note::{NoteDetails as NativeNoteDetails, NoteTag as NativeNoteTag};
use crate::prelude::*;

use crate::models::note_details::NoteDetails;
use crate::models::note_tag::NoteTag;

/// Pair of note details and tag used when declaring expected notes.
#[bindings]
#[derive(Clone)]
pub struct NoteDetailsAndTag {
    note_details: NoteDetails,
    tag: NoteTag,
}

#[bindings]
impl NoteDetailsAndTag {
    /// Creates a new pair from note details and tag.
    #[bindings(constructor)]
    pub fn new(note_details: &NoteDetails, tag: &NoteTag) -> NoteDetailsAndTag {
        NoteDetailsAndTag { note_details: note_details.clone(), tag: *tag }
    }

    /// Returns the note details.
    #[bindings(getter)]
    pub fn note_details(&self) -> NoteDetails {
        self.note_details.clone()
    }

    /// Returns the note tag.
    #[bindings(getter)]
    pub fn tag(&self) -> NoteTag {
        self.tag
    }
}

impl NoteDetailsAndTag {
    /// Internal constructor used when converting from native types.
    pub(crate) fn new_internal(note_details: NoteDetails, tag: NoteTag) -> NoteDetailsAndTag {
        NoteDetailsAndTag { note_details, tag }
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

