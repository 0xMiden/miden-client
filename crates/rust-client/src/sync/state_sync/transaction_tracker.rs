use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::note::Nullifier;
use miden_protocol::transaction::TransactionId;

use crate::rpc::domain::transaction::TransactionInclusion;
use crate::transaction::{DiscardCause, TransactionRecord, TransactionStatus};

// TRANSACTION UPDATE TRACKER
// ================================================================================================

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

    /// Processes transaction inclusions from the sync response.
    ///
    /// 1. Stores execution-ordered nullifiers for later consumption ordering.
    /// 2. Processes each transaction inclusion (committing local transactions or recording external
    ///    consumers for tracked accounts).
    /// 3. Discards stale/expired pending transactions.
    pub fn apply_transaction_inclusions(
        &mut self,
        chain_tip_header: &BlockHeader,
        tx_inclusions: &[TransactionInclusion],
        ordered_nullifiers: Vec<Nullifier>,
        tx_discard_delta: Option<u32>,
    ) {
        self.extend_nullifiers(ordered_nullifiers);

        for transaction_inclusion in tx_inclusions {
            self.apply_transaction_inclusion(
                transaction_inclusion,
                u64::from(chain_tip_header.timestamp()),
            );
        }

        self.apply_sync_height_update(chain_tip_header.block_num(), tx_discard_delta);
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
            if predicate(transaction) && transaction.discard_transaction(discard_cause) {
                new_invalid_account_states.push(transaction.details.final_account_state);
            }
        }

        for state in new_invalid_account_states {
            self.apply_invalid_initial_account_state(state);
        }
    }
}
