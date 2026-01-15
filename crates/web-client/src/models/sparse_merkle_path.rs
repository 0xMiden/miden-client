use miden_protocol::crypto::merkle::SparseMerklePath as NativeSparseMerklePath;
use wasm_bindgen::prelude::*;

use super::word::Word;

/// Represents a sparse Merkle path.
#[derive(Clone)]
#[wasm_bindgen]
pub struct SparseMerklePath(NativeSparseMerklePath);

#[wasm_bindgen]
impl SparseMerklePath {
    /// Returns the empty nodes mask used by this path.
    #[wasm_bindgen(js_name = "emptyNodesMask")]
    pub fn empty_nodes_mask(&self) -> u64 {
        let (mask, _siblings) = self.0.clone().into_parts();
        mask
    }

    /// Returns the sibling nodes that make up the path.
    pub fn nodes(&self) -> Vec<Word> {
        let (_mask, siblings) = self.0.clone().into_parts();
        siblings.into_iter().map(Into::into).collect()
    }

    /// Verifies the path against a root.
    pub fn verify(&self, index: u64, node: &Word, root: &Word) -> bool {
        self.0.verify(index, node.clone().into(), &root.clone().into()).is_ok()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeSparseMerklePath> for SparseMerklePath {
    fn from(native_path: NativeSparseMerklePath) -> Self {
        SparseMerklePath(native_path)
    }
}

impl From<&NativeSparseMerklePath> for SparseMerklePath {
    fn from(native_path: &NativeSparseMerklePath) -> Self {
        SparseMerklePath(native_path.clone())
    }
}
