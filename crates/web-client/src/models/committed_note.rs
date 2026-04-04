use miden_client::note::NoteMetadata as NativeNoteMetadata;
use miden_client::rpc::domain::note::CommittedNote as NativeCommittedNote;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::note_id::NoteId;
use super::note_inclusion_proof::NoteInclusionProof;
use super::note_metadata::NoteMetadata;
use super::note_type::NoteType;
use super::sparse_merkle_path::SparseMerklePath;

/// Represents a note committed on chain.
#[derive(Clone)]
#[wasm_bindgen]
pub struct CommittedNote(NativeCommittedNote);

#[wasm_bindgen]
impl CommittedNote {
    /// Returns the note ID.
    #[wasm_bindgen(js_name = "noteId")]
    pub fn note_id(&self) -> NoteId {
        (*self.0.note_id()).into()
    }

    /// Returns the note index in the block's note tree.
    #[wasm_bindgen(js_name = "noteIndex")]
    pub fn note_index(&self) -> u16 {
        self.0.inclusion_proof().location().block_note_tree_index()
    }

    /// Returns the inclusion path for the note in the block's note tree.
    #[wasm_bindgen(js_name = "inclusionPath")]
    pub fn inclusion_path(&self) -> SparseMerklePath {
        self.0.inclusion_proof().note_path().into()
    }

    /// Returns the note type (public, private, etc.).
    #[wasm_bindgen(js_name = "noteType")]
    pub fn note_type(&self) -> NoteType {
        self.0.note_type().into()
    }

    /// Returns the note sender, even when only header metadata is available.
    pub fn sender(&self) -> AccountId {
        self.0.committed_metadata().sender().into()
    }

    /// Returns the note tag.
    pub fn tag(&self) -> u32 {
        self.0.tag().as_u32()
    }

    /// Returns the note metadata.
    ///
    /// If only metadata headers are available, the returned metadata contains
    /// the sender, note type, and tag without attachment payload.
    pub fn metadata(&self) -> NoteMetadata {
        self.0.metadata().map_or_else(
            || {
                NativeNoteMetadata::new(self.0.committed_metadata().sender(), self.0.note_type())
                    .with_tag(self.0.tag())
                    .into()
            },
            Into::into,
        )
    }

    /// Returns the full note metadata when the attachment payload is available.
    #[wasm_bindgen(js_name = "fullMetadata")]
    pub fn full_metadata(&self) -> Option<NoteMetadata> {
        self.0.metadata().map(Into::into)
    }

    /// Returns the inclusion proof for this note.
    #[wasm_bindgen(js_name = "inclusionProof")]
    pub fn inclusion_proof(&self) -> NoteInclusionProof {
        self.0.inclusion_proof().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeCommittedNote> for CommittedNote {
    fn from(native_note: NativeCommittedNote) -> Self {
        CommittedNote(native_note)
    }
}

impl From<&NativeCommittedNote> for CommittedNote {
    fn from(native_note: &NativeCommittedNote) -> Self {
        CommittedNote(native_note.clone())
    }
}
