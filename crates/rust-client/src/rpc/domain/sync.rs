use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::MmrDelta;

use super::note::CommittedNote;
use super::transaction::TransactionInclusion;
use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::{RpcError, generated as proto};

// STATE SYNC INFO
// ================================================================================================

/// Represents the composed result of `sync_notes`, `sync_chain_mmr`, and `sync_transactions`
/// with fields converted into domain types.
pub struct StateSyncInfo {
    /// The block number of the chain tip at the moment of the response.
    pub chain_tip: BlockNumber,
    /// The returned block header.
    pub block_header: BlockHeader,
    /// MMR delta that contains data for (`current_block.num`, `incoming_block_header.num-1`).
    pub mmr_delta: MmrDelta,
    /// Tuples of `AccountId` alongside their new account commitments.
    pub account_commitment_updates: Vec<(AccountId, Word)>,
    /// List of tuples of Note ID, Note Index and Merkle Path for all new notes.
    pub note_inclusions: Vec<CommittedNote>,
    /// List of transaction IDs of transaction that were included in (`request.block_num`,
    /// `response.block_num-1`) along with the account the tx was executed against and the block
    /// number the transaction was included in.
    pub transactions: Vec<TransactionInclusion>,
}

// CHAIN MMR INFO
// ================================================================================================

/// Represents the result of a `SyncChainMmr` RPC call, with fields converted into domain types.
pub struct ChainMmrInfo {
    /// The MMR delta for the requested block range.
    pub mmr_delta: MmrDelta,
}

impl TryFrom<proto::rpc::SyncChainMmrResponse> for ChainMmrInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc::SyncChainMmrResponse) -> Result<Self, Self::Error> {
        let mmr_delta = value
            .mmr_delta
            .ok_or(proto::rpc::SyncChainMmrResponse::missing_field(stringify!(mmr_delta)))?
            .try_into()?;

        Ok(Self { mmr_delta })
    }
}
