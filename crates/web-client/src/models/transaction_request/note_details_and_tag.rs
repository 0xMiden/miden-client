use crate::models::miden_arrays::NoteDetailsAndTagArray;
use crate::models::note_details::NoteDetails;
use crate::models::note_tag::NoteTag;
use miden_objects::note::{NoteDetails as NativeNoteDetails, NoteTag as NativeNoteTag};
use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteDetailsAndTag {
    note_details: NoteDetails,
    tag: NoteTag,
}

#[wasm_bindgen]
impl NoteDetailsAndTag {
    #[wasm_bindgen(constructor)]
    pub fn new(note_details: NoteDetails, tag: NoteTag) -> NoteDetailsAndTag {
        NoteDetailsAndTag { note_details, tag }
    }

    #[wasm_bindgen(getter)]
    #[wasm_bindgen(js_name = "noteDetails")]
    pub fn note_details(&self) -> NoteDetails {
        self.note_details.clone()
    }

    #[wasm_bindgen(getter)]
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
        note_details_and_tag_array.__inner.into_iter().map(Into::into).collect()
    }
}

impl From<&NoteDetailsAndTagArray> for Vec<(NativeNoteDetails, NativeNoteTag)> {
    fn from(note_details_and_tag_array: &NoteDetailsAndTagArray) -> Self {
        note_details_and_tag_array.__inner.iter().map(Into::into).collect()
    }
}
