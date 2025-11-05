use miden_client::note::NoteDetails as NativeNoteDetails;
use wasm_bindgen::prelude::*;

use super::note_assets::NoteAssets;
use super::note_id::NoteId;
use super::note_recipient::NoteRecipient;

/// Note details used in transaction requests and note stores.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteDetails(NativeNoteDetails);

#[wasm_bindgen]
impl NoteDetails {
    #[wasm_bindgen(constructor)]
    /// Creates new note details from assets and recipient.
    pub fn new(note_assets: &NoteAssets, note_recipient: &NoteRecipient) -> NoteDetails {
        NoteDetails(NativeNoteDetails::new(note_assets.into(), note_recipient.into()))
    }

    /// Returns the note identifier.
    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    /// Returns the assets locked in the note.
    pub fn assets(&self) -> NoteAssets {
        self.0.assets().into()
    }

    /// Returns the note recipient descriptor.
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

/// Convenience wrapper for passing arrays of [`NoteDetails`].
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteDetailsArray(Vec<NoteDetails>);

#[wasm_bindgen]
impl NoteDetailsArray {
    #[wasm_bindgen(constructor)]
    /// Creates an array of note details.
    pub fn new(note_details_array: Option<Vec<NoteDetails>>) -> NoteDetailsArray {
        let note_details_array = note_details_array.unwrap_or_default();
        NoteDetailsArray(note_details_array)
    }

    /// Pushes another note details entry into the collection.
    pub fn push(&mut self, note_details: &NoteDetails) {
        self.0.push(note_details.clone());
    }
}

impl From<NoteDetailsArray> for Vec<NativeNoteDetails> {
    fn from(note_details_array: NoteDetailsArray) -> Self {
        note_details_array.0.into_iter().map(Into::into).collect()
    }
}

impl From<&NoteDetailsArray> for Vec<NativeNoteDetails> {
    fn from(note_details_array: &NoteDetailsArray) -> Self {
        note_details_array.0.iter().map(Into::into).collect()
    }
}
