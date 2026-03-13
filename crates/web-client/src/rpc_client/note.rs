use js_export_macro::js_export;
use miden_client::note::{
    NoteHeader as NativeNoteHeader,
    NoteInclusionProof as NativeNoteInclusionProof,
};

use crate::models::NoteType;
use crate::models::input_note::InputNote;
use crate::models::note::Note;
use crate::models::note_header::NoteHeader;
use crate::models::note_id::NoteId;
use crate::models::note_inclusion_proof::NoteInclusionProof;
use crate::models::note_metadata::NoteMetadata;

/// Wrapper for a note fetched over RPC.
///
/// It contains the note header and inclusion proof. The note details are only present for
/// public notes.
#[derive(Clone)]
#[js_export]
pub struct FetchedNote {
    header: NoteHeader,
    inclusion_proof: NoteInclusionProof,
    note: Option<Note>,
}

// Internal methods accessible from Rust code (not processed by napi/wasm_bindgen).
impl FetchedNote {
    /// The unique identifier of the note (internal Rust access).
    pub(crate) fn note_id(&self) -> NoteId {
        self.header.id()
    }

    /// The note's inclusion proof (internal Rust access).
    pub(crate) fn inclusion_proof(&self) -> NoteInclusionProof {
        self.inclusion_proof.clone()
    }

    /// Returns whether the note is private, encrypted, or public (internal Rust access).
    pub(crate) fn note_type(&self) -> NoteType {
        self.header.metadata().note_type()
    }

    /// The full note data (internal Rust access).
    pub(crate) fn note(&self) -> Option<Note> {
        self.note.clone()
    }
}

#[js_export]
impl FetchedNote {
    /// Create a `FetchedNote` with an optional [`Note`].
    #[js_export(constructor)]
    pub fn new(
        note_id: NoteId,
        metadata: NoteMetadata,
        inclusion_proof: NoteInclusionProof,
        note: Option<Note>,
    ) -> FetchedNote {
        // Convert note_id and metadata to NativeNoteHeader, then to web NoteHeader
        let native_note_id = note_id.into();
        let native_metadata = metadata.into();
        let native_header = NativeNoteHeader::new(native_note_id, native_metadata);
        let header = native_header.into();
        FetchedNote { header, inclusion_proof, note }
    }

    // GETTERS
    // --------------------------------------------------------------------------------------------

    /// The unique identifier of the note.
    #[js_export(getter, js_name = "noteId")]
    pub fn get_note_id(&self) -> NoteId {
        self.header.id()
    }

    /// The note's metadata, including sender, tag, and other properties.
    /// Available for both private and public notes.
    #[js_export(getter)]
    pub fn metadata(&self) -> NoteMetadata {
        self.header.metadata()
    }

    /// The note's header, containing the ID and metadata.
    #[js_export(getter)]
    pub fn header(&self) -> NoteHeader {
        self.header.clone()
    }

    /// The full [`Note`] data.
    ///
    /// For public notes, it contains the complete note data.
    /// For private notes, it will be undefined.
    #[js_export(getter, js_name = "note")]
    pub fn get_note(&self) -> Option<Note> {
        self.note.clone()
    }

    /// The note's inclusion proof.
    ///
    /// Contains the data required to prove inclusion of the note in the canonical chain.
    #[js_export(getter, js_name = "inclusionProof")]
    pub fn get_inclusion_proof(&self) -> NoteInclusionProof {
        self.inclusion_proof.clone()
    }

    /// Returns whether the note is private, encrypted, or public.
    #[js_export(getter, js_name = "noteType")]
    pub fn get_note_type(&self) -> NoteType {
        self.header.metadata().note_type()
    }

    // CONVERSIONS
    // --------------------------------------------------------------------------------------------

    /// Returns an [`InputNote`] when the fetched note is public.
    ///
    /// Returns `undefined` when the note body is missing (e.g. private notes); in that case build
    /// an `InputNote` manually using the inclusion proof and note data obtained elsewhere.
    #[js_export(js_name = "asInputNote")]
    pub fn as_input_note(&self) -> Option<InputNote> {
        self.note().map(|note| InputNote::authenticated(&note, &self.inclusion_proof))
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
