use miden_objects::note::NoteDetails as NativeNoteDetails;
use wasm_bindgen::prelude::*;

use super::note_assets::NoteAssets;
use super::note_id::NoteId;
use super::note_recipient::NoteRecipient;

#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteDetails(NativeNoteDetails);

#[wasm_bindgen]
impl NoteDetails {
    #[wasm_bindgen(constructor)]
    pub fn new(note_assets: &NoteAssets, note_recipient: &NoteRecipient) -> NoteDetails {
        NoteDetails(NativeNoteDetails::new(note_assets.into(), note_recipient.into()))
    }

    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    pub fn assets(&self) -> NoteAssets {
        self.0.assets().into()
    }

    pub fn recipient(&self) -> NoteRecipient {
        self.0.recipient().into()
    }
}

impl From<NoteDetails> for NativeNoteDetails {
    fn from(note_details: NoteDetails) -> Self {
        note_details.0
    }
}

impl From<&NoteDetails> for NativeNoteDetails {
    fn from(note_details: &NoteDetails) -> Self {
        note_details.0.clone()
    }
}

impl From<NativeNoteDetails> for NoteDetails {
    fn from(note_details: NativeNoteDetails) -> NoteDetails {
        NoteDetails(note_details)
    }
}

impl From<&NativeNoteDetails> for NoteDetails {
    fn from(note_details: &NativeNoteDetails) -> NoteDetails {
        NoteDetails(note_details.clone())
    }
}
