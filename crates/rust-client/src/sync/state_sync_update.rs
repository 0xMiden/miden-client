use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::{InOrderIndex, MmrPeaks};
use miden_protocol::note::{NoteId, Nullifier};
use miden_protocol::transaction::TransactionId;

use super::SyncSummary;
use crate::ClientError;
use crate::account::Account;
use crate::note::{NoteUpdateTracker, NoteUpdateType};
use crate::rpc::domain::nullifier::NullifierUpdate;
use crate::rpc::domain::transaction::TransactionInclusion;
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
    /// Applies transaction data from the sync response.
    ///
    /// This processes transaction inclusions (committing or recording external transactions),
    /// discards stale/expired transactions, stores execution-ordered nullifiers, and transitions
    /// tracked output notes to committed using inclusion proofs.
    pub fn apply_transaction_data(
        &mut self,
        chain_tip_header: &BlockHeader,
        transactions: &[TransactionInclusion],
        nullifiers: Vec<Nullifier>,
        tx_discard_delta: Option<u32>,
    ) -> Result<(), ClientError> {
        self.transaction_updates.extend_nullifiers(nullifiers);

        for transaction_inclusion in transactions {
            self.transaction_updates.apply_transaction_inclusion(
                transaction_inclusion,
                u64::from(chain_tip_header.timestamp()),
            );
        }

        self.transaction_updates
            .apply_sync_height_update(chain_tip_header.block_num(), tx_discard_delta);

        for transaction in transactions {
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
            let consumed_tx_order =
                self.transaction_updates.nullifier_order(&update.nullifier);

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
