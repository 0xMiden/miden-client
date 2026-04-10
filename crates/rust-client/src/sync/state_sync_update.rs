use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::{InOrderIndex, MmrPeaks};
use miden_protocol::note::{NoteId, Nullifier};
use miden_protocol::transaction::{InputNoteCommitment, TransactionId};

use super::SyncSummary;
use crate::ClientError;
use crate::account::Account;
use crate::note::{NoteUpdateTracker, NoteUpdateType};
use crate::rpc::domain::nullifier::NullifierUpdate;
use crate::rpc::domain::transaction::{
    TransactionInclusion,
    TransactionRecord as RpcTransactionRecord,
};
use crate::transaction::{DiscardCause, TransactionRecord, TransactionStatus};

// STATE SYNC UPDATE
// ================================================================================================

/// Contains all information needed to apply the update in the store after syncing with the node.
#[derive(Default)]
pub struct StateSyncUpdate {
    /// The block number of the last block that was synced.
    pub block_num: BlockNumber,
    /// New blocks and authentication nodes.
    pub block_updates: BlockUpdates,
    /// New and updated notes to be upserted in the store.
    pub note_updates: NoteUpdateTracker,
    /// Committed and discarded transactions after the sync.
    pub transaction_updates: TransactionUpdateTracker,
    /// Public account updates and mismatched private accounts after the sync.
    pub account_updates: AccountUpdates,
}

impl From<&StateSyncUpdate> for SyncSummary {
    fn from(value: &StateSyncUpdate) -> Self {
        let new_public_note_ids = value
            .note_updates
            .updated_input_notes()
            .filter_map(|note_update| {
                let note = note_update.inner();
                if let NoteUpdateType::Insert = note_update.update_type() {
                    Some(note.id())
                } else {
                    None
                }
            })
            .collect();

        let committed_note_ids: BTreeSet<NoteId> = value
            .note_updates
            .updated_input_notes()
            .filter_map(|note_update| {
                let note = note_update.inner();
                if let NoteUpdateType::Update = note_update.update_type() {
                    note.is_committed().then_some(note.id())
                } else {
                    None
                }
            })
            .chain(value.note_updates.updated_output_notes().filter_map(|note_update| {
                let note = note_update.inner();
                if let NoteUpdateType::Update = note_update.update_type() {
                    note.is_committed().then_some(note.id())
                } else {
                    None
                }
            }))
            .collect();

        let consumed_note_ids: BTreeSet<NoteId> = value
            .note_updates
            .updated_input_notes()
            .filter_map(|note| note.inner().is_consumed().then_some(note.inner().id()))
            .collect();

        SyncSummary::new(
            value.block_num,
            new_public_note_ids,
            committed_note_ids.into_iter().collect(),
            consumed_note_ids.into_iter().collect(),
            value
                .account_updates
                .updated_public_accounts()
                .iter()
                .map(Account::id)
                .collect(),
            value
                .account_updates
                .mismatched_private_accounts()
                .iter()
                .map(|(id, _)| *id)
                .collect(),
            value.transaction_updates.committed_transactions().map(|t| t.id).collect(),
        )
    }
}

impl StateSyncUpdate {
    /// Applies pre-derived transaction data to the update.
    ///
    /// 1. Stores execution-ordered nullifiers for later consumption ordering.
    /// 2. Processes each transaction inclusion (committing local transactions or recording external
    ///    consumers for tracked accounts).
    /// 3. Discards stale/expired pending transactions.
    /// 4. Transitions tracked output notes to committed using inclusion proofs.
    pub fn apply_transaction_data(
        &mut self,
        chain_tip_header: &BlockHeader,
        tx_inclusions: &[TransactionInclusion],
        ordered_nullifiers: Vec<Nullifier>,
        tx_discard_delta: Option<u32>,
    ) -> Result<(), ClientError> {
        self.transaction_updates.extend_nullifiers(ordered_nullifiers);

        for transaction_inclusion in tx_inclusions {
            self.transaction_updates.apply_transaction_inclusion(
                transaction_inclusion,
                u64::from(chain_tip_header.timestamp()),
            );
        }

        self.transaction_updates
            .apply_sync_height_update(chain_tip_header.block_num(), tx_discard_delta);

        for transaction in tx_inclusions {
            self.note_updates
                .apply_output_note_inclusion_proofs(&transaction.output_notes)?;
        }

        Ok(())
    }

    /// Applies nullifier updates to both note and transaction trackers.
    ///
    /// For each nullifier:
    /// 1. Resolves whether a committed local transaction consumed the note.
    /// 2. Resolves the external consumer account (if any) from the transaction tracker.
    /// 3. Applies the state transition to the note tracker.
    /// 4. Discards pending transactions whose input note was nullified.
    pub fn apply_nullifier_updates(
        &mut self,
        nullifier_updates: Vec<NullifierUpdate>,
    ) -> Result<(), ClientError> {
        for update in nullifier_updates {
            let external_consumer =
                self.transaction_updates.external_nullifier_account(&update.nullifier);
            let consumed_tx_order = self.transaction_updates.nullifier_order(&update.nullifier);

            let transaction_updates = &self.transaction_updates;
            self.note_updates.apply_nullifiers_state_transitions(
                &update,
                |tx_id| transaction_updates.committed_transaction_block(tx_id),
                external_consumer,
                consumed_tx_order,
            )?;

            self.transaction_updates.apply_input_note_nullified(update.nullifier);
        }
        Ok(())
    }
}

/// Contains all the block information that needs to be added in the client's store after a sync.
#[derive(Debug, Clone, Default)]
pub struct BlockUpdates {
    /// New block headers to be stored, keyed by block number. The value contains the block
    /// header, a flag indicating whether the block contains notes relevant to the client, and
    /// the MMR peaks for the block.
    block_headers: BTreeMap<BlockNumber, (BlockHeader, bool, MmrPeaks)>,
    /// New authentication nodes that are meant to be stored in order to authenticate block
    /// headers.
    new_authentication_nodes: Vec<(InOrderIndex, Word)>,
}

impl BlockUpdates {
    /// Adds or updates a block header and its corresponding data in this [`BlockUpdates`].
    ///
    /// If the block header already exists (same block number), the `has_client_notes` flag is
    /// OR-ed and the peaks are kept from the first insertion. Otherwise a new entry is added.
    pub fn insert(
        &mut self,
        block_header: BlockHeader,
        has_client_notes: bool,
        peaks: MmrPeaks,
        new_authentication_nodes: Vec<(InOrderIndex, Word)>,
    ) {
        self.block_headers
            .entry(block_header.block_num())
            .and_modify(|(_, existing_has_notes, _)| {
                *existing_has_notes |= has_client_notes;
            })
            .or_insert((block_header, has_client_notes, peaks));

        self.new_authentication_nodes.extend(new_authentication_nodes);
    }

    /// Returns the new block headers to be stored, along with a flag indicating whether the block
    /// contains notes that are relevant to the client and the MMR peaks for the block.
    pub fn block_headers(&self) -> impl Iterator<Item = &(BlockHeader, bool, MmrPeaks)> {
        self.block_headers.values()
    }

    /// Adds authentication nodes without an associated block header.
    ///
    /// This is used when a synced block is not stored (no relevant notes and not the chain tip)
    /// but the MMR authentication nodes it produced must still be persisted so that the on-disk
    /// state stays consistent with the in-memory `PartialMmr`.
    pub fn extend_authentication_nodes(&mut self, nodes: Vec<(InOrderIndex, Word)>) {
        self.new_authentication_nodes.extend(nodes);
    }

    /// Returns the new authentication nodes that are meant to be stored in order to authenticate
    /// block headers.
    pub fn new_authentication_nodes(&self) -> &[(InOrderIndex, Word)] {
        &self.new_authentication_nodes
    }
}

/// Contains transaction changes to apply to the store.
#[derive(Default)]
pub struct TransactionUpdateTracker {
    /// Transactions that were committed in the block.
    transactions: BTreeMap<TransactionId, TransactionRecord>,
    /// Nullifier-to-account mappings from external transactions by tracked accounts.
    external_nullifier_accounts: BTreeMap<Nullifier, AccountId>,
    /// Map from nullifier to its per-account position in the consuming transaction order.
    /// Populated from execution-ordered nullifiers derived from `sync_transactions`.
    nullifier_order: BTreeMap<Nullifier, u32>,
}

impl TransactionUpdateTracker {
    /// Creates a new [`TransactionUpdateTracker`]
    pub fn new(transactions: Vec<TransactionRecord>) -> Self {
        let transactions =
            transactions.into_iter().map(|tx| (tx.id, tx)).collect::<BTreeMap<_, _>>();

        Self {
            transactions,
            external_nullifier_accounts: BTreeMap::new(),
            nullifier_order: BTreeMap::new(),
        }
    }

    /// Returns a reference to committed transactions.
    pub fn committed_transactions(&self) -> impl Iterator<Item = &TransactionRecord> {
        self.transactions
            .values()
            .filter(|tx| matches!(tx.status, TransactionStatus::Committed { .. }))
    }

    /// Returns a reference to discarded transactions.
    pub fn discarded_transactions(&self) -> impl Iterator<Item = &TransactionRecord> {
        self.transactions
            .values()
            .filter(|tx| matches!(tx.status, TransactionStatus::Discarded(_)))
    }

    /// Returns a mutable reference to pending transactions in the tracker.
    fn mutable_pending_transactions(&mut self) -> impl Iterator<Item = &mut TransactionRecord> {
        self.transactions
            .values_mut()
            .filter(|tx| matches!(tx.status, TransactionStatus::Pending))
    }

    /// Returns transaction IDs of all transactions that have been updated.
    pub fn updated_transaction_ids(&self) -> impl Iterator<Item = TransactionId> {
        self.committed_transactions()
            .chain(self.discarded_transactions())
            .map(|tx| tx.id)
    }

    /// Returns the account ID that consumed the given nullifier in an external transaction, if
    /// available.
    pub fn external_nullifier_account(&self, nullifier: &Nullifier) -> Option<AccountId> {
        self.external_nullifier_accounts.get(nullifier).copied()
    }

    /// Appends execution-ordered nullifiers derived from transaction records.
    ///
    /// Nullifiers from the same account are in execution order; ordering across different
    /// accounts is not guaranteed.
    pub fn extend_nullifiers(&mut self, nullifiers: impl IntoIterator<Item = Nullifier>) {
        for nullifier in nullifiers {
            let next_pos =
                u32::try_from(self.nullifier_order.len()).expect("nullifier count exceeds u32");
            self.nullifier_order.entry(nullifier).or_insert(next_pos);
        }
    }

    /// Returns the per-account execution position of the given nullifier, or `None` if it is
    /// not present.
    pub fn nullifier_order(&self, nullifier: &Nullifier) -> Option<u32> {
        self.nullifier_order.get(nullifier).copied()
    }

    /// Returns the block number at which the given transaction was committed, if it exists and
    /// is in the committed state.
    pub fn committed_transaction_block(&self, id: &TransactionId) -> Option<BlockNumber> {
        self.transactions.get(id).and_then(|tx| match tx.status {
            TransactionStatus::Committed { block_number, .. } => Some(block_number),
            _ => None,
        })
    }

    /// Applies the necessary state transitions to the [`TransactionUpdateTracker`] when a
    /// transaction is included in a block.
    pub fn apply_transaction_inclusion(
        &mut self,
        transaction_inclusion: &TransactionInclusion,
        timestamp: u64,
    ) {
        if let Some(transaction) = self.transactions.get_mut(&transaction_inclusion.transaction_id)
        {
            transaction.commit_transaction(transaction_inclusion.block_num, timestamp);
            return;
        }

        // Fallback for transactions with unauthenticated input notes: the node
        // authenticates these notes during processing, which changes the transaction
        // ID. Match by account ID and pre-transaction state instead.
        if let Some(transaction) = self.transactions.values_mut().find(|tx| {
            tx.details.account_id == transaction_inclusion.account_id
                && tx.details.init_account_state == transaction_inclusion.initial_state_commitment
        }) {
            transaction.commit_transaction(transaction_inclusion.block_num, timestamp);
            return;
        }

        // No local transaction matched. This is an external transaction by a tracked account.
        // Record the nullifier→account mappings so we can attribute note consumption to tracked
        // accounts during nullifier processing.
        for nullifier in &transaction_inclusion.nullifiers {
            self.external_nullifier_accounts
                .insert(*nullifier, transaction_inclusion.account_id);
        }
    }

    /// Applies the necessary state transitions to the [`TransactionUpdateTracker`] when a the sync
    /// height of the client is updated. This may result in stale or expired transactions.
    pub fn apply_sync_height_update(
        &mut self,
        new_sync_height: BlockNumber,
        tx_discard_delta: Option<u32>,
    ) {
        if let Some(tx_discard_delta) = tx_discard_delta {
            self.discard_transaction_with_predicate(
                |transaction| {
                    transaction.details.submission_height
                        < new_sync_height.checked_sub(tx_discard_delta).unwrap_or_default()
                },
                DiscardCause::Stale,
            );
        }

        // NOTE: we check for <= new_sync height because at this point we would have committed the
        // transaction otherwise
        self.discard_transaction_with_predicate(
            |transaction| transaction.details.expiration_block_num <= new_sync_height,
            DiscardCause::Expired,
        );
    }

    /// Applies the necessary state transitions to the [`TransactionUpdateTracker`] when a note is
    /// nullified. this may result in transactions being discarded because they were processing the
    /// nullified note.
    pub fn apply_input_note_nullified(&mut self, input_note_nullifier: Nullifier) {
        self.discard_transaction_with_predicate(
            |transaction| {
                // Check if the note was being processed by a local transaction that didn't end up
                // being committed so it should be discarded
                transaction
                    .details
                    .input_note_nullifiers
                    .contains(&input_note_nullifier.as_word())
            },
            DiscardCause::InputConsumed,
        );
    }

    /// Discards transactions that have the same initial account state as the provided one.
    pub fn apply_invalid_initial_account_state(&mut self, invalid_account_state: Word) {
        self.discard_transaction_with_predicate(
            |transaction| transaction.details.init_account_state == invalid_account_state,
            DiscardCause::DiscardedInitialState,
        );
    }

    /// Discards transactions that match the predicate and also applies the new invalid account
    /// states
    fn discard_transaction_with_predicate<F>(&mut self, predicate: F, discard_cause: DiscardCause)
    where
        F: Fn(&TransactionRecord) -> bool,
    {
        let mut new_invalid_account_states = vec![];

        for transaction in self.mutable_pending_transactions() {
            // Discard transactions, and also push the invalid account state if the transaction
            // got correctly discarded
            // NOTE: previous updates in a chain of state syncs could have committed a transaction,
            // so we need to check that `discard_transaction` returns `true` here (aka, it got
            // discarded from a valid state)
            if predicate(transaction) && transaction.discard_transaction(discard_cause) {
                new_invalid_account_states.push(transaction.details.final_account_state);
            }
        }

        for state in new_invalid_account_states {
            self.apply_invalid_initial_account_state(state);
        }
    }
}

// ACCOUNT UPDATES
// ================================================================================================

/// Contains account changes to apply to the store after a sync request.
#[derive(Debug, Clone, Default)]
pub struct AccountUpdates {
    /// Updated public accounts.
    updated_public_accounts: Vec<Account>,
    /// Account commitments received from the network that don't match the currently
    /// locally-tracked state of the private accounts.
    ///
    /// These updates may represent a stale account commitment (meaning that the latest local state
    /// hasn't been committed). If this is not the case, the account may be locked until the state
    /// is restored manually.
    mismatched_private_accounts: Vec<(AccountId, Word)>,
}

impl AccountUpdates {
    /// Creates a new instance of `AccountUpdates`.
    pub fn new(
        updated_public_accounts: Vec<Account>,
        mismatched_private_accounts: Vec<(AccountId, Word)>,
    ) -> Self {
        Self {
            updated_public_accounts,
            mismatched_private_accounts,
        }
    }

    /// Returns the updated public accounts.
    pub fn updated_public_accounts(&self) -> &[Account] {
        &self.updated_public_accounts
    }

    /// Returns the mismatched private accounts.
    pub fn mismatched_private_accounts(&self) -> &[(AccountId, Word)] {
        &self.mismatched_private_accounts
    }

    pub fn extend(&mut self, other: AccountUpdates) {
        self.updated_public_accounts.extend(other.updated_public_accounts);
        self.mismatched_private_accounts.extend(other.mismatched_private_accounts);
    }
}

// HELPERS
// ================================================================================================

/// Returns nullifiers ordered by consuming transaction position, per account.
///
/// Groups RPC transaction records by (`account_id`, `block_num`), chains them using
/// `initial_state_commitment` / `final_state_commitment`, and collects each transaction's
/// input note nullifiers in execution order. Nullifiers from the same account are in execution
/// order; ordering across different accounts is arbitrary.
fn compute_ordered_nullifiers(transaction_records: &[RpcTransactionRecord]) -> Vec<Nullifier> {
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

/// Derives account commitment updates from transaction records.
///
/// For each unique account, takes the `final_state_commitment` from the transaction with the
/// highest `block_num`.
pub(crate) fn derive_account_commitment_updates(
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

/// Derives transaction inclusions and execution-ordered nullifiers from raw transaction records.
///
/// Returns the transaction inclusions (with nullifiers and output notes extracted from each
/// record's header) and nullifiers ordered by consuming transaction position per account.
pub(crate) fn derive_transaction_inclusions(
    transaction_records: Vec<RpcTransactionRecord>,
) -> (Vec<TransactionInclusion>, Vec<Nullifier>) {
    let ordered_nullifiers = compute_ordered_nullifiers(&transaction_records);

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

    (tx_inclusions, ordered_nullifiers)
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
                InputNotes::new_unchecked(vec![InputNoteCommitment::from(Nullifier::from_raw(
                    word(40),
                ))]),
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
