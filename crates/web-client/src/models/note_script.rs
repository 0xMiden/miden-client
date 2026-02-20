use miden_client::PrettyPrint;
use miden_client::note::NoteScript as NativeNoteScript;
use miden_standards::note::StandardNote;
use crate::prelude::*;

use super::word::Word;

/// An executable program of a note.
///
/// A note's script represents a program which must be executed for a note to be consumed. As such
/// it defines the rules and side effects of consuming a given note.
#[bindings]
#[derive(Clone)]
pub struct NoteScript(pub(crate) NativeNoteScript);

#[bindings]
impl NoteScript {
    pub fn serialize(&self) -> JsBytes {
        platform::serialize_to_bytes(&self.0)
    }

    #[bindings(factory)]
    pub fn deserialize(bytes: JsBytes) -> JsResult<NoteScript> {
        platform::deserialize_from_bytes::<NativeNoteScript>(&bytes).map(NoteScript)
    }

    pub fn root(&self) -> Word {
        self.0.root().into()
    }

    #[bindings(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string_js(&self) -> String {
        self.0.to_pretty_string()
    }

    #[bindings(factory)]
    pub fn p2id() -> Self {
        StandardNote::P2ID.script().into()
    }

    #[bindings(factory)]
    pub fn p2ide() -> Self {
        StandardNote::P2IDE.script().into()
    }

    #[bindings(factory)]
    pub fn swap() -> Self {
        StandardNote::SWAP.script().into()
    }
}

// wasm-specific: from_package
#[cfg(feature = "wasm")]
impl NoteScript {
    /// Creates a `NoteScript` from the given `Package`.
    /// Throws if the package is invalid.
    
    pub fn from_package(package: &super::package::Package) -> JsResult<NoteScript> {
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
