use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use miden_protocol::account::{
    Account,
    AccountDelta,
    AccountHeader,
    AccountId,
    AccountStorage,
    AccountStorageDelta,
    AccountVaultDelta,
    StorageMapKey,
    StorageSlotName,
};
use miden_protocol::asset::{Asset, AssetVault, AssetVaultKey};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::{InOrderIndex, MmrPeaks};
use miden_protocol::errors::AccountDeltaError;
use miden_protocol::note::{NoteId, Nullifier};
use miden_protocol::transaction::TransactionId;
use miden_protocol::{Felt, Word};

use super::SyncSummary;
use crate::note::{NoteUpdateTracker, NoteUpdateType};
use crate::rpc::domain::account_vault::AccountVaultUpdate;
use crate::rpc::domain::storage_map::StorageMapUpdate;
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
                .map(PublicAccountUpdate::id)
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
        debug_assert_eq!(
            peaks.forest().num_leaves(),
            block_header.block_num().as_usize(),
            "MMR peaks stored for a block header must use that block number as the forest",
        );

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
}

impl TransactionUpdateTracker {
    /// Creates a new [`TransactionUpdateTracker`]
    pub fn new(transactions: Vec<TransactionRecord>) -> Self {
        let transactions =
            transactions.into_iter().map(|tx| (tx.id, tx)).collect::<BTreeMap<_, _>>();

        Self {
            transactions,
            external_nullifier_accounts: BTreeMap::new(),
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

// PUBLIC ACCOUNT UPDATE
// ================================================================================================

/// Update to a single tracked public account.
///
/// `StateSync` emits one of two variants depending on whether the node could return the account's
/// full state in a single response:
///
/// - [`PublicAccountUpdate::Full`] carries the new [`Account`] state directly (used when no storage
///   map is oversized and the vault fits in the response). The store applies it by replacing the
///   local state — no delta computation needed.
/// - [`PublicAccountUpdate::Delta`] carries a [`PublicAccountDelta`] payload (new header plus
///   incremental updates from `sync_storage_maps` and `sync_account_vault`, used when any part of
///   the account is oversized). The store calls [`PublicAccountDelta::compute_account_delta`] to
///   derive the [`AccountDelta`] to apply.
pub enum PublicAccountUpdate {
    /// The account fits in a single proof response — the new full state is carried as-is.
    Full(Account),
    /// The account is oversized in some dimension. The new state must be reconstructed by
    /// replaying the carried incremental updates against the locally-stored state.
    Delta(PublicAccountDelta),
}

impl PublicAccountUpdate {
    /// Returns the account ID for this update.
    pub fn id(&self) -> AccountId {
        match self {
            Self::Full(account) => account.id(),
            Self::Delta(delta) => delta.id(),
        }
    }
}

/// Incremental delta payload for a public account update.
///
/// Carries the new account header plus the per-block updates fetched from the node's incremental
/// endpoints (`sync_storage_maps` and `sync_account_vault`). The store derives the
/// [`AccountDelta`] to apply by replaying these updates against its locally-stored account state
/// via [`Self::compute_account_delta`].
pub struct PublicAccountDelta {
    /// The new account header after applying these updates.
    new_header: AccountHeader,
    /// First block of the synced range (the client's previous sync height).
    block_from: BlockNumber,
    /// Last block of the synced range (the block at which `new_header` is observed).
    block_to: BlockNumber,
    /// New value-slot values from the `get_account_proof` storage header. Value slots are
    /// always small enough to fit in the response.
    value_slot_updates: Vec<(StorageSlotName, Word)>,
    /// Per-block storage map updates from `sync_storage_maps`.
    storage_map_updates: Vec<StorageMapUpdate>,
    /// Per-block vault updates from `sync_account_vault`.
    vault_updates: Vec<AccountVaultUpdate>,
}

impl PublicAccountDelta {
    /// Creates a new [`PublicAccountDelta`].
    pub fn new(
        new_header: AccountHeader,
        block_from: BlockNumber,
        block_to: BlockNumber,
        value_slot_updates: Vec<(StorageSlotName, Word)>,
        storage_map_updates: Vec<StorageMapUpdate>,
        vault_updates: Vec<AccountVaultUpdate>,
    ) -> Self {
        Self {
            new_header,
            block_from,
            block_to,
            value_slot_updates,
            storage_map_updates,
            vault_updates,
        }
    }

    /// Returns the account ID this delta applies to.
    pub fn id(&self) -> AccountId {
        self.new_header.id()
    }

    /// Returns the new account header that this delta advances the local state to.
    pub fn new_header(&self) -> &AccountHeader {
        &self.new_header
    }

    /// Returns the first block of the synced range.
    pub fn block_from(&self) -> BlockNumber {
        self.block_from
    }

    /// Returns the last block of the synced range.
    pub fn block_to(&self) -> BlockNumber {
        self.block_to
    }

    /// Computes the [`AccountDelta`] implied by this payload by replaying the carried
    /// incremental updates against the locally-stored account state.
    pub fn compute_account_delta(
        &self,
        local_header: &AccountHeader,
        local_storage: &AccountStorage,
        local_vault: &AssetVault,
    ) -> Result<AccountDelta, AccountDeltaError> {
        let storage_delta = replay_storage_updates(
            local_storage,
            &self.value_slot_updates,
            &self.storage_map_updates,
        )?;
        let vault_delta = replay_vault_updates(local_vault, &self.vault_updates)?;

        let old_nonce = local_header.nonce().as_canonical_u64();
        let new_nonce = self.new_header.nonce().as_canonical_u64();
        let nonce_delta = Felt::new(new_nonce.saturating_sub(old_nonce));

        AccountDelta::new(self.new_header.id(), storage_delta, vault_delta, nonce_delta)
    }
}

// DELTA REPLAY HELPERS
// ================================================================================================

/// Computes a storage delta by replaying incremental updates onto the locally-stored state.
fn replay_storage_updates(
    local_storage: &AccountStorage,
    value_slot_updates: &[(StorageSlotName, Word)],
    storage_map_updates: &[StorageMapUpdate],
) -> Result<AccountStorageDelta, AccountDeltaError> {
    let mut storage_delta = AccountStorageDelta::new();

    // Value slots: emit only the slots whose new value differs from local.
    for (slot_name, new_value) in value_slot_updates {
        let local_value = local_storage.get_item(slot_name).ok();
        if local_value.as_ref() != Some(new_value) {
            storage_delta.set_item(slot_name.clone(), *new_value)?;
        }
    }

    // Map slots: dedup updates per (slot, key) keeping the latest value by block number.
    let mut by_slot: BTreeMap<StorageSlotName, BTreeMap<StorageMapKey, Word>> = BTreeMap::new();
    let mut sorted: Vec<&StorageMapUpdate> = storage_map_updates.iter().collect();
    sorted.sort_by_key(|u| u.block_num);
    for update in sorted {
        by_slot
            .entry(update.slot_name.clone())
            .or_default()
            .insert(update.key, update.value);
    }
    for (slot_name, entries) in by_slot {
        for (key, value) in entries {
            storage_delta.set_map_item(slot_name.clone(), key, value)?;
        }
    }

    Ok(storage_delta)
}

/// Computes a vault delta by replaying incremental updates onto the locally-stored vault.
fn replay_vault_updates(
    local_vault: &AssetVault,
    vault_updates: &[AccountVaultUpdate],
) -> Result<AccountVaultDelta, AccountDeltaError> {
    let mut vault_delta = AccountVaultDelta::default();

    let mut final_vault: BTreeMap<AssetVaultKey, Asset> =
        local_vault.assets().map(|asset| (asset.vault_key(), asset)).collect();

    let mut sorted: Vec<&AccountVaultUpdate> = vault_updates.iter().collect();
    sorted.sort_by_key(|u| u.block_num);
    for update in sorted {
        match update.asset {
            Some(asset) => {
                final_vault.insert(update.vault_key, asset);
            },
            None => {
                final_vault.remove(&update.vault_key);
            },
        }
    }

    let local_assets: BTreeMap<AssetVaultKey, Asset> =
        local_vault.assets().map(|a| (a.vault_key(), a)).collect();
    for (key, final_asset) in &final_vault {
        match local_assets.get(key) {
            None => {
                vault_delta.add_asset(*final_asset)?;
            },
            Some(local_asset) if local_asset != final_asset => {
                vault_delta.remove_asset(*local_asset)?;
                vault_delta.add_asset(*final_asset)?;
            },
            _ => {},
        }
    }
    for (key, local_asset) in &local_assets {
        if !final_vault.contains_key(key) {
            vault_delta.remove_asset(*local_asset)?;
        }
    }

    Ok(vault_delta)
}

// ACCOUNT UPDATES
// ================================================================================================

/// Contains account changes to apply to the store after a sync request.
#[derive(Default)]
#[allow(clippy::struct_field_names)]
pub struct AccountUpdates {
    /// Updated public accounts, either as full state replacements or incremental deltas.
    updated_public_accounts: Vec<PublicAccountUpdate>,
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
        updated_public_accounts: Vec<PublicAccountUpdate>,
        mismatched_private_accounts: Vec<(AccountId, Word)>,
    ) -> Self {
        Self {
            updated_public_accounts,
            mismatched_private_accounts,
        }
    }

    /// Returns the updated public accounts.
    pub fn updated_public_accounts(&self) -> &[PublicAccountUpdate] {
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
