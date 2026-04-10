use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::MmrDelta;

use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::{RpcError, generated as proto};

// SYNC UPPER BOUND
// ================================================================================================

/// Upper bound for chain MMR synchronization.
///
/// Determines how far ahead to sync: either to a specific block number or to a chain tip
/// finality level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncTarget {
    /// Sync up to a specific block number (inclusive).
    BlockNumber(BlockNumber),
    /// Sync up to the latest committed block (the chain tip).
    CommittedChainTip,
    /// Sync up to the latest proven block, which may be behind the committed tip.
    ProvenChainTip,
}

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
    /// The block header at `block_to`.
    pub block_header: BlockHeader,
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

        let block_header = value
            .block_header
            .ok_or(proto::rpc::SyncChainMmrResponse::missing_field(stringify!(block_header)))?
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
            block_header,
        })
    }
}
