use miden_client::note::{NoteDetails as NativeNoteDetails, NoteId as NativeNoteId};
use miden_client::notes::NoteFile as NativeNoteFile;
use miden_client::{Deserializable, Serializable};
use wasm_bindgen::prelude::*;

use super::input_note::InputNote;
use super::note::Note;
use super::output_note::OutputNote;
use crate::js_error_with_context;
use crate::models::note_details::NoteDetails;
use crate::models::note_id::NoteId;
use crate::models::note_inclusion_proof::NoteInclusionProof;
use crate::models::note_tag::NoteTag;

/// A serialized representation of a note.
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

    /// Returns the note ID for any `NoteFile` variant.
    #[wasm_bindgen(js_name = "noteId")]
    pub fn note_id(&self) -> NoteId {
        match &self.inner {
            NativeNoteFile::NoteId(note_id) => (*note_id).into(),
            NativeNoteFile::NoteDetails { details, .. } => details.id().into(),
            NativeNoteFile::NoteWithProof(note, _) => note.id().into(),
        }
    }

    /// Returns the note details if present.
    #[wasm_bindgen(js_name = "noteDetails")]
    pub fn note_details(&self) -> Option<NoteDetails> {
        match &self.inner {
            NativeNoteFile::NoteDetails { details, .. } => Some(details.into()),
            _ => None,
        }
    }

    /// Returns the full note when the file includes it.
    pub fn note(&self) -> Option<Note> {
        match &self.inner {
            NativeNoteFile::NoteWithProof(note, _) => Some(note.into()),
            _ => None,
        }
    }

    /// Returns the inclusion proof if present.
    #[wasm_bindgen(js_name = "inclusionProof")]
    pub fn inclusion_proof(&self) -> Option<NoteInclusionProof> {
        match &self.inner {
            NativeNoteFile::NoteWithProof(_, proof) => Some(proof.into()),
            _ => None,
        }
    }

    /// Returns the after-block hint when present.
    #[wasm_bindgen(js_name = "afterBlockNum")]
    pub fn after_block_num(&self) -> Option<u32> {
        match &self.inner {
            NativeNoteFile::NoteDetails { after_block_num, .. } => Some(after_block_num.as_u32()),
            _ => None,
        }
    }

    /// Returns the note tag hint when present.
    #[wasm_bindgen(js_name = "noteTag")]
    pub fn note_tag(&self) -> Option<NoteTag> {
        match &self.inner {
            NativeNoteFile::NoteDetails { tag, .. } => tag.map(Into::into),
            _ => None,
        }
    }

    /// Returns the note nullifier when present.
    pub fn nullifier(&self) -> Option<String> {
        match &self.inner {
            NativeNoteFile::NoteDetails { details, .. } => Some(details.nullifier().to_hex()),
            NativeNoteFile::NoteWithProof(note, _) => Some(note.nullifier().to_hex()),
            NativeNoteFile::NoteId(_) => None,
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

    /// Creates a `NoteFile` from an input note, preserving proof when available.
    #[wasm_bindgen(js_name = fromInputNote)]
    pub fn from_input_note(note: &InputNote) -> Self {
        if let Some(inclusion_proof) = note.proof() {
            Self {
                inner: NativeNoteFile::NoteWithProof(note.note().into(), inclusion_proof.into()),
            }
        } else {
            let assets = note.note().assets();
            let recipient = note.note().recipient();
            let details = NativeNoteDetails::new(assets.into(), recipient.into());
            Self { inner: details.into() }
        }
    }

    /// Creates a `NoteFile` from an output note, choosing details when present.
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

    /// Creates a `NoteFile` from note details.
    #[wasm_bindgen(js_name = fromNoteDetails)]
    pub fn from_note_details(note_details: &NoteDetails) -> Self {
        note_details.into()
    }

    /// Creates a `NoteFile` from a note ID.
    #[wasm_bindgen(js_name = fromNoteId)]
    pub fn from_note_id(note_details: &NoteId) -> Self {
        note_details.into()
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

impl From<&NoteDetails> for NoteFile {
    fn from(details: &NoteDetails) -> Self {
        let note_details: NativeNoteDetails = details.into();
        Self { inner: note_details.into() }
    }
}

impl From<&NoteId> for NoteFile {
    fn from(note_id: &NoteId) -> Self {
        let note_id: NativeNoteId = note_id.into();
        NoteFile { inner: note_id.into() }
    }
}
