use miden_client::PrettyPrint;
use miden_client::note::{NoteScript as NativeNoteScript, WellKnownNote};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use super::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Executable script that governs when and how a note can be consumed.
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

    /// Serializes the note script into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a note script from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<NoteScript, JsValue> {
        deserialize_from_uint8array::<NativeNoteScript>(bytes).map(NoteScript)
    }

    /// Returns the well-known pay-to-identity note script.
    pub fn p2id() -> Self {
        WellKnownNote::P2ID.script().into()
    }

    /// Returns the well-known pay-to-identity with embedded conditions script.
    pub fn p2ide() -> Self {
        WellKnownNote::P2IDE.script().into()
    }

    /// Returns the built-in swap note script.
    pub fn swap() -> Self {
        WellKnownNote::SWAP.script().into()
    }

    /// Returns the MAST root hash of the script.
    pub fn root(&self) -> Word {
        self.0.root().into()
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
