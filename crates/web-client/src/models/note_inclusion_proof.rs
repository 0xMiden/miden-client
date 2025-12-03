use miden_client::crypto::MerklePath as NativeMerklePath;
use miden_client::note::NoteInclusionProof as NativeNoteInclusionProof;
use wasm_bindgen::prelude::*;

use super::merkle_path::MerklePath;
use super::note_location::NoteLocation;

/// Proof that a note commitment exists at a specific position in the note tree.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteInclusionProof(NativeNoteInclusionProof);

#[wasm_bindgen]
impl NoteInclusionProof {
    /// Returns the location of the note within the tree.
    pub fn location(&self) -> NoteLocation {
        self.0.location().into()
    }

    /// Returns the Merkle authentication path for the note.
    #[wasm_bindgen(js_name = "notePath")]
    pub fn note_path(&self) -> MerklePath {
        NativeMerklePath::from(self.0.note_path().clone()).into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteInclusionProof> for NoteInclusionProof {
    fn from(native_proof: NativeNoteInclusionProof) -> Self {
        NoteInclusionProof(native_proof)
    }
}

impl From<&NativeNoteInclusionProof> for NoteInclusionProof {
    fn from(native_proof: &NativeNoteInclusionProof) -> Self {
        NoteInclusionProof(native_proof.clone())
    }
}
impl From<NoteInclusionProof> for NativeNoteInclusionProof {
    fn from(proof: NoteInclusionProof) -> Self {
        proof.0
    }
}
