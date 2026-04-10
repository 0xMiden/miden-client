use js_export_macro::js_export;
use miden_client::crypto::SparseMerklePath as NativeSparseMerklePath;

use super::word::Word;
use crate::platform::{JsU64, js_u64_to_u64};

/// Represents a sparse Merkle path.
#[derive(Clone)]
#[js_export]
pub struct SparseMerklePath(NativeSparseMerklePath);

#[js_export]
impl SparseMerklePath {
    /// Returns the empty nodes mask used by this path.
    #[js_export(js_name = "emptyNodesMask")]
    pub fn empty_nodes_mask(&self) -> JsU64 {
        let (mask, _siblings) = self.0.clone().into_parts();
        mask as JsU64
    }

    /// Returns the sibling nodes that make up the path.
    pub fn nodes(&self) -> Vec<Word> {
        let (_mask, siblings) = self.0.clone().into_parts();
        siblings.into_iter().map(Into::into).collect()
    }

    /// Verifies the path against a root.
    pub fn verify(&self, index: JsU64, node: &Word, root: &Word) -> bool {
        self.0
            .verify(js_u64_to_u64(index), node.clone().into(), &root.clone().into())
            .is_ok()
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
