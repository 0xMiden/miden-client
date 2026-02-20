use miden_client::crypto::SparseMerklePath as NativeSparseMerklePath;

use crate::prelude::*;

use super::word::Word;

/// Represents a sparse Merkle path.
#[bindings]
#[derive(Clone)]
pub struct SparseMerklePath(NativeSparseMerklePath);

#[bindings]
impl SparseMerklePath {
    /// Returns the sibling nodes that make up the path.
    pub fn nodes(&self) -> Vec<Word> {
        let (_mask, siblings) = self.0.clone().into_parts();
        siblings.into_iter().map(Into::into).collect()
    }

    /// Returns the empty nodes mask used by this path.
    #[bindings]
    pub fn empty_nodes_mask(&self) -> i64 {
        let (mask, _siblings) = self.0.clone().into_parts();
        mask as i64
    }

    /// Verifies the path against a root.
    pub fn verify(&self, index: i64, node: &Word, root: &Word) -> bool {
        self.0.verify(index as u64, node.clone().into(), &root.clone().into()).is_ok()
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
