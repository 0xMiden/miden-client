use miden_client::note::{
    NoteHeader as NativeNoteHeader,
    NoteInclusionProof as NativeNoteInclusionProof,
};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::models::note::Note;
use crate::models::note_header::NoteHeader;
use crate::models::note_id::NoteId;
use crate::models::note_inclusion_proof::NoteInclusionProof;
use crate::models::note_metadata::NoteMetadata;
use crate::models::note_type::NoteType;

/// Represents a note fetched from a Miden node via RPC.
#[derive(Clone)]
#[wasm_bindgen]
pub struct FetchedNote {
    header: NoteHeader,
    note: Option<Note>,
    inclusion_proof: NoteInclusionProof,
}

#[wasm_bindgen]
impl FetchedNote {
    /// Create a `FetchedNote` with an optional [`Note`].
    #[wasm_bindgen(constructor)]
    pub fn new(
        note_id: NoteId,
        metadata: NoteMetadata,
        note: Option<Note>,
        inclusion_proof: NoteInclusionProof,
    ) -> FetchedNote {
        // Convert note_id and metadata to NativeNoteHeader, then to web NoteHeader
        let native_note_id = note_id.into();
        let native_metadata = metadata.into();
        let native_header = NativeNoteHeader::new(native_note_id, native_metadata);
        let header = native_header.into();
        FetchedNote { header, note, inclusion_proof }
    }

    /// The unique identifier of the note.
    #[wasm_bindgen(getter)]
    #[wasm_bindgen(js_name = "noteId")]
    pub fn note_id(&self) -> NoteId {
        self.header.id()
    }

    /// The note's metadata, including sender, tag, and other properties.
    /// Available for both private and public notes.
    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> NoteMetadata {
        self.header.metadata()
    }

    /// The note's header, containing the ID and metadata.
    #[wasm_bindgen(getter)]
    pub fn header(&self) -> NoteHeader {
        self.header.clone()
    }

    /// The full [`Note`] data.
    ///
    /// For public notes, it contains the complete note data.
    /// For private notes, it will be `None`.
    #[wasm_bindgen(getter)]
    pub fn note(&self) -> Option<Note> {
        self.note.clone()
    }

    /// The note's inclusion proof.
    ///
    /// Contains the data required to prove inclusion of the note in the canonical chain.
    #[wasm_bindgen(getter)]
    #[wasm_bindgen(js_name = "inclusionProof")]
    pub fn inclusion_proof(&self) -> NoteInclusionProof {
        self.inclusion_proof.clone()
    }

    #[wasm_bindgen(getter)]
    #[wasm_bindgen(js_name = "noteType")]
    pub fn note_type(&self) -> NoteType {
        self.header.metadata().note_type()
    }
}

impl FetchedNote {
    /// Create a `FetchedNote` from a native `NoteHeader` (internal use).
    pub(super) fn from_header(
        header: NativeNoteHeader,
        note: Option<Note>,
        inclusion_proof: NativeNoteInclusionProof,
    ) -> Self {
        FetchedNote {
            header: header.into(),
            note,
            inclusion_proof: inclusion_proof.into(),
        }
    }
}
