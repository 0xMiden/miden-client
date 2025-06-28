//! Provides an IndexedDB-backed implementation of the [Store] trait for web environments.
//!
//! This module enables persistence of client data (accounts, transactions, notes, block headers,
//! etc.) when running in a browser. It uses wasm-bindgen to interface with JavaScript and
//! `IndexedDB`, allowing the Miden client to store and retrieve data asynchronously.
//!
//! **Note:** This implementation is only available when targeting WebAssembly with the `web_store`
//! feature enabled.

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::ToString,
    vec::Vec,
};

use miden_objects::{
    Digest, Word,
    account::{Account, AccountCode, AccountHeader, AccountId},
    block::{BlockHeader, BlockNumber},
    crypto::merkle::{InOrderIndex, MmrPeaks},
    note::{NoteTag, Nullifier},
};
use tonic::async_trait;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, js_sys, wasm_bindgen};

use super::SqliteStore;
use crate::store::sqlite_store::db_management::{connect, utils::apply_migrations};
use crate::store::{
    AccountRecord, AccountStatus, InputNoteRecord, NoteFilter, OutputNoteRecord,
    PartialBlockchainFilter, Store, StoreError, TransactionFilter,
};
use crate::{
    sync::{NoteTagRecord, StateSyncUpdate},
    transaction::{TransactionRecord, TransactionStoreUpdate},
};
use rusqlite::Connection;
pub struct WebStore(Connection);

impl WebStore {
    pub async fn new() -> Result<WebStore, StoreError> {
        let mut connection =
            connect().await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        let _ = apply_migrations(&mut connection)
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        Ok(WebStore(connection))
    }
}

impl WebStore {
    pub fn get_current_timestamp(&mut self) -> Option<u64> {
        let now = chrono::Utc::now();
        Some(u64::try_from(now.timestamp()).expect("timestamp is always after epoch"))
    }

    // SYNC
    // --------------------------------------------------------------------------------------------
    pub fn get_note_tags(&mut self) -> Result<Vec<NoteTagRecord>, StoreError> {
        SqliteStore::get_note_tags(&mut self.0)
    }

    pub fn get_unique_note_tags(&mut self) -> Result<BTreeSet<NoteTag>, StoreError> {
        SqliteStore::get_unique_note_tags(&mut self.0)
    }

    pub fn add_note_tag(&mut self, tag: NoteTagRecord) -> Result<bool, StoreError> {
        SqliteStore::add_note_tag(&mut self.0, tag)
    }

    pub fn remove_note_tag(&mut self, tag: NoteTagRecord) -> Result<usize, StoreError> {
        SqliteStore::remove_note_tag(&mut self.0, tag)
    }

    pub fn get_sync_height(&mut self) -> Result<BlockNumber, StoreError> {
        SqliteStore::get_sync_height(&mut self.0)
    }

    pub fn apply_state_sync(
        &mut self,
        state_sync_update: StateSyncUpdate,
    ) -> Result<(), StoreError> {
        SqliteStore::apply_state_sync(&mut self.0, state_sync_update)
    }

    // TRANSACTIONS
    // --------------------------------------------------------------------------------------------

    pub fn get_transactions(
        &mut self,
        transaction_filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, StoreError> {
        SqliteStore::get_transactions(&mut self.0, &transaction_filter)
    }

    pub fn apply_transaction(
        &mut self,
        tx_update: TransactionStoreUpdate,
    ) -> Result<(), StoreError> {
        SqliteStore::apply_transaction(&mut self.0, &tx_update)
    }

    // NOTES
    // --------------------------------------------------------------------------------------------
    pub fn get_input_notes(
        &mut self,
        filter: NoteFilter,
    ) -> Result<Vec<InputNoteRecord>, StoreError> {
        SqliteStore::get_input_notes(&mut self.0, &filter)
    }

    pub fn get_output_notes(
        &mut self,
        note_filter: NoteFilter,
    ) -> Result<Vec<OutputNoteRecord>, StoreError> {
        SqliteStore::get_output_notes(&mut self.0, &note_filter)
    }

    pub fn upsert_input_notes(&mut self, notes: &[InputNoteRecord]) -> Result<(), StoreError> {
        SqliteStore::upsert_input_notes(&mut self.0, &notes)
    }

    // CHAIN DATA
    // --------------------------------------------------------------------------------------------

    pub fn insert_block_header(
        &mut self,
        block_header: &BlockHeader,
        partial_blockchain_peaks: MmrPeaks,
        has_client_notes: bool,
    ) -> Result<(), StoreError> {
        SqliteStore::insert_block_header(
            &mut self.0,
            &block_header,
            &partial_blockchain_peaks,
            has_client_notes,
        )
    }

    pub fn prune_irrelevant_blocks(&mut self) -> Result<(), StoreError> {
        SqliteStore::prune_irrelevant_blocks(&mut self.0)
    }

    pub fn get_block_headers(
        &mut self,
        block_numbers: &BTreeSet<BlockNumber>,
    ) -> Result<Vec<(BlockHeader, bool)>, StoreError> {
        SqliteStore::get_block_headers(&mut self.0, &block_numbers)
    }

    pub fn get_tracked_block_headers(&mut self) -> Result<Vec<BlockHeader>, StoreError> {
        SqliteStore::get_tracked_block_headers(&mut self.0)
    }

    pub fn get_partial_blockchain_nodes(
        &mut self,
        filter: PartialBlockchainFilter,
    ) -> Result<BTreeMap<InOrderIndex, Digest>, StoreError> {
        SqliteStore::get_partial_blockchain_nodes(&mut self.0, &filter)
    }

    pub fn insert_partial_blockchain_nodes(
        &mut self,
        nodes: &[(InOrderIndex, Digest)],
    ) -> Result<(), StoreError> {
        SqliteStore::insert_partial_blockchain_nodes(&mut self.0, &nodes)
    }

    pub fn get_partial_blockchain_peaks_by_block_num(
        &mut self,
        block_num: BlockNumber,
    ) -> Result<MmrPeaks, StoreError> {
        SqliteStore::get_partial_blockchain_peaks_by_block_num(&mut self.0, block_num)
    }

    // ACCOUNTS
    // --------------------------------------------------------------------------------------------

    pub fn insert_account(
        &mut self,
        account: &Account,
        account_seed: Option<Word>,
    ) -> Result<(), StoreError> {
        SqliteStore::insert_account(&mut self.0, &account, account_seed)
    }

    pub fn update_account(&mut self, account: &Account) -> Result<(), StoreError> {
        SqliteStore::update_account(&mut self.0, account)
    }

    pub fn get_account_ids(&mut self) -> Result<Vec<AccountId>, StoreError> {
        SqliteStore::get_account_ids(&mut self.0)
    }

    pub fn get_account_headers(
        &mut self,
    ) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
        SqliteStore::get_account_headers(&mut self.0)
    }

    pub fn get_account_header(
        &mut self,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        SqliteStore::get_account_header(&mut self.0, account_id)
    }

    pub fn get_account_header_by_commitment(
        &mut self,
        account_commitment: Digest,
    ) -> Result<Option<AccountHeader>, StoreError> {
        SqliteStore::get_account_header_by_commitment(&mut self.0, account_commitment)
    }

    pub fn get_account(
        &mut self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        SqliteStore::get_account(&mut self.0, account_id)
    }

    pub fn upsert_foreign_account_code(
        &mut self,
        account_id: AccountId,
        code: AccountCode,
    ) -> Result<(), StoreError> {
        SqliteStore::upsert_foreign_account_code(&mut self.0, account_id, &code)
    }

    pub fn get_foreign_account_code(
        &mut self,
        account_ids: Vec<AccountId>,
    ) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
        SqliteStore::get_foreign_account_code(&mut self.0, account_ids)
    }

    pub fn get_unspent_input_note_nullifiers(&mut self) -> Result<Vec<Nullifier>, StoreError> {
        SqliteStore::get_unspent_input_note_nullifiers(&mut self.0)
    }
}
