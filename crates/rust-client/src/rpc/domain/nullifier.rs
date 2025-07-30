use miden_objects::{Word, note::Nullifier};

use crate::rpc::{domain::MissingFieldHelper, errors::RpcConversionError, generated as proto};

// NULLIFIER UPDATE
// ================================================================================================

/// Represents a note that was consumed in the node at a certain block.
#[derive(Debug, Clone)]
pub struct NullifierUpdate {
    /// The nullifier of the consumed note.
    pub nullifier: Nullifier,
    /// The number of the block in which the note consumption was registered.
    pub block_num: u32,
}

// CONVERSIONS
// ================================================================================================

impl TryFrom<proto::primitives::Digest> for Nullifier {
    type Error = RpcConversionError;

    fn try_from(value: proto::primitives::Digest) -> Result<Self, Self::Error> {
        let word: Word = value.try_into()?;
        Ok(word.into())
    }
}

impl TryFrom<&proto::rpc_store::check_nullifiers_by_prefix_response::NullifierUpdate>
    for NullifierUpdate
{
    type Error = RpcConversionError;

    fn try_from(
        value: &proto::rpc_store::check_nullifiers_by_prefix_response::NullifierUpdate,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            nullifier: value
                .nullifier
                .ok_or(proto::rpc_store::check_nullifiers_by_prefix_response::NullifierUpdate::missing_field(stringify!(nullifier)))?
                .try_into()?,
            block_num: value.block_num,
        })
    }
}
