use alloc::vec::Vec;

use miden_objects::{
    Word,
    crypto::merkle::{Forest, MerklePath, MmrDelta},
};

use crate::rpc::{errors::RpcConversionError, generated};

// MERKLE PATH
// ================================================================================================

impl From<MerklePath> for generated::merkle::MerklePath {
    fn from(value: MerklePath) -> Self {
        (&value).into()
    }
}

impl From<&MerklePath> for generated::merkle::MerklePath {
    fn from(value: &MerklePath) -> Self {
        let siblings = value.nodes().iter().map(generated::digest::Digest::from).collect();
        generated::merkle::MerklePath { siblings }
    }
}

impl TryFrom<&generated::merkle::MerklePath> for MerklePath {
    type Error = RpcConversionError;

    fn try_from(merkle_path: &generated::merkle::MerklePath) -> Result<Self, Self::Error> {
        merkle_path.siblings.iter().map(Word::try_from).collect()
    }
}

impl TryFrom<generated::merkle::MerklePath> for MerklePath {
    type Error = RpcConversionError;

    fn try_from(merkle_path: generated::merkle::MerklePath) -> Result<Self, Self::Error> {
        MerklePath::try_from(&merkle_path)
    }
}

// MMR DELTA
// ================================================================================================

impl TryFrom<MmrDelta> for generated::mmr::MmrDelta {
    type Error = RpcConversionError;

    fn try_from(value: MmrDelta) -> Result<Self, Self::Error> {
        let data = value.data.into_iter().map(generated::digest::Digest::from).collect();
        Ok(generated::mmr::MmrDelta {
            forest: u64::try_from(value.forest.num_leaves())?,
            data,
        })
    }
}

impl TryFrom<generated::mmr::MmrDelta> for MmrDelta {
    type Error = RpcConversionError;

    fn try_from(value: generated::mmr::MmrDelta) -> Result<Self, Self::Error> {
        let data: Result<Vec<_>, RpcConversionError> =
            value.data.into_iter().map(Word::try_from).collect();

        Ok(MmrDelta {
            forest: Forest::new(usize::try_from(value.forest).expect("u64 should fit in usize")),
            data: data?,
        })
    }
}
