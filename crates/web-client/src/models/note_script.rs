use miden_client::PrettyPrint;
use miden_client::note::NoteScript as NativeNoteScript;
use miden_standards::note::StandardNote;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use super::word::Word;
use crate::models::package::Package;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// An executable program of a note.
///
/// A note's script represents a program which must be executed for a note to be consumed. As such
/// it defines the rules and side effects of consuming a given note.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteScript(NativeNoteScript);

#[wasm_bindgen]
impl NoteScript {
    /// Pretty-prints the MAST source for this script.
    #[wasm_bindgen(js_name = toString)]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_pretty_string()
    }

    /// Serializes the script into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a script from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<NoteScript, JsValue> {
        deserialize_from_uint8array::<NativeNoteScript>(bytes).map(NoteScript)
    }

    /// Returns the well-known P2ID script.
    pub fn p2id() -> Self {
        StandardNote::P2ID.script().into()
    }

    /// Returns the well-known P2IDE script (P2ID with execution hint).
    pub fn p2ide() -> Self {
        StandardNote::P2IDE.script().into()
    }

    /// Returns the well-known SWAP script.
    pub fn swap() -> Self {
        StandardNote::SWAP.script().into()
    }

    /// Returns the MAST root of this script.
    pub fn root(&self) -> Word {
        self.0.root().into()
    }

    /// Creates a `NoteScript` from the given `Package`.
    /// Throws if the package is invalid.
    #[wasm_bindgen(js_name = "fromPackage")]
    pub fn from_package(package: &Package) -> Result<NoteScript, JsValue> {
        let program = package.as_program()?;
        let native_note_script = NativeNoteScript::new(program.into());
        Ok(native_note_script.into())
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
