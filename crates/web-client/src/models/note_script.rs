use miden_client::note::{NoteScript as NativeNoteScript, WellKnownNote};
use miden_objects::PrettyPrint;
use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteScript(NativeNoteScript);

#[wasm_bindgen]
impl NoteScript {
    /// Print the MAST source for this script.
    #[wasm_bindgen(js_name = toString)]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_pretty_string()
    }

    pub fn p2id() -> Self {
        WellKnownNote::P2ID.script().into()
    }

    pub fn p2ide() -> Self {
        WellKnownNote::P2IDE.script().into()
    }

    pub fn swap() -> Self {
        WellKnownNote::SWAP.script().into()
    }
}
// CONVERSIONS
// ================================================================================================

impl From<NativeNoteScript> for NoteScript {
    fn from(native_note_script: NativeNoteScript) -> Self {
        NoteScript(native_note_script)
    }
}

impl From<&NativeNoteScript> for NoteScript {
    fn from(native_note_script: &NativeNoteScript) -> Self {
        NoteScript(native_note_script.clone())
    }
}

impl From<NoteScript> for NativeNoteScript {
    fn from(note_script: NoteScript) -> Self {
        note_script.0
    }
}

impl From<&NoteScript> for NativeNoteScript {
    fn from(note_script: &NoteScript) -> Self {
        note_script.0.clone()
    }
}
