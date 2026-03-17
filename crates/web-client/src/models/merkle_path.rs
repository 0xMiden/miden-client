use js_export_macro::js_export;
use miden_client::crypto::MerklePath as NativeMerklePath;

use super::word::Word;
use crate::platform::js_u64_to_u64;

/// Represents a Merkle path.
#[derive(Clone)]
#[js_export]
pub struct MerklePath(NativeMerklePath);

#[js_export]
impl MerklePath {
    /// Computes the root given a leaf index and value.
    #[js_export(js_name = "computeRoot")]
    pub fn compute_root(&self, index: JsU64, node: &Word) -> Word {
        self.0.compute_root(js_u64_to_u64(index), node.clone().into()).unwrap().into()
    }

    /// Verifies the path against a root.
    pub fn verify(&self, index: JsU64, node: &Word, root: &Word) -> bool {
        self.0
            .verify(js_u64_to_u64(index), node.clone().into(), &root.clone().into())
            .is_ok()
    }

    /// Returns the depth of the path.
    pub fn depth(&self) -> u8 {
        self.0.depth()
    }

    /// Returns the nodes that make up the path.
    pub fn nodes(&self) -> Vec<Word> {
        self.0.nodes().iter().map(Into::into).collect()
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
