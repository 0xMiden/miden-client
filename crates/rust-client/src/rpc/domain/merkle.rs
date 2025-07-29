use alloc::vec::Vec;

use miden_objects::{
    Word,
    crypto::merkle::{Forest, MerklePath, MmrDelta, SparseMerklePath},
};

use crate::rpc::{errors::RpcConversionError, generated as proto};

// MERKLE PATH
// ================================================================================================

impl From<MerklePath> for proto::primitives::MerklePath {
    fn from(value: MerklePath) -> Self {
        (&value).into()
    }
}

impl From<&MerklePath> for proto::primitives::MerklePath {
    fn from(value: &MerklePath) -> Self {
        let siblings = value.nodes().iter().map(proto::primitives::Digest::from).collect();
        proto::primitives::MerklePath { siblings }
    }
}

impl TryFrom<&proto::primitives::MerklePath> for MerklePath {
    type Error = RpcConversionError;

    fn try_from(merkle_path: &proto::primitives::MerklePath) -> Result<Self, Self::Error> {
        merkle_path.siblings.iter().map(Word::try_from).collect()
    }
}

impl TryFrom<proto::primitives::MerklePath> for MerklePath {
    type Error = RpcConversionError;

    fn try_from(merkle_path: proto::primitives::MerklePath) -> Result<Self, Self::Error> {
        MerklePath::try_from(&merkle_path)
    }
}

// SPARSE MERKLE PATH

// ================================================================================================

impl From<SparseMerklePath> for proto::primitives::SparseMerklePath {
    fn from(value: SparseMerklePath) -> Self {
        let (empty_nodes_mask, siblings) = value.into_parts();

        proto::primitives::SparseMerklePath {
            empty_nodes_mask,

            siblings: siblings.into_iter().map(proto::primitives::Digest::from).collect(),
        }
    }
}

impl TryFrom<proto::primitives::SparseMerklePath> for SparseMerklePath {
    type Error = RpcConversionError;

    fn try_from(merkle_path: proto::primitives::SparseMerklePath) -> Result<Self, Self::Error> {
        Ok(SparseMerklePath::from_parts(
            merkle_path.empty_nodes_mask,
            merkle_path
                .siblings
                .into_iter()
                .map(Word::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        )?)
    }
}

// MMR DELTA
// ================================================================================================

impl TryFrom<MmrDelta> for proto::primitives::MmrDelta {
    type Error = RpcConversionError;

    fn try_from(value: MmrDelta) -> Result<Self, Self::Error> {
        let data = value.data.into_iter().map(proto::primitives::Digest::from).collect();
        Ok(proto::primitives::MmrDelta {
            forest: u64::try_from(value.forest.num_leaves())?,
            data,
        })
    }
}

impl TryFrom<proto::primitives::MmrDelta> for MmrDelta {
    type Error = RpcConversionError;

    fn try_from(value: proto::primitives::MmrDelta) -> Result<Self, Self::Error> {
        let data: Result<Vec<_>, RpcConversionError> =
            value.data.into_iter().map(Word::try_from).collect();

        Ok(MmrDelta {
            forest: Forest::new(usize::try_from(value.forest).expect("u64 should fit in usize")),
            data: data?,
        })
    }
}
