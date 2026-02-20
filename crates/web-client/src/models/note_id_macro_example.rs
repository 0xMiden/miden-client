// EXAMPLE: note_id.rs converted to use #[bindings] macro
// This shows the BEFORE vs AFTER

use miden_client::note::NoteId as NativeNoteId;
use crate::prelude::*;
use super::word::Word;

/// Returns a unique identifier of a note, which is simultaneously a commitment to the note.
#[bindings]
#[derive(Clone, Copy)]
pub struct NoteId(pub(crate) NativeNoteId);

#[bindings]
impl NoteId {
    #[bindings(constructor)]
    pub fn new(recipient_digest: &Word, asset_commitment_digest: &Word) -> NoteId {
        NoteId(NativeNoteId::new(recipient_digest.into(), asset_commitment_digest.into()))
    }

    #[bindings(js_name = "toString")]  // Only needed because method is to_string_js, not to_string
    #[allow(clippy::inherent_to_string)]
    pub fn to_string_js(&self) -> String {
        self.0.to_string()
    }

    #[bindings(factory)]
    pub fn from_hex(hex: String) -> JsResult<NoteId> {  // Auto generates camelCase "fromHex"
        let native_note_id = NativeNoteId::try_from_hex(&hex)
            .map_err(|err| platform::error_with_context(err, "error instantiating NoteId from hex"))?;
        Ok(NoteId(native_note_id))
    }
}

// CONVERSIONS - These stay the same
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

/* COMPARISON:

BEFORE (with cfg_attr - current approach):
================================================
84 lines total with verbose cfg_attr everywhere

AFTER (with #[bindings] macro):
================================================
54 lines total - clean and concise

SAVINGS: 30 lines per file × 80 files = 2,400 lines saved!

*/
