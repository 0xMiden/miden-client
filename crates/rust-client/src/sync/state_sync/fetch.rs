use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::{Account, AccountHeader, AccountId};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::MmrDelta;
use miden_protocol::note::{Note, NoteId, NoteTag};
use tracing::info;

use super::StateSync;
use crate::ClientError;
use crate::rpc::domain::note::NoteSyncBlock;
use crate::rpc::domain::transaction::TransactionRecord as RpcTransactionRecord;

// SYNC DATA
// ================================================================================================

/// Data fetched from the node needed to sync the client to the chain tip.
///
/// Aggregates the RPC responses from `sync_chain_mmr`, `sync_notes`, and `sync_transactions`.
/// This raw data is then processed into a [`StateSyncUpdate`](super::super::StateSyncUpdate)
/// before being applied to the store.
pub(super) struct SyncData {
    /// MMR delta covering the full range from `current_block` to `chain_tip`.
    pub mmr_delta: MmrDelta,
    /// Chain tip block header.
    pub chain_tip_header: BlockHeader,
    /// Blocks with matching notes that the client is interested in.
    pub note_blocks: Vec<NoteSyncBlock>,
    /// Full note bodies for public notes, keyed by note ID.
    pub public_notes: BTreeMap<NoteId, Note>,
    /// Transaction records for tracked accounts in the synced range.
    pub transaction_records: Vec<RpcTransactionRecord>,
}

// FETCH METHODS
// ================================================================================================

impl StateSync {
    /// Fetches sync data from the node.
    ///
    /// Calls the following RPC endpoints:
    /// 1. `sync_chain_mmr` — discovers the chain tip, gets the MMR delta and chain tip header.
    /// 2. `sync_notes` — fetches note inclusions and public note bodies for the range.
    /// 3. `sync_transactions` — gets transaction records for tracked accounts.
    ///
    /// Returns `None` when the client is already at the chain tip (no progress).
    pub(super) async fn fetch_sync_data(
        &self,
        current_block_num: BlockNumber,
        account_ids: &[AccountId],
        note_tags: &BTreeSet<NoteTag>,
    ) -> Result<Option<SyncData>, ClientError> {
        // Step 1: Fetch the MMR delta and chain tip header.
        let chain_mmr_info = self.rpc_api.sync_chain_mmr(current_block_num, None).await?;
        let chain_tip = chain_mmr_info.block_to;

        // No progress — already at the tip.
        if chain_tip == current_block_num {
            info!(block_num = %current_block_num, "Already at chain tip, nothing to sync.");
            return Ok(None);
        }

        info!(
            block_from = %current_block_num,
            block_to = %chain_tip,
            "Syncing state.",
        );

        // Step 2: Paginate sync_notes using the same chain tip so MMR paths are opened at
        // a consistent forest.
        let sync_notes_result = self
            .rpc_api
            .sync_notes_with_details(current_block_num, Some(chain_tip), note_tags)
            .await?;

        let note_count: usize = sync_notes_result.blocks.iter().map(|b| b.notes.len()).sum();
        info!(
            blocks_with_notes = sync_notes_result.blocks.len(),
            notes = note_count,
            public_notes = sync_notes_result.public_notes.len(),
            "Fetched note sync data.",
        );

        // Step 3: Gather transaction records for tracked accounts over the full range.
        let transaction_records = if account_ids.is_empty() {
            vec![]
        } else {
            self.rpc_api
                .sync_transactions(current_block_num, Some(chain_tip), account_ids.to_vec())
                .await?
                .transaction_records
        };

        Ok(Some(SyncData {
            mmr_delta: chain_mmr_info.mmr_delta,
            chain_tip_header: chain_mmr_info.block_header,
            note_blocks: sync_notes_result.blocks,
            public_notes: sync_notes_result.public_notes,
            transaction_records,
        }))
    }

    /// Queries the node for the latest state of the public accounts that don't match the current
    /// state of the client.
    pub(super) async fn get_updated_public_accounts(
        &self,
        account_updates: &[(AccountId, Word)],
        current_public_accounts: &[&AccountHeader],
    ) -> Result<Vec<Account>, ClientError> {
        let mut mismatched_public_accounts = vec![];

        for (id, commitment) in account_updates {
            if let Some(account) = current_public_accounts
                .iter()
                .find(|acc| *id == acc.id() && *commitment != acc.to_commitment())
            {
                mismatched_public_accounts.push(*account);
            }
        }

        self.rpc_api
            .get_updated_public_accounts(&mismatched_public_accounts)
            .await
            .map_err(ClientError::RpcError)
    }
}
