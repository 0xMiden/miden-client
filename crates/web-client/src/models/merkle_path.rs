use miden_client::crypto::MerklePath as NativeMerklePath;
use wasm_bindgen::prelude::*;

use super::word::Word;

/// Authentication path in a Merkle tree.
#[derive(Clone)]
#[wasm_bindgen]
pub struct MerklePath(NativeMerklePath);

#[wasm_bindgen]
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
    #[wasm_bindgen(js_name = "computeRoot")]
    pub fn compute_root(&self, index: u64, node: &Word) -> Word {
        self.0.compute_root(index, node.clone().into()).unwrap().into()
    }

    /// Verifies the path against a root.
    pub fn verify(&self, index: u64, node: &Word, root: &Word) -> bool {
        self.0.verify(index, node.clone().into(), &root.clone().into()).is_ok()
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
