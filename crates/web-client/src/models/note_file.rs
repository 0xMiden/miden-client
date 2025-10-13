use miden_client::note::NoteDetails as NativeNoteDetails;
use miden_client::notes::NoteFile as NativeNoteFile;
use miden_client::{Deserializable, Serializable};
use wasm_bindgen::prelude::*;

use super::input_note::InputNote;
use super::output_note::OutputNote;
use crate::js_error_with_context;

#[wasm_bindgen(inspectable)]
pub struct NoteFile {
    pub(crate) inner: NativeNoteFile,
}

#[wasm_bindgen]
impl NoteFile {
    /// Returns this `NoteFile`'s types.
    #[wasm_bindgen(js_name = noteType)]
    pub fn note_type(&self) -> String {
        match &self.inner {
            NativeNoteFile::NoteId(_) => "NoteId".to_owned(),
            NativeNoteFile::NoteDetails { .. } => "NoteDetails".to_owned(),
            NativeNoteFile::NoteWithProof(..) => "NoteWithProof".to_owned(),
        }
    }

    /// Turn a notefile into its byte representation.
    #[wasm_bindgen(js_name = serialize)]
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = vec![];
        self.inner.write_into(&mut buffer);
        buffer
    }

    /// Given a valid byte representation of a `NoteFile`,
    /// return it as a struct.
    #[wasm_bindgen(js_name = deserialize)]
    pub fn deserialize(bytes: &[u8]) -> Result<Self, JsValue> {
        let deserialized = NativeNoteFile::read_from_bytes(bytes)
            .map_err(|err| js_error_with_context(err, "notefile deserialization failed"))?;
        Ok(Self { inner: deserialized })
    }

    #[wasm_bindgen(js_name = fromInputNote)]
    pub fn from_input_note(note: &InputNote) -> Self {
        match note.proof() {
            Some(inclusion_proof) => Self {
                inner: NativeNoteFile::NoteWithProof(note.note().into(), inclusion_proof.into()),
            },
            None => {
                let assets = note.note().assets();
                let recipient = note.note().recipient();
                let details = NativeNoteDetails::new(assets.into(), recipient.into());
                Self { inner: details.into() }
            },
        }
    }

    #[wasm_bindgen(js_name = fromOutputNote)]
    pub fn from_output_note(note: &OutputNote) -> Self {
        let native_note = note.note();
        match (native_note.assets(), native_note.recipient()) {
            (Some(assets), Some(recipient)) => {
                let details = NativeNoteDetails::new(assets.clone(), recipient.clone());
                Self { inner: details.into() }
            },
            _ => Self { inner: native_note.id().into() },
        }
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteFile> for NoteFile {
    fn from(note_file: NativeNoteFile) -> Self {
        NoteFile { inner: note_file }
    }
}

impl From<NoteFile> for NativeNoteFile {
    fn from(note_file: NoteFile) -> Self {
        note_file.inner
    }
}
