use miden_client::crypto::MerklePath as NativeMerklePath;
use miden_client::note::NoteInclusionProof as NativeNoteInclusionProof;

use super::merkle_path::MerklePath;
use super::note_location::NoteLocation;
use crate::prelude::*;

/// Contains the data required to prove inclusion of a note in the canonical chain.
#[bindings]
#[derive(Clone)]
pub struct NoteInclusionProof(NativeNoteInclusionProof);

#[bindings]
impl NoteInclusionProof {
    pub fn location(&self) -> NoteLocation {
        self.0.location().into()
    }

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
