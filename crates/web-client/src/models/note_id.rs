use miden_client::note::NoteId as NativeNoteId;
use wasm_bindgen::prelude::*;

use super::word::Word;
use crate::js_error_with_context;

/// Identifier of a note, derived from recipient and assets.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct NoteId(NativeNoteId);

#[wasm_bindgen]
impl NoteId {
    #[wasm_bindgen(constructor)]
    /// Constructs a note ID from recipient and asset commitments.
    pub fn new(recipient_digest: &Word, asset_commitment_digest: &Word) -> NoteId {
        NoteId(NativeNoteId::new(recipient_digest.into(), asset_commitment_digest.into()))
    }

    #[wasm_bindgen(js_name = "fromHex")]
    /// Parses a note ID from its hex encoding.
    pub fn from_hex(hex: &str) -> Result<NoteId, JsValue> {
        let native_note_id = NativeNoteId::try_from_hex(hex)
            .map_err(|err| js_error_with_context(err, "error instantiating NoteId from hex"))?;
        Ok(NoteId(native_note_id))
    }

    #[wasm_bindgen(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    /// Returns the canonical string representation of the note ID.
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteId> for NoteId {
    fn from(native_note_id: NativeNoteId) -> Self {
        NoteId(native_note_id)
    }
}

impl From<&NativeNoteId> for NoteId {
    fn from(native_note_id: &NativeNoteId) -> Self {
        NoteId(*native_note_id)
    }
}

impl From<NoteId> for NativeNoteId {
    fn from(note_id: NoteId) -> Self {
        note_id.0
    }
}

impl From<&NoteId> for NativeNoteId {
    fn from(note_id: &NoteId) -> Self {
        note_id.0
    }
}
