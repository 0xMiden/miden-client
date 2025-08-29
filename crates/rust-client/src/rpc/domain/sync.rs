use alloc::vec::Vec;

use miden_objects::Word;
use miden_objects::account::AccountId;
use miden_objects::block::{BlockHeader, BlockNumber};
use miden_objects::crypto::merkle::MmrDelta;
use miden_objects::note::NoteId;
use miden_objects::transaction::TransactionId;

use super::note::CommittedNote;
use super::transaction::TransactionInclusion;
use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::{RpcError, generated as proto};

// STATE SYNC INFO
// ================================================================================================

/// Represents a `proto::rpc_store::SyncStateResponse` with fields converted into domain types.
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

// STATE SYNC INFO CONVERSION
// ================================================================================================

impl TryFrom<proto::rpc_store::SyncStateResponse> for StateSyncInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::SyncStateResponse) -> Result<Self, Self::Error> {
        let chain_tip = value.chain_tip;

        // Validate and convert block header
        let block_header: BlockHeader = value
            .block_header
            .ok_or(proto::rpc_store::SyncStateResponse::missing_field(stringify!(block_header)))?
            .try_into()?;

        // Validate and convert MMR Delta
        let mmr_delta = value
            .mmr_delta
            .ok_or(proto::rpc_store::SyncStateResponse::missing_field(stringify!(mmr_delta)))?
            .try_into()?;

        // Validate and convert account commitment updates into an (AccountId, Word) tuple
        let mut account_commitment_updates = vec![];
        for update in value.accounts {
            let account_id = update
                .account_id
                .ok_or(proto::rpc_store::SyncStateResponse::missing_field(stringify!(
                    accounts.account_id
                )))?
                .try_into()?;
            let account_commitment = update
                .account_commitment
                .ok_or(proto::rpc_store::SyncStateResponse::missing_field(stringify!(
                    accounts.account_commitment
                )))?
                .try_into()?;
            account_commitment_updates.push((account_id, account_commitment));
        }

        // Validate and convert account note inclusions into an (AccountId, Word) tuple
        let mut note_inclusions = vec![];
        for note in value.notes {
            let note_id: NoteId = note
                .note_id
                .ok_or(proto::rpc_store::SyncStateResponse::missing_field(stringify!(
                    notes.note_id
                )))?
                .try_into()?;

            let inclusion_path = note
                .inclusion_path
                .ok_or(proto::rpc_store::SyncStateResponse::missing_field(stringify!(
                    notes.inclusion_path
                )))?
                .try_into()?;

            let metadata = note
                .metadata
                .ok_or(proto::rpc_store::SyncStateResponse::missing_field(stringify!(
                    notes.metadata
                )))?
                .try_into()?;

            let committed_note = super::note::CommittedNote::new(
                note_id,
                u16::try_from(note.note_index_in_block).expect("note index out of range"),
                inclusion_path,
                metadata,
            );

            note_inclusions.push(committed_note);
        }

        let transactions = value
            .transactions
            .iter()
            .map(|transaction_summary| {
                let transaction_id = transaction_summary.transaction_id.ok_or(
                    proto::rpc_store::SyncStateResponse::missing_field(stringify!(
                        transactions.transaction_id
                    )),
                )?;
                let transaction_id = TransactionId::try_from(transaction_id)?;

                let transaction_block_num = transaction_summary.block_num;

                let transaction_account_id = transaction_summary.account_id.clone().ok_or(
                    proto::rpc_store::SyncStateResponse::missing_field(stringify!(
                        transactions.account_id
                    )),
                )?;
                let transaction_account_id = AccountId::try_from(transaction_account_id)?;

                Ok(TransactionInclusion {
                    transaction_id,
                    block_num: transaction_block_num,
                    account_id: transaction_account_id,
                })
            })
            .collect::<Result<Vec<TransactionInclusion>, RpcError>>()?;

        Ok(Self {
            chain_tip: chain_tip.into(),
            block_header,
            mmr_delta,
            account_commitment_updates,
            note_inclusions,
            transactions,
        })
    }
}
