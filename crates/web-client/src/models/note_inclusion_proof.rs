use js_export_macro::js_export;
use miden_client::crypto::MerklePath as NativeMerklePath;
use miden_client::note::NoteInclusionProof as NativeNoteInclusionProof;

use super::merkle_path::MerklePath;
use super::note_location::NoteLocation;

/// Contains the data required to prove inclusion of a note in the canonical chain.
#[derive(Clone)]
#[js_export]
pub struct NoteInclusionProof(NativeNoteInclusionProof);

#[js_export]
impl NoteInclusionProof {
    /// Returns the location of the note within the tree.
    pub fn location(&self) -> NoteLocation {
        self.0.location().into()
    }

    /// Returns the Merkle authentication path for the note.
    #[js_export(js_name = "notePath")]
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

impl_napi_from_value!(NoteInclusionProof);
