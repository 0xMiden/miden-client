use miden_client::note::NoteMetadata as NativeNoteMetadata;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::felt::Felt;
use super::note_execution_hint::NoteExecutionHint;
use super::note_tag::NoteTag;
use super::note_type::NoteType;

/// Public metadata describing a note (sender, type, tag, and execution hint).
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteMetadata(NativeNoteMetadata);

#[wasm_bindgen]
impl NoteMetadata {
    /// Creates metadata for a note.
    #[wasm_bindgen(constructor)]
    pub fn new(
        sender: &AccountId,
        note_type: NoteType,
        note_tag: &NoteTag,
        note_execution_hint: &NoteExecutionHint,
        aux: Option<Felt>, // Create an OptionFelt type so user has choice to consume or not
    ) -> NoteMetadata {
        let native_note_metadata = NativeNoteMetadata::new(
            sender.into(),
            note_type.into(),
            note_tag.into(),
            note_execution_hint.into(),
            aux.map_or(miden_client::Felt::default(), Into::into),
        )
        .unwrap();
        NoteMetadata(native_note_metadata)
    }

    /// Returns the account that created the note.
    pub fn sender(&self) -> AccountId {
        self.0.sender().into()
    }

    /// Returns the tag associated with the note.
    pub fn tag(&self) -> NoteTag {
        self.0.tag().into()
    }

    /// Returns whether the note is private, encrypted, or public.
    #[wasm_bindgen(js_name = "noteType")]
    pub fn note_type(&self) -> NoteType {
        self.0.note_type().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteMetadata> for NoteMetadata {
    fn from(native_note_metadata: NativeNoteMetadata) -> Self {
        NoteMetadata(native_note_metadata)
    }
}

impl From<&NativeNoteMetadata> for NoteMetadata {
    fn from(native_note_metadata: &NativeNoteMetadata) -> Self {
        NoteMetadata(*native_note_metadata)
    }
}

impl From<NoteMetadata> for NativeNoteMetadata {
    fn from(note_metadata: NoteMetadata) -> Self {
        note_metadata.0
    }
}

impl From<&NoteMetadata> for NativeNoteMetadata {
    fn from(note_metadata: &NoteMetadata) -> Self {
        note_metadata.0
    }
}
