use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::{Account, AccountHeader, AccountId};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::MmrDelta;
use miden_protocol::note::{Note, NoteId, NoteTag, Nullifier};
use miden_protocol::transaction::InputNoteCommitment;
use tracing::info;

use super::StateSync;
use crate::ClientError;
use crate::rpc::domain::note::NoteSyncBlock;
use crate::rpc::domain::transaction::{
    TransactionInclusion,
    TransactionRecord as RpcTransactionRecord,
};

// SYNC DATA
// ================================================================================================

/// Data fetched and derived from the node needed to sync the client to the chain tip.
///
/// Aggregates RPC responses (`sync_chain_mmr`, `sync_notes`, `sync_transactions`) along with
/// derived data such as account commitment updates and execution-ordered nullifiers. This is an
/// intermediate representation that is filtered and transformed into a [`StateSyncUpdate`] before
/// being applied to the store.
pub(super) struct SyncData {
    /// MMR delta covering the full range from `current_block` to `chain_tip`.
    pub mmr_delta: MmrDelta,
    /// Chain tip block header.
    pub chain_tip_header: BlockHeader,
    /// Blocks with matching notes that the client is interested in.
    pub note_blocks: Vec<NoteSyncBlock>,
    /// Full note bodies for public notes, keyed by note ID.
    pub public_notes: BTreeMap<NoteId, Note>,
    /// Latest account state commitment per account, derived from transaction records.
    pub account_commitment_updates: Vec<(AccountId, Word)>,
    /// Transaction inclusions for tracked accounts, derived from transaction records.
    pub transactions: Vec<TransactionInclusion>,
    /// Execution-ordered nullifiers, derived from transaction records.
    pub nullifiers: Vec<Nullifier>,
}

// FETCH METHODS
// ================================================================================================

impl StateSync {
    /// Fetches and derives sync data from the node.
    ///
    /// Calls the following RPC endpoints:
    /// 1. `sync_chain_mmr` — discovers the chain tip, gets the MMR delta and chain tip header.
    /// 2. `sync_notes` — fetches note inclusions and public note bodies for the range.
    /// 3. `sync_transactions` — gets transaction records for tracked accounts, from which
    ///    account commitment updates, transaction inclusions, and execution-ordered nullifiers
    ///    are derived.
    ///
    /// Returns `None` when the client is already at the chain tip (no progress).
    pub(super) async fn fetch_sync_data(
        &self,
        current_block_num: BlockNumber,
        account_ids: &[AccountId],
        note_tags: &Arc<BTreeSet<NoteTag>>,
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
            .sync_notes_with_details(current_block_num, Some(chain_tip), note_tags.as_ref())
            .await?;

        let note_count: usize = sync_notes_result.blocks.iter().map(|b| b.notes.len()).sum();
        info!(
            blocks_with_notes = sync_notes_result.blocks.len(),
            notes = note_count,
            public_notes = sync_notes_result.public_notes.len(),
            "Fetched note sync data.",
        );

        // Step 3: Gather transactions for tracked accounts over the full range.
        let (account_commitment_updates, transactions, nullifiers) =
            self.fetch_transaction_data(current_block_num, chain_tip, account_ids).await?;

        Ok(Some(SyncData {
            mmr_delta: chain_mmr_info.mmr_delta,
            chain_tip_header: chain_mmr_info.block_header,
            note_blocks: sync_notes_result.blocks,
            public_notes: sync_notes_result.public_notes,
            account_commitment_updates,
            transactions,
            nullifiers,
        }))
    }

    /// Fetches transaction data for the given range and account IDs.
    async fn fetch_transaction_data(
        &self,
        block_from: BlockNumber,
        block_to: BlockNumber,
        account_ids: &[AccountId],
    ) -> Result<(Vec<(AccountId, Word)>, Vec<TransactionInclusion>, Vec<Nullifier>), ClientError>
    {
        if account_ids.is_empty() {
            return Ok((vec![], vec![], vec![]));
        }

        let tx_info = self
            .rpc_api
            .sync_transactions(block_from, Some(block_to), account_ids.to_vec())
            .await?;

        let transaction_records = tx_info.transaction_records;

        let account_updates = derive_account_commitment_updates(&transaction_records);
        let nullifiers = compute_ordered_nullifiers(&transaction_records);

        let tx_inclusions = transaction_records
            .into_iter()
            .map(|r| {
                let nullifiers = r
                    .transaction_header
                    .input_notes()
                    .iter()
                    .map(InputNoteCommitment::nullifier)
                    .collect();
                TransactionInclusion {
                    transaction_id: r.transaction_header.id(),
                    block_num: r.block_num,
                    account_id: r.transaction_header.account_id(),
                    initial_state_commitment: r.transaction_header.initial_state_commitment(),
                    nullifiers,
                    output_notes: r.output_notes,
                }
            })
            .collect();

        Ok((account_updates, tx_inclusions, nullifiers))
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
            // check if this updated account state is tracked by the client
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

// HELPERS
// ================================================================================================

/// Derives account commitment updates from transaction records.
///
/// For each unique account, takes the `final_state_commitment` from the transaction with the
/// highest `block_num`. This replicates the old `SyncState` behavior where the node returned
/// the latest account commitment per account in the synced range.
fn derive_account_commitment_updates(
    transaction_records: &[RpcTransactionRecord],
) -> Vec<(AccountId, Word)> {
    let mut latest_by_account: BTreeMap<AccountId, &RpcTransactionRecord> = BTreeMap::new();

    for record in transaction_records {
        let account_id = record.transaction_header.account_id();
        latest_by_account
            .entry(account_id)
            .and_modify(|existing| {
                if record.block_num > existing.block_num {
                    *existing = record;
                }
            })
            .or_insert(record);
    }

    latest_by_account
        .into_iter()
        .map(|(account_id, record)| {
            (account_id, record.transaction_header.final_state_commitment())
        })
        .collect()
}

/// Returns nullifiers ordered by consuming transaction position, per account.
///
/// Groups RPC transaction records by (`account_id`, `block_num`), chains them using
/// `initial_state_commitment` / `final_state_commitment`, and collects each transaction's
/// input note nullifiers in execution order. Nullifiers from the same account are in execution
/// order; ordering across different accounts is arbitrary.
pub(super) fn compute_ordered_nullifiers(
    transaction_records: &[RpcTransactionRecord],
) -> Vec<Nullifier> {
    // Group transactions by (account_id, block_num).
    let mut groups: BTreeMap<(AccountId, BlockNumber), Vec<&RpcTransactionRecord>> =
        BTreeMap::new();

    for record in transaction_records {
        let account_id = record.transaction_header.account_id();
        groups.entry((account_id, record.block_num)).or_default().push(record);
    }

    let mut result = Vec::new();

    for txs in groups.values() {
        // Build a lookup from initial_state_commitment -> transaction record.
        let mut init_to_tx: BTreeMap<Word, &RpcTransactionRecord> = txs
            .iter()
            .map(|tx| (tx.transaction_header.initial_state_commitment(), *tx))
            .collect();

        // Build a set of all final states to find the chain start.
        let final_states: BTreeSet<Word> =
            txs.iter().map(|tx| tx.transaction_header.final_state_commitment()).collect();

        // Find the chain start: the tx whose initial_state_commitment is not any other tx's
        // final_state_commitment.
        let chain_start = txs
            .iter()
            .find(|tx| !final_states.contains(&tx.transaction_header.initial_state_commitment()));

        let Some(start_tx) = chain_start else {
            continue;
        };

        // Walk the chain from start, removing each step from the map.
        let mut current =
            init_to_tx.remove(&start_tx.transaction_header.initial_state_commitment());

        while let Some(tx) = current {
            for commitment in tx.transaction_header.input_notes().iter() {
                result.push(commitment.nullifier());
            }
            current = init_to_tx.remove(&tx.transaction_header.final_state_commitment());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use miden_protocol::asset::FungibleAsset;
    use miden_protocol::block::BlockNumber;
    use miden_protocol::note::Nullifier;
    use miden_protocol::transaction::{InputNoteCommitment, InputNotes, TransactionHeader};
    use miden_protocol::{Felt, ZERO};

    use crate::rpc::domain::transaction::{
        ACCOUNT_ID_NATIVE_ASSET_FAUCET,
        TransactionRecord as RpcTransactionRecord,
    };

    fn word(n: u64) -> miden_protocol::Word {
        [Felt::new(n), ZERO, ZERO, ZERO].into()
    }

    fn make_rpc_tx(
        init_state: u64,
        final_state: u64,
        nullifier_vals: &[u64],
        block_number: u32,
    ) -> RpcTransactionRecord {
        let account_id = miden_protocol::account::AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE,
        )
        .unwrap();

        let input_notes = InputNotes::new_unchecked(
            nullifier_vals
                .iter()
                .map(|v| InputNoteCommitment::from(Nullifier::from_raw(word(*v))))
                .collect(),
        );

        let fee =
            FungibleAsset::new(ACCOUNT_ID_NATIVE_ASSET_FAUCET.try_into().expect("valid"), 0u64)
                .unwrap();

        RpcTransactionRecord {
            block_num: BlockNumber::from(block_number),
            transaction_header: TransactionHeader::new(
                account_id,
                word(init_state),
                word(final_state),
                input_notes,
                vec![],
                fee,
            ),
            output_notes: vec![],
        }
    }

    #[test]
    fn chains_rpc_transactions_by_state_commitment() {
        let tx_a = make_rpc_tx(1, 2, &[10], 5);
        let tx_b = make_rpc_tx(2, 3, &[20], 5);
        let tx_c = make_rpc_tx(3, 4, &[30], 5);

        let result = super::compute_ordered_nullifiers(&[tx_c, tx_a, tx_b]);

        assert_eq!(result[0], Nullifier::from_raw(word(10)));
        assert_eq!(result[1], Nullifier::from_raw(word(20)));
        assert_eq!(result[2], Nullifier::from_raw(word(30)));
    }

    #[test]
    fn groups_independently_by_account_and_block() {
        let tx_a1 = make_rpc_tx(1, 2, &[10], 5);
        let tx_a2 = make_rpc_tx(2, 3, &[20], 5);
        let tx_a3 = make_rpc_tx(3, 4, &[30], 6);

        let account_b = miden_protocol::account::AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
        )
        .unwrap();

        let fee =
            FungibleAsset::new(ACCOUNT_ID_NATIVE_ASSET_FAUCET.try_into().expect("valid"), 0u64)
                .unwrap();

        let tx_b1 = RpcTransactionRecord {
            block_num: BlockNumber::from(5u32),
            transaction_header: TransactionHeader::new(
                account_b,
                word(100),
                word(200),
                InputNotes::new_unchecked(vec![InputNoteCommitment::from(
                    Nullifier::from_raw(word(40)),
                )]),
                vec![],
                fee,
            ),
            output_notes: vec![],
        };

        let result = super::compute_ordered_nullifiers(&[tx_a2, tx_b1, tx_a3, tx_a1]);

        let pos = |val: u64| -> usize {
            result.iter().position(|n| *n == Nullifier::from_raw(word(val))).unwrap()
        };

        assert!(pos(10) < pos(20));
        assert!(result.contains(&Nullifier::from_raw(word(30))));
        assert!(result.contains(&Nullifier::from_raw(word(40))));
    }

    #[test]
    fn multiple_nullifiers_per_transaction_are_consecutive() {
        let tx = make_rpc_tx(1, 2, &[10, 20, 30], 5);

        let result = super::compute_ordered_nullifiers(&[tx]);

        assert_eq!(result.len(), 3);
        assert!(result.contains(&Nullifier::from_raw(word(10))));
        assert!(result.contains(&Nullifier::from_raw(word(20))));
        assert!(result.contains(&Nullifier::from_raw(word(30))));
    }

    #[test]
    fn empty_input_returns_empty_vec() {
        let result = super::compute_ordered_nullifiers(&[]);
        assert!(result.is_empty());
    }
}
