use miden_client::crypto::MerklePath as NativeMerklePath;

use crate::prelude::*;

use super::word::Word;

/// Represents a Merkle path.
#[bindings]
#[derive(Clone)]
pub struct MerklePath(NativeMerklePath);

#[bindings]
impl MerklePath {
    /// Returns the depth of the path.
    pub fn depth(&self) -> u8 {
        self.0.depth()
    }

    /// Returns the nodes that make up the path.
    pub fn nodes(&self) -> Vec<Word> {
        self.0.nodes().iter().map(Into::into).collect()
    }

    /// Computes the root given a leaf index and value.
    pub fn compute_root(&self, index: i64, node: &Word) -> Word {
        self.0.compute_root(index as u64, node.clone().into()).unwrap().into()
    }

    /// Verifies the path against a root.
    pub fn verify(&self, index: i64, node: &Word, root: &Word) -> bool {
        self.0.verify(index as u64, node.clone().into(), &root.clone().into()).is_ok()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeMerklePath> for MerklePath {
    fn from(native_path: NativeMerklePath) -> Self {
        MerklePath(native_path)
    }
}

impl From<&NativeMerklePath> for MerklePath {
    fn from(native_path: &NativeMerklePath) -> Self {
        MerklePath(native_path.clone())
    }
}
