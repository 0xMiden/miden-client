use miden_client::note::NoteFile as NativeNoteFile;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[wasm_bindgen]
pub struct NoteFile(NativeNoteFile);

#[wasm_bindgen]
impl NoteFile {
    /// Serializes the `NoteFile` into a byte array
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a byte array into a `NoteFile`
    pub fn deserialize(bytes: &Uint8Array) -> Result<NoteFile, JsValue> {
        let native_note_file: NativeNoteFile = deserialize_from_uint8array(bytes)?;
        Ok(Self(native_note_file))
    }
}

impl From<NativeNoteFile> for NoteFile {
    fn from(native_note_file: NativeNoteFile) -> Self {
        Self(native_note_file)
    }
}

impl From<NoteFile> for NativeNoteFile {
    fn from(note_file: NoteFile) -> Self {
        note_file.0
    }
}
