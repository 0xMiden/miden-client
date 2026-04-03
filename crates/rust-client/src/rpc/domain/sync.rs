use miden_protocol::block::BlockNumber;
use miden_protocol::crypto::merkle::mmr::MmrDelta;

use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::{RpcError, generated as proto};

// CHAIN MMR INFO
// ================================================================================================

/// Represents the result of a `SyncChainMmr` RPC call, with fields converted into domain types.
pub struct ChainMmrInfo {
    /// The block number from which the delta starts (inclusive).
    pub block_from: BlockNumber,
    /// The block number up to which the delta covers (inclusive).
    pub block_to: BlockNumber,
    /// The MMR delta for the requested block range.
    pub mmr_delta: MmrDelta,
}

impl TryFrom<proto::rpc::SyncChainMmrResponse> for ChainMmrInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc::SyncChainMmrResponse) -> Result<Self, Self::Error> {
        let block_range = value
            .block_range
            .ok_or(proto::rpc::SyncChainMmrResponse::missing_field(stringify!(block_range)))?;

        let mmr_delta = value
            .mmr_delta
            .ok_or(proto::rpc::SyncChainMmrResponse::missing_field(stringify!(mmr_delta)))?
            .try_into()?;

        Ok(Self {
            block_from: block_range.block_from.into(),
            block_to: block_range
                .block_to
                .ok_or(proto::rpc::SyncChainMmrResponse::missing_field(stringify!(
                    block_range.block_to
                )))?
                .into(),
            mmr_delta,
        })
    }
}
