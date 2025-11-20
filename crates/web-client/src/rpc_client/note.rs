use wasm_bindgen::prelude::wasm_bindgen;

use crate::models::input_note::InputNote;
use crate::models::note_id::NoteId;
use crate::models::note_inclusion_proof::NoteInclusionProof;
use crate::models::note_metadata::NoteMetadata;
use crate::models::note_type::NoteType;

/// Represents a note fetched from a Miden node via RPC.
#[derive(Clone)]
#[wasm_bindgen]
pub struct FetchedNote {
    note_id: NoteId,
    metadata: NoteMetadata,
    inclusion_proof: Option<NoteInclusionProof>,
    input_note: Option<InputNote>,
}

#[wasm_bindgen]
impl FetchedNote {
    /// Create a note with an optional `InputNote` and optional `NoteInclusionProof`.
    #[wasm_bindgen(constructor)]
    pub fn new(
        note_id: NoteId,
        metadata: NoteMetadata,
        inclusion_proof: Option<NoteInclusionProof>,
        input_note: Option<InputNote>,
    ) -> FetchedNote {
        FetchedNote {
            note_id,
            metadata,
            inclusion_proof,
            input_note,
        }
    }

    /// The unique identifier of the note.
    #[wasm_bindgen(getter)]
    #[wasm_bindgen(js_name = "noteId")]
    pub fn note_id(&self) -> NoteId {
        self.note_id
    }

    /// The note's metadata, including sender, tag, and other properties.
    /// Available for both private and public notes.
    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> NoteMetadata {
        self.metadata
    }

    /// The note's inclusion proof.
    ///
    /// Returns the inclusion proof either directly (for private notes) or via the `input_note`
    /// (for public notes).
    #[wasm_bindgen(getter)]
    #[wasm_bindgen(js_name = "inclusionProof")]
    pub fn inclusion_proof(&self) -> Option<NoteInclusionProof> {
        self.inclusion_proof.clone().or_else(|| self.input_note.as_ref()?.proof())
    }

    /// The full [`InputNote`] with inclusion proof.
    ///
    /// For public notes, it contains the complete note data and inclusion proof.
    /// For private notes, it will be ``None`.
    #[wasm_bindgen(getter)]
    #[wasm_bindgen(js_name = "inputNote")]
    pub fn input_note(&self) -> Option<InputNote> {
        self.input_note.clone()
    }

    #[wasm_bindgen(getter)]
    #[wasm_bindgen(js_name = "noteType")]
    pub fn note_type(&self) -> NoteType {
        self.metadata.note_type()
    }
}
