use miden_client::note::NoteMetadata as NativeNoteMetadata;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::felt::Felt;
use super::note_execution_hint::NoteExecutionHint;
use super::note_tag::NoteTag;
use super::note_type::NoteType;

/// Metadata describing the origin and policy for a note.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteMetadata(NativeNoteMetadata);

#[wasm_bindgen]
impl NoteMetadata {
    #[wasm_bindgen(constructor)]
    /// Creates note metadata from sender, type, tag, execution hint, and optional auxiliary data.
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

    /// Returns the sender account identifier.
    pub fn sender(&self) -> AccountId {
        self.0.sender().into()
    }

    /// Returns the tag describing the note use case.
    pub fn tag(&self) -> NoteTag {
        self.0.tag().into()
    }

    #[wasm_bindgen(js_name = "noteType")]
    /// Returns the note type (private, encrypted, public).
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
