//! Defines the storage interfaces used by the Miden client.
//!
//! It provides mechanisms for persisting and retrieving data, such as account states, transaction
//! history, block headers, notes, and MMR nodes.
//!
//! ## Overview
//!
//! The storage module is central to the Miden client’s persistence layer. It defines the
//! [`Store`] trait which abstracts over any concrete storage implementation. The trait exposes
//! methods to (among others):
//!
//! - Retrieve and update transactions, notes, and accounts.
//! - Store and query block headers along with MMR peaks and authentication nodes.
//! - Manage note tags for synchronizing with the node.
//!
//! These are all used by the Miden client to provide transaction execution in the correct contexts.
//!
//! In addition to the main [`Store`] trait, the module provides types for filtering queries, such
//! as [`TransactionFilter`] and [`NoteFilter`], to narrow down the set of returned transactions or
//! notes. For more advanced usage, see the documentation of individual methods in the [`Store`]
//! trait.

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Debug;

use miden_objects::account::{
    Account,
    AccountCode,
    AccountHeader,
    AccountId,
    AccountIdPrefix,
    AccountStorage,
    StorageMapWitness,
    StorageSlot,
};
use miden_objects::address::Address;
use miden_objects::asset::{Asset, AssetVault, AssetWitness};
use miden_objects::block::{BlockHeader, BlockNumber};
use miden_objects::crypto::merkle::{InOrderIndex, MmrPeaks, PartialMmr};
use miden_objects::note::{NoteId, NoteTag, Nullifier};
use miden_objects::transaction::TransactionId;
use miden_objects::{AccountError, Word};

use crate::note_transport::{
    NOTE_TRANSPORT_CURSOR_STORE_SETTING,
    NoteTransportCursor,
    NoteTransportUpdate,
};
use crate::sync::{NoteTagRecord, StateSyncUpdate};
use crate::transaction::{TransactionRecord, TransactionStoreUpdate};

/// Contains [`ClientDataStore`] to automatically implement [`DataStore`] for anything that
/// implements [`Store`]. This isn't public because it's an implementation detail to instantiate the
/// executor.
///
/// The user is tasked with creating a [`Store`] which the client will wrap into a
/// [`ClientDataStore`] at creation time.
pub(crate) mod data_store;

mod errors;
pub use errors::*;

mod account;
pub use account::{AccountRecord, AccountStatus, AccountUpdates};
mod note_record;
pub use note_record::{
    InputNoteRecord,
    InputNoteState,
    NoteExportType,
    NoteRecordError,
    NoteScriptRecord,
    OutputNoteRecord,
    OutputNoteState,
    input_note_states,
};

// STORE TRAIT
// ================================================================================================

/// The [`Store`] trait exposes all methods that the client store needs in order to track the
/// current state.
///
/// All update functions are implied to be atomic. That is, if multiple entities are meant to be
/// updated as part of any single function and an error is returned during its execution, any
/// changes that might have happened up to that point need to be rolled back and discarded.
///
/// Because the [`Store`]'s ownership is shared between the executor and the client, interior
/// mutability is expected to be implemented, which is why all methods receive `&self` and
/// not `&mut self`.
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait Store: Send + Sync {
    /// Returns the current timestamp tracked by the store, measured in non-leap seconds since
    /// Unix epoch. If the store implementation is incapable of tracking time, it should return
    /// `None`.
    ///
    /// This method is used to add time metadata to notes' states. This information doesn't have a
    /// functional impact on the client's operation, it's shown to the user for informational
    /// purposes.
    fn get_current_timestamp(&self) -> Option<u64>;

    // TRANSACTIONS
    // --------------------------------------------------------------------------------------------

    /// Retrieves stored transactions, filtered by [`TransactionFilter`].
    async fn get_transactions(
        &self,
        filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, StoreError>;

    /// Applies a transaction, atomically updating the current state based on the
    /// [`TransactionStoreUpdate`].
    ///
    /// An update involves:
    /// - Updating the stored account which is being modified by the transaction.
    /// - Storing new input/output notes and payback note details as a result of the transaction
    ///   execution.
    /// - Updating the input notes that are being processed by the transaction.
    /// - Inserting the new tracked tags into the store.
    /// - Inserting the transaction into the store to track.
    async fn apply_transaction(&self, tx_update: TransactionStoreUpdate) -> Result<(), StoreError>;

    // NOTES
    // --------------------------------------------------------------------------------------------

    /// Retrieves the input notes from the store.
    async fn get_input_notes(&self, filter: NoteFilter)
    -> Result<Vec<InputNoteRecord>, StoreError>;

    /// Retrieves the output notes from the store.
    async fn get_output_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<OutputNoteRecord>, StoreError>;

    /// Returns the nullifiers of all unspent input notes.
    ///
    /// The default implementation of this method uses [`Store::get_input_notes`].
    async fn get_unspent_input_note_nullifiers(&self) -> Result<Vec<Nullifier>, StoreError> {
        self.get_input_notes(NoteFilter::Unspent)
            .await?
            .iter()
            .map(|input_note| Ok(input_note.nullifier()))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Inserts the provided input notes into the database. If a note with the same ID already
    /// exists, it will be replaced.
    async fn upsert_input_notes(&self, notes: &[InputNoteRecord]) -> Result<(), StoreError>;

    /// Returns the note script associated with the given root.
    async fn get_note_script(&self, script_root: Word) -> Result<NoteScriptRecord, StoreError>;

    /// Inserts the provided note scripts into the database. If a script with the same root already
    /// exists, it will be replaced.
    async fn upsert_note_scripts(&self, note_script: &[NoteScriptRecord])
    -> Result<(), StoreError>;

    // CHAIN DATA
    // --------------------------------------------------------------------------------------------

    /// Retrieves a vector of [`BlockHeader`]s filtered by the provided block numbers.
    ///
    /// The returned vector may not contain some or all of the requested block headers. It's up to
    /// the callee to check whether all requested block headers were found.
    ///
    /// For each block header an additional boolean value is returned representing whether the block
    /// contains notes relevant to the client.
    async fn get_block_headers(
        &self,
        block_numbers: &BTreeSet<BlockNumber>,
    ) -> Result<Vec<(BlockHeader, BlockRelevance)>, StoreError>;

    /// Retrieves a [`BlockHeader`] corresponding to the provided block number and a boolean value
    /// that represents whether the block contains notes relevant to the client. Returns `None` if
    /// the block is not found.
    ///
    /// The default implementation of this method uses [`Store::get_block_headers`].
    async fn get_block_header_by_num(
        &self,
        block_number: BlockNumber,
    ) -> Result<Option<(BlockHeader, BlockRelevance)>, StoreError> {
        self.get_block_headers(&[block_number].into_iter().collect())
            .await
            .map(|mut block_headers_list| block_headers_list.pop())
    }

    /// Retrieves a list of [`BlockHeader`] that include relevant notes to the client.
    async fn get_tracked_block_headers(&self) -> Result<Vec<BlockHeader>, StoreError>;

    /// Retrieves all MMR authentication nodes based on [`PartialBlockchainFilter`].
    async fn get_partial_blockchain_nodes(
        &self,
        filter: PartialBlockchainFilter,
    ) -> Result<BTreeMap<InOrderIndex, Word>, StoreError>;

    /// Inserts blockchain MMR authentication nodes.
    ///
    /// In the case where the [`InOrderIndex`] already exists on the table, the insertion is
    /// ignored.
    async fn insert_partial_blockchain_nodes(
        &self,
        nodes: &[(InOrderIndex, Word)],
    ) -> Result<(), StoreError>;

    /// Returns peaks information from the blockchain by a specific block number.
    ///
    /// If there is no partial blockchain info stored for the provided block returns an empty
    /// [`MmrPeaks`].
    async fn get_partial_blockchain_peaks_by_block_num(
        &self,
        block_num: BlockNumber,
    ) -> Result<MmrPeaks, StoreError>;

    /// Inserts a block header into the store, alongside peaks information at the block's height.
    ///
    /// `has_client_notes` describes whether the block has relevant notes to the client; this means
    /// the client might want to authenticate merkle paths based on this value.
    /// If the block header exists and `has_client_notes` is `true` then the `has_client_notes`
    /// column is updated to `true` to signify that the block now contains a relevant note.
    async fn insert_block_header(
        &self,
        block_header: &BlockHeader,
        partial_blockchain_peaks: MmrPeaks,
        has_client_notes: bool,
    ) -> Result<(), StoreError>;

    /// Removes block headers that do not contain any client notes and aren't the genesis or last
    /// block.
    async fn prune_irrelevant_blocks(&self) -> Result<(), StoreError>;

    // ACCOUNT
    // --------------------------------------------------------------------------------------------

    /// Returns the account IDs of all accounts stored in the database.
    async fn get_account_ids(&self) -> Result<Vec<AccountId>, StoreError>;

    /// Returns a list of [`AccountHeader`] of all accounts stored in the database along with their
    /// statuses.
    ///
    /// Said accounts' state is the state after the last performed sync.
    async fn get_account_headers(&self) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError>;

    /// Retrieves an [`AccountHeader`] object for the specified [`AccountId`] along with its status.
    /// Returns `None` if the account is not found.
    ///
    /// Said account's state is the state according to the last sync performed.
    async fn get_account_header(
        &self,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError>;

    /// Returns an [`AccountHeader`] corresponding to the stored account state that matches the
    /// given commitment. If no account state matches the provided commitment, `None` is returned.
    async fn get_account_header_by_commitment(
        &self,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError>;

    /// Retrieves a full [`AccountRecord`] object, this contains the account's latest state along
    /// with its status. Returns `None` if the account is not found.
    async fn get_account(&self, account_id: AccountId)
    -> Result<Option<AccountRecord>, StoreError>;

    /// Inserts an [`Account`] to the store.
    /// Receives an [`Address`] as the initial address to associate with the account. This address
    /// will be tracked for incoming notes and its derived note tag will be monitored.
    ///
    /// # Errors
    ///
    /// - If the account is new and does not contain a seed
    async fn insert_account(
        &self,
        account: &Account,
        initial_address: Address,
    ) -> Result<(), StoreError>;

    /// Upserts the account code for a foreign account. This value will be used as a cache of known
    /// script roots and added to the `GetForeignAccountCode` request.
    async fn upsert_foreign_account_code(
        &self,
        account_id: AccountId,
        code: AccountCode,
    ) -> Result<(), StoreError>;

    /// Retrieves the cached account code for various foreign accounts.
    async fn get_foreign_account_code(
        &self,
        account_ids: Vec<AccountId>,
    ) -> Result<BTreeMap<AccountId, AccountCode>, StoreError>;

    /// Retrieves all [`Address`] objects that correspond to the provided account ID.
    async fn get_addresses_by_account_id(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<Address>, StoreError>;

    /// Updates an existing [`Account`] with a new state.
    ///
    /// # Errors
    ///
    /// Returns a `StoreError::AccountDataNotFound` if there is no account for the provided ID.
    async fn update_account(&self, new_account_state: &Account) -> Result<(), StoreError>;

    // SETTINGS
    // --------------------------------------------------------------------------------------------

    /// Adds a value to the `settings` table.
    async fn set_setting(&self, key: String, value: Vec<u8>) -> Result<(), StoreError>;

    /// Retrieves a value from the `settings` table.
    async fn get_setting(&self, key: String) -> Result<Option<Vec<u8>>, StoreError>;

    /// Deletes a value from the `settings` table.
    async fn remove_setting(&self, key: String) -> Result<(), StoreError>;

    /// Returns all the keys from the `settings` table.
    async fn list_setting_keys(&self) -> Result<Vec<String>, StoreError>;

    // SYNC
    // --------------------------------------------------------------------------------------------

    /// Returns the note tag records that the client is interested in.
    async fn get_note_tags(&self) -> Result<Vec<NoteTagRecord>, StoreError>;

    /// Returns the unique note tags (without source) that the client is interested in.
    async fn get_unique_note_tags(&self) -> Result<BTreeSet<NoteTag>, StoreError> {
        Ok(self.get_note_tags().await?.into_iter().map(|r| r.tag).collect())
    }

    /// Adds a note tag to the list of tags that the client is interested in.
    ///
    /// If the tag was already being tracked, returns false since no new tags were actually added.
    /// Otherwise true.
    async fn add_note_tag(&self, tag: NoteTagRecord) -> Result<bool, StoreError>;

    /// Removes a note tag from the list of tags that the client is interested in.
    ///
    /// If the tag wasn't present in the store returns false since no tag was actually removed.
    /// Otherwise returns true.
    async fn remove_note_tag(&self, tag: NoteTagRecord) -> Result<usize, StoreError>;

    /// Returns the block number of the last state sync block.
    async fn get_sync_height(&self) -> Result<BlockNumber, StoreError>;

    /// Applies the state sync update to the store. An update involves:
    ///
    /// - Inserting the new block header to the store alongside new MMR peaks information.
    /// - Updating the corresponding tracked input/output notes.
    /// - Removing note tags that are no longer relevant.
    /// - Updating transactions in the store, marking as `committed` or `discarded`.
    ///   - In turn, validating private account's state transitions. If a private account's
    ///     commitment locally does not match the `StateSyncUpdate` information, the account may be
    ///     locked.
    /// - Storing new MMR authentication nodes.
    /// - Updating the tracked public accounts.
    async fn apply_state_sync(&self, state_sync_update: StateSyncUpdate) -> Result<(), StoreError>;

    // TRANSPORT
    // --------------------------------------------------------------------------------------------

    /// Gets the note transport cursor.
    ///
    /// This is used to reduce the number of fetched notes from the note transport network.
    async fn get_note_transport_cursor(&self) -> Result<NoteTransportCursor, StoreError> {
        let cursor_bytes = self
            .get_setting(NOTE_TRANSPORT_CURSOR_STORE_SETTING.into())
            .await?
            .ok_or_else(|| StoreError::NoteTransportCursorNotFound)?;
        let array: [u8; 8] = cursor_bytes
            .as_slice()
            .try_into()
            .map_err(|e: core::array::TryFromSliceError| StoreError::ParsingError(e.to_string()))?;
        let cursor = u64::from_be_bytes(array);
        Ok(cursor.into())
    }

    /// Updates the note transport cursor.
    ///
    /// This is used to track the last cursor position when fetching notes from the note transport
    /// network.
    async fn update_note_transport_cursor(
        &self,
        cursor: NoteTransportCursor,
    ) -> Result<(), StoreError> {
        let cursor_bytes = cursor.value().to_be_bytes().to_vec();
        self.set_setting(NOTE_TRANSPORT_CURSOR_STORE_SETTING.into(), cursor_bytes)
            .await?;
        Ok(())
    }

    /// Applies a note transport update
    ///
    /// An update involves:
    /// - Insert fetched notes;
    /// - Update pagination cursor used in note fetching.
    async fn apply_note_transport_update(
        &self,
        note_transport_update: NoteTransportUpdate,
    ) -> Result<(), StoreError> {
        self.update_note_transport_cursor(note_transport_update.cursor).await?;
        self.upsert_input_notes(&note_transport_update.note_updates).await?;
        Ok(())
    }

    // PARTIAL MMR
    // --------------------------------------------------------------------------------------------

    /// Builds the current view of the chain's [`PartialMmr`]. Because we want to add all new
    /// authentication nodes that could come from applying the MMR updates, we need to track all
    /// known leaves thus far.
    ///
    /// The default implementation is based on [`Store::get_partial_blockchain_nodes`],
    /// [`Store::get_partial_blockchain_peaks_by_block_num`] and [`Store::get_block_header_by_num`]
    async fn get_current_partial_mmr(&self) -> Result<PartialMmr, StoreError> {
        let current_block_num = self.get_sync_height().await?;

        let tracked_nodes = self.get_partial_blockchain_nodes(PartialBlockchainFilter::All).await?;
        let current_peaks =
            self.get_partial_blockchain_peaks_by_block_num(current_block_num).await?;

        // FIXME: Because each block stores the peaks for the MMR for the leaf of pos `block_num-1`,
        // we can get an MMR based on those peaks, add the current block number and align it with
        // the set of all nodes in the store.
        // Otherwise, by doing `PartialMmr::from_parts` we would effectively have more nodes than
        // we need for the passed peaks. The alternative here is to truncate the set of all nodes
        // before calling `from_parts`
        //
        // This is a bit hacky but it works. One alternative would be to _just_ get nodes required
        // for tracked blocks in the MMR. This would however block us from the convenience of
        // just getting all nodes from the store.

        let (current_block, has_client_notes) = self
            .get_block_header_by_num(current_block_num)
            .await?
            .expect("Current block should be in the store");

        let mut current_partial_mmr = PartialMmr::from_peaks(current_peaks);
        let has_client_notes = has_client_notes.into();
        current_partial_mmr.add(current_block.commitment(), has_client_notes);

        let current_partial_mmr =
            PartialMmr::from_parts(current_partial_mmr.peaks(), tracked_nodes, has_client_notes);

        Ok(current_partial_mmr)
    }

    // ACCOUNT VAULT AND STORE
    // --------------------------------------------------------------------------------------------

    /// Retrieves the asset vault for a specific account.
    async fn get_account_vault(&self, account_id: AccountId) -> Result<AssetVault, StoreError>;

    /// Retrieves a specific asset from the account's vault along with its Merkle witness.
    ///
    /// The default implementation of this method uses [`Store::get_account_vault`].
    async fn get_account_asset(
        &self,
        account_id: AccountId,
        faucet_id_prefix: AccountIdPrefix,
    ) -> Result<Option<(Asset, AssetWitness)>, StoreError> {
        let vault = self.get_account_vault(account_id).await?;
        let Some(asset) = vault.assets().find(|a| a.faucet_id_prefix() == faucet_id_prefix) else {
            return Ok(None);
        };

        let witness = AssetWitness::new(vault.open(asset.vault_key()).into())?;

        Ok(Some((asset, witness)))
    }

    /// Retrieves the storage for a specific account.
    async fn get_account_storage(
        &self,
        account_id: AccountId,
    ) -> Result<AccountStorage, StoreError>;

    /// Retrieves a specific item from the account's storage map along with its Merkle proof.
    ///
    /// The default implementation of this method uses [`Store::get_account_storage`].
    async fn get_account_map_item(
        &self,
        account_id: AccountId,
        index: u8,
        key: Word,
    ) -> Result<(Word, StorageMapWitness), StoreError> {
        let storage = self.get_account_storage(account_id).await?;
        let Some(StorageSlot::Map(map)) = storage.slots().get(index as usize) else {
            return Err(StoreError::AccountError(AccountError::StorageSlotNotMap(index)));
        };

        let value = map.get(&key);
        let witness = map.open(&key);

        Ok((value, witness))
    }
}

// PARTIAL BLOCKCHAIN NODE FILTER
// ================================================================================================

/// Filters for searching specific MMR nodes.
// TODO: Should there be filters for specific blocks instead of nodes?
pub enum PartialBlockchainFilter {
    /// Return all nodes.
    All,
    /// Filter by the specified in-order indices.
    List(Vec<InOrderIndex>),
}

// TRANSACTION FILTERS
// ================================================================================================

/// Filters for narrowing the set of transactions returned by the client's store.
#[derive(Debug, Clone)]
pub enum TransactionFilter {
    /// Return all transactions.
    All,
    /// Filter by transactions that haven't yet been committed to the blockchain as per the last
    /// sync.
    Uncommitted,
    /// Return a list of the transaction that matches the provided [`TransactionId`]s.
    Ids(Vec<TransactionId>),
    /// Return a list of the expired transactions that were executed before the provided
    /// [`BlockNumber`]. Transactions created after the provided block number are not
    /// considered.
    ///
    /// A transaction is considered expired if is uncommitted and the transaction's block number
    /// is less than the provided block number.
    ExpiredBefore(BlockNumber),
}

// NOTE FILTER
// ================================================================================================

/// Filters for narrowing the set of notes returned by the client's store.
#[derive(Debug, Clone)]
pub enum NoteFilter {
    /// Return a list of all notes ([`InputNoteRecord`] or [`OutputNoteRecord`]).
    All,
    /// Return a list of committed notes ([`InputNoteRecord`] or [`OutputNoteRecord`]). These
    /// represent notes that the blockchain has included in a block.
    Committed,
    /// Filter by consumed notes ([`InputNoteRecord`] or [`OutputNoteRecord`]). notes that have
    /// been used as inputs in transactions.
    Consumed,
    /// Return a list of expected notes ([`InputNoteRecord`] or [`OutputNoteRecord`]). These
    /// represent notes for which the store doesn't have anchor data.
    Expected,
    /// Return a list containing any notes that match with the provided [`NoteId`] vector.
    List(Vec<NoteId>),
    /// Return a list containing any notes that match the provided [`Nullifier`] vector.
    Nullifiers(Vec<Nullifier>),
    /// Return a list of notes that are currently being processed. This filter doesn't apply to
    /// output notes.
    Processing,
    /// Return a list containing the note that matches with the provided [`NoteId`]. The query will
    /// return an error if the note isn't found.
    Unique(NoteId),
    /// Return a list containing notes that haven't been nullified yet, this includes expected,
    /// committed, processing and unverified notes.
    Unspent,
    /// Return a list containing notes with unverified inclusion proofs. This filter doesn't apply
    /// to output notes.
    Unverified,
}

// BLOCK RELEVANCE
// ================================================================================================

/// Expresses metadata about the block header.
#[derive(Debug, Clone)]
pub enum BlockRelevance {
    /// The block header includes notes that the client may consume.
    HasNotes,
    /// The block header does not contain notes relevant to the client.
    Irrelevant,
}

impl From<BlockRelevance> for bool {
    fn from(val: BlockRelevance) -> Self {
        match val {
            BlockRelevance::HasNotes => true,
            BlockRelevance::Irrelevant => false,
        }
    }
}

impl From<bool> for BlockRelevance {
    fn from(has_notes: bool) -> Self {
        if has_notes {
            BlockRelevance::HasNotes
        } else {
            BlockRelevance::Irrelevant
        }
    }
}
