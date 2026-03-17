//! Provides the client APIs for synchronizing the client's local state with the Miden
//! network. It ensures that the client maintains a valid, up-to-date view of the chain.
//!
//! ## Overview
//!
//! This module handles the synchronization process between the local client and the Miden network.
//! The sync operation involves:
//!
//! - Querying the Miden node for state updates using tracked account IDs, note tags, and nullifier
//!   prefixes.
//! - Processing the received data to update note inclusion proofs, reconcile note state (new,
//!   committed, or consumed), and update account states.
//! - Incorporating new block headers and updating the local Merkle Mountain Range (MMR) with new
//!   peaks and authentication nodes.
//! - Aggregating transaction updates to determine which transactions have been committed or
//!   discarded.
//!
//! The result of the synchronization process is captured in a [`SyncSummary`], which provides
//! a summary of the new block number along with lists of received, committed, and consumed note
//! IDs, updated account IDs, locked accounts, and committed transaction IDs.
//!
//! Once the data is requested and retrieved, updates are persisted in the client's store.
//!
//! ## Examples
//!
//! The following example shows how to initiate a state sync and handle the resulting summary:
//!
//! ```rust
//! # use miden_client::auth::TransactionAuthenticator;
//! # use miden_client::sync::SyncSummary;
//! # use miden_client::{Client, ClientError};
//! # use miden_protocol::{block::BlockHeader, Felt, Word, StarkField};
//! # use miden_protocol::crypto::rand::FeltRng;
//! # async fn run_sync<AUTH: TransactionAuthenticator + Sync + 'static>(client: &mut Client<AUTH>) -> Result<(), ClientError> {
//! // Attempt to synchronize the client's state with the Miden network.
//! // The requested data is based on the client's state: it gets updates for accounts, relevant
//! // notes, etc. For more information on the data that gets requested, see the doc comments for
//! // `sync_state()`.
//! let sync_summary: SyncSummary = client.sync_state().await?;
//!
//! println!("Synced up to block number: {}", sync_summary.block_num);
//! println!("Committed notes: {}", sync_summary.committed_notes.len());
//! println!("Consumed notes: {}", sync_summary.consumed_notes.len());
//! println!("Updated accounts: {}", sync_summary.updated_accounts.len());
//! println!("Locked accounts: {}", sync_summary.locked_accounts.len());
//! println!("Committed transactions: {}", sync_summary.committed_transactions.len());
//!
//! Ok(())
//! # }
//! ```
//!
//! The `sync_state` method loops internally until the client is fully synced to the network tip.
//!
//! For more advanced usage, refer to the individual functions (such as
//! `committed_note_updates` and `consumed_note_updates`) to understand how the sync data is
//! processed and applied to the local store.

use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cmp::max;

use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::{NoteId, NoteTag};
use miden_protocol::transaction::TransactionId;
use miden_tx::auth::TransactionAuthenticator;
use miden_tx::utils::{Deserializable, DeserializationError, Serializable};
use tracing::{debug, info};

use crate::note::NoteScreener;
use crate::store::{NoteFilter, TransactionFilter};
use crate::{Client, ClientError};
mod block_header;

mod tag;
pub use tag::{NoteTagRecord, NoteTagSource};

mod state_sync;
pub use state_sync::{NoteUpdateAction, OnNoteReceived, StateSync};

mod state_sync_update;
pub use state_sync_update::{
    AccountUpdates,
    BlockUpdates,
    StateSyncUpdate,
    TransactionUpdateTracker,
};

/// Client synchronization methods.
impl<AUTH> Client<AUTH>
where
    AUTH: TransactionAuthenticator + Sync + 'static,
{
    // SYNC STATE
    // --------------------------------------------------------------------------------------------

    /// Returns the block number of the last state sync block.
    pub async fn get_sync_height(&self) -> Result<BlockNumber, ClientError> {
        self.store.get_sync_height().await.map_err(Into::into)
    }

    /// Syncs the client's state with the current state of the Miden network and returns a
    /// [`SyncSummary`] corresponding to the local state update.
    ///
    /// The sync process is done in multiple steps:
    /// 1. A request is sent to the node to get the state updates. This request includes tracked
    ///    account IDs and the tags of notes that might have changed or that might be of interest to
    ///    the client.
    /// 2. A response is received with the current state of the network. The response includes
    ///    information about new/committed/consumed notes, updated accounts, and committed
    ///    transactions.
    /// 3. Tracked notes are updated with their new states.
    /// 4. New notes are checked, and only relevant ones are stored. Relevant notes are those that
    ///    can be consumed by accounts the client is tracking (this is checked by the
    ///    [`crate::note::NoteScreener`])
    /// 5. Transactions are updated with their new states.
    /// 6. Tracked public accounts are updated and private accounts are validated against the node
    ///    state.
    /// 7. The MMR is updated with the new peaks and authentication nodes.
    /// 8. All updates are applied to the store to be persisted.
    pub async fn sync_state(&mut self) -> Result<SyncSummary, ClientError> {
        _ = self.ensure_genesis_in_place().await?;

        let note_screener = NoteScreener::new(self.store.clone());
        let state_sync =
            StateSync::new(self.rpc_api.clone(), Arc::new(note_screener), self.tx_graceful_blocks);

        // Get current state of the client
        let accounts = self
            .store
            .get_account_headers()
            .await?
            .into_iter()
            .map(|(acc_header, _)| acc_header)
            .collect();

        let note_tags: BTreeSet<NoteTag> = self.store.get_unique_note_tags().await?;

        // Note Transport update
        // TODO We can run both sync_state, fetch_transport_notes futures in parallel
        if self.is_note_transport_enabled() {
            let cursor = self.store.get_note_transport_cursor().await?;
            self.fetch_transport_notes(cursor, note_tags.clone()).await?;
        }

        let unspent_input_notes = self.store.get_input_notes(NoteFilter::Unspent).await?;
        let unspent_output_notes = self.store.get_output_notes(NoteFilter::Unspent).await?;

        let uncommitted_transactions =
            self.store.get_transactions(TransactionFilter::Uncommitted).await?;

        // Build current partial MMR
        let current_partial_mmr = self.store.get_current_partial_mmr().await?;

        // Get the sync update from the network
        let state_sync_update: StateSyncUpdate = state_sync
            .sync_state(
                current_partial_mmr,
                accounts,
                note_tags,
                unspent_input_notes,
                unspent_output_notes,
                uncommitted_transactions,
            )
            .await?;

        let sync_summary: SyncSummary = (&state_sync_update).into();
        debug!(sync_summary = ?sync_summary, "Sync summary computed");
        info!("Applying changes to the store.");

        // Apply received and computed updates to the store
        self.store
            .apply_state_sync(state_sync_update)
            .await
            .map_err(ClientError::StoreError)?;

        // Verify stale Expected notes BEFORE pruning block headers.
        // Checks all Expected notes whose commitment block has already been synced.
        // This is a single get_notes_by_id RPC call regardless of note count.
        self.verify_stale_expected_notes().await?;

        // Remove irrelevant block headers
        self.store.prune_irrelevant_blocks().await?;

        Ok(sync_summary)
    }

    /// Checks for Expected notes whose commitment block has already been synced and fetches
    /// their inclusion proofs from the node to transition them to Committed.
    ///
    /// This handles the race condition where NTL delivers note data after the on-chain
    /// commitment was already synced. Uses a single `get_notes_by_id` RPC call for all
    /// stale notes regardless of count.
    async fn verify_stale_expected_notes(&mut self) -> Result<(), ClientError> {
        use crate::rpc::domain::note::FetchedNote;
        use crate::store::InputNoteState;

        let sync_height = self.store.get_sync_height().await?;
        let expected_notes = self.store.get_input_notes(NoteFilter::Expected).await?;

        // Find Expected notes whose commitment block has already been synced
        let stale_note_ids: Vec<NoteId> = expected_notes
            .iter()
            .filter(|note| match note.state() {
                InputNoteState::Expected(state) => state.after_block_num < sync_height,
                _ => false,
            })
            .map(|note| note.id())
            .collect();

        if stale_note_ids.is_empty() {
            return Ok(());
        }

        info!(
            "Found {} stale Expected notes behind sync height {}, fetching inclusion proofs...",
            stale_note_ids.len(),
            sync_height
        );

        // Fetch inclusion proofs from the node
        let fetched_notes = match self.rpc_api.get_notes_by_id(&stale_note_ids).await {
            Ok(notes) => notes,
            Err(e) => {
                info!("Failed to fetch inclusion proofs for stale notes: {}", e);
                return Ok(());
            },
        };

        for fetched_note in fetched_notes {
            let (note_id, inclusion_proof, metadata) = match &fetched_note {
                FetchedNote::Private(header, proof) => {
                    (header.id(), proof.clone(), header.metadata().clone())
                },
                FetchedNote::Public(note, proof) => {
                    (note.id(), proof.clone(), note.metadata().clone())
                },
            };

            // Get the block header for verification
            let block_num = inclusion_proof.location().block_num();
            let block_headers =
                self.store.get_block_headers(&BTreeSet::from([block_num])).await?;

            let Some((block_header, _)) = block_headers.into_iter().next() else {
                debug!(
                    "Block header {} not stored locally for note {}, will retry on next sync",
                    block_num, note_id
                );
                continue;
            };

            // Re-read the note from the store
            let mut notes =
                self.store.get_input_notes(NoteFilter::List(vec![note_id])).await?;
            let Some(mut note_record) = notes.pop() else {
                continue;
            };

            // Transition the note: Expected → Unverified → Committed
            if let Err(e) = note_record.inclusion_proof_received(inclusion_proof, metadata) {
                info!("Failed to apply inclusion proof for note {}: {}", note_id, e);
                continue;
            }

            if let Err(e) = note_record.block_header_received(&block_header) {
                info!("Failed to verify note {} against block header: {}", note_id, e);
                continue;
            }

            // Persist the updated note state
            self.store.upsert_input_notes(&[note_record]).await?;

            // Mark the block as having client notes so it survives pruning
            // and is available in the partial MMR for transaction execution.
            let current_partial_mmr = self.store.get_current_partial_mmr().await?;
            self.store
                .insert_block_header(&block_header, current_partial_mmr.peaks(), true)
                .await?;

            info!("Transitioned stale Expected note {} to Committed", note_id);
        }

        Ok(())
    }

    /// Applies the state sync update to the store.
    ///
    /// See [`crate::Store::apply_state_sync()`] for what the update implies.
    pub async fn apply_state_sync(&mut self, update: StateSyncUpdate) -> Result<(), ClientError> {
        self.store.apply_state_sync(update).await.map_err(ClientError::StoreError)?;

        // Remove irrelevant block headers
        self.store.prune_irrelevant_blocks().await.map_err(ClientError::StoreError)
    }
}

// SYNC SUMMARY
// ================================================================================================

/// Contains stats about the sync operation.
#[derive(Debug, PartialEq)]
pub struct SyncSummary {
    /// Block number up to which the client has been synced.
    pub block_num: BlockNumber,
    /// IDs of new public notes that the client has received.
    pub new_public_notes: Vec<NoteId>,
    /// IDs of tracked notes that have been committed.
    pub committed_notes: Vec<NoteId>,
    /// IDs of notes that have been consumed.
    pub consumed_notes: Vec<NoteId>,
    /// IDs of on-chain accounts that have been updated.
    pub updated_accounts: Vec<AccountId>,
    /// IDs of private accounts that have been locked.
    pub locked_accounts: Vec<AccountId>,
    /// IDs of committed transactions.
    pub committed_transactions: Vec<TransactionId>,
}

impl SyncSummary {
    pub fn new(
        block_num: BlockNumber,
        new_public_notes: Vec<NoteId>,
        committed_notes: Vec<NoteId>,
        consumed_notes: Vec<NoteId>,
        updated_accounts: Vec<AccountId>,
        locked_accounts: Vec<AccountId>,
        committed_transactions: Vec<TransactionId>,
    ) -> Self {
        Self {
            block_num,
            new_public_notes,
            committed_notes,
            consumed_notes,
            updated_accounts,
            locked_accounts,
            committed_transactions,
        }
    }

    pub fn new_empty(block_num: BlockNumber) -> Self {
        Self {
            block_num,
            new_public_notes: vec![],
            committed_notes: vec![],
            consumed_notes: vec![],
            updated_accounts: vec![],
            locked_accounts: vec![],
            committed_transactions: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.new_public_notes.is_empty()
            && self.committed_notes.is_empty()
            && self.consumed_notes.is_empty()
            && self.updated_accounts.is_empty()
            && self.locked_accounts.is_empty()
            && self.committed_transactions.is_empty()
    }

    pub fn combine_with(&mut self, mut other: Self) {
        self.block_num = max(self.block_num, other.block_num);
        self.new_public_notes.append(&mut other.new_public_notes);
        self.committed_notes.append(&mut other.committed_notes);
        self.consumed_notes.append(&mut other.consumed_notes);
        self.updated_accounts.append(&mut other.updated_accounts);
        self.locked_accounts.append(&mut other.locked_accounts);
        self.committed_transactions.append(&mut other.committed_transactions);
    }
}

impl Serializable for SyncSummary {
    fn write_into<W: miden_tx::utils::ByteWriter>(&self, target: &mut W) {
        self.block_num.write_into(target);
        self.new_public_notes.write_into(target);
        self.committed_notes.write_into(target);
        self.consumed_notes.write_into(target);
        self.updated_accounts.write_into(target);
        self.locked_accounts.write_into(target);
        self.committed_transactions.write_into(target);
    }
}

impl Deserializable for SyncSummary {
    fn read_from<R: miden_tx::utils::ByteReader>(
        source: &mut R,
    ) -> Result<Self, DeserializationError> {
        let block_num = BlockNumber::read_from(source)?;
        let new_public_notes = Vec::<NoteId>::read_from(source)?;
        let committed_notes = Vec::<NoteId>::read_from(source)?;
        let consumed_notes = Vec::<NoteId>::read_from(source)?;
        let updated_accounts = Vec::<AccountId>::read_from(source)?;
        let locked_accounts = Vec::<AccountId>::read_from(source)?;
        let committed_transactions = Vec::<TransactionId>::read_from(source)?;

        Ok(Self {
            block_num,
            new_public_notes,
            committed_notes,
            consumed_notes,
            updated_accounts,
            locked_accounts,
            committed_transactions,
        })
    }
}
