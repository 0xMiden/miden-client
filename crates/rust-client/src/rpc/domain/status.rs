use alloc::string::String;
use core::convert::TryInto;

use miden_protocol::Word;

use crate::rpc::{RpcError, generated as proto};

/// Represents node status info with fields converted into domain types.
pub struct RpcStatusInfo {
    pub version: String,
    pub genesis_commitment: Option<Word>,
    pub store: Option<StoreStatusInfo>,
    pub block_producer: Option<BlockProducerStatusInfo>,
}

/// Represents store status info with fields converted into domain types.
pub struct StoreStatusInfo {
    pub version: String,
    pub status: String,
    pub chain_tip: u32,
}

/// Represents block producer status info with fields converted into domain types.
pub struct BlockProducerStatusInfo {
    pub version: String,
    pub status: String,
    pub chain_tip: u32,
    pub mempool_stats: Option<MempoolStatsInfo>,
}

/// Represents mempool stats with fields converted into domain types.
pub struct MempoolStatsInfo {
    pub unbatched_transactions: u64,
    pub proposed_batches: u64,
    pub proven_batches: u64,
}

impl TryFrom<proto::rpc::RpcStatus> for RpcStatusInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc::RpcStatus) -> Result<Self, Self::Error> {
        let genesis_commitment = value.genesis_commitment.map(TryInto::try_into).transpose()?;
        Ok(Self {
            version: value.version,
            genesis_commitment,
            store: value.store.map(Into::into),
            block_producer: value.block_producer.map(Into::into),
        })
    }
}

impl From<proto::rpc::StoreStatus> for StoreStatusInfo {
    fn from(value: proto::rpc::StoreStatus) -> Self {
        Self {
            version: value.version,
            status: value.status,
            chain_tip: value.chain_tip,
        }
    }
}

impl From<proto::rpc::BlockProducerStatus> for BlockProducerStatusInfo {
    fn from(value: proto::rpc::BlockProducerStatus) -> Self {
        Self {
            version: value.version,
            status: value.status,
            chain_tip: value.chain_tip,
            mempool_stats: value.mempool_stats.map(Into::into),
        }
    }
}

impl From<proto::rpc::MempoolStats> for MempoolStatsInfo {
    fn from(value: proto::rpc::MempoolStats) -> Self {
        Self {
            unbatched_transactions: value.unbatched_transactions,
            proposed_batches: value.proposed_batches,
            proven_batches: value.proven_batches,
        }
    }
}
