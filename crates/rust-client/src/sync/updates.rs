use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::{InOrderIndex, MmrPeaks};
use miden_protocol::note::NoteId;

use super::{SyncSummary, TransactionUpdateTracker};
use crate::account::Account;
use crate::note::{NoteUpdateTracker, NoteUpdateType};

// STATE SYNC UPDATE
// ================================================================================================

/// Contains all information needed to apply the update in the store after syncing with the node.
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

        let committed_note_ids: alloc::collections::BTreeSet<NoteId> = value
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

        let consumed_note_ids: alloc::collections::BTreeSet<NoteId> = value
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

// BLOCK UPDATES
// ================================================================================================

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
