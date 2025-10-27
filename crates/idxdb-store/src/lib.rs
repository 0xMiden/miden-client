//! Provides an IndexedDB-backed implementation of the [Store] trait for web environments.
//!
//! This module enables persistence of client data (accounts, transactions, notes, block headers,
//! etc.) when running in a browser. It uses wasm-bindgen to interface with JavaScript and
//! `IndexedDB`, allowing the Miden client to store and retrieve data asynchronously.
//!
//! **Note:** This implementation is only available when targeting WebAssembly

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use base64::Engine;
use base64::engine::general_purpose;
use miden_client::Word;
use miden_client::account::{
    Account,
    AccountCode,
    AccountHeader,
    AccountId,
    AccountStorage,
    Address,
};
use miden_client::asset::AssetVault;
use miden_client::block::BlockHeader;
use miden_client::crypto::{InOrderIndex, MmrPeaks};
use miden_client::note::{BlockNumber, Nullifier};
use miden_client::store::{
    AccountRecord,
    AccountStatus,
    BlockRelevance,
    InputNoteRecord,
    NoteFilter,
    OutputNoteRecord,
    PartialBlockchainFilter,
    Store,
    StoreError,
    TransactionFilter,
};
use miden_client::sync::{NoteTagRecord, StateSyncUpdate};
use miden_client::transaction::{TransactionRecord, TransactionStoreUpdate};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, js_sys};

pub mod account;
pub mod auth;
pub mod chain_data;
pub mod export;
pub mod import;
pub mod note;
mod promise;
pub mod settings;
pub mod sync;
pub mod transaction;

#[wasm_bindgen(module = "/src/js/utils.js")]
extern "C" {
    #[wasm_bindgen(js_name = logWebStoreError)]
    fn log_web_store_error(error: JsValue, error_context: alloc::string::String);
}

// Initialize IndexedDB
#[wasm_bindgen(module = "/src/js/schema.js")]
extern "C" {
    #[wasm_bindgen(js_name = openDatabase)]
    fn setup_indexed_db() -> js_sys::Promise;
}

pub struct WebStore {}

impl WebStore {
    pub async fn new() -> Result<WebStore, JsValue> {
        JsFuture::from(setup_indexed_db()).await?;
        Ok(WebStore {})
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl Store for WebStore {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn get_current_timestamp(&self) -> Option<u64> {
        Some(current_timestamp_u64())
    }

    // SYNC
    // --------------------------------------------------------------------------------------------
    async fn get_note_tags(&self) -> Result<Vec<NoteTagRecord>, StoreError> {
        WebStore::get_note_tags(self).await
    }

    async fn add_note_tag(&self, tag: NoteTagRecord) -> Result<bool, StoreError> {
        WebStore::add_note_tag(self, tag).await
    }

    async fn remove_note_tag(&self, tag: NoteTagRecord) -> Result<usize, StoreError> {
        WebStore::remove_note_tag(self, tag).await
    }

    async fn get_sync_height(&self) -> Result<BlockNumber, StoreError> {
        WebStore::get_sync_height(self).await
    }

    async fn apply_state_sync(&self, state_sync_update: StateSyncUpdate) -> Result<(), StoreError> {
        WebStore::apply_state_sync(self, state_sync_update).await
    }

    // TRANSACTIONS
    // --------------------------------------------------------------------------------------------

    async fn get_transactions(
        &self,
        transaction_filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, StoreError> {
        WebStore::get_transactions(self, transaction_filter).await
    }

    async fn apply_transaction(&self, tx_update: TransactionStoreUpdate) -> Result<(), StoreError> {
        WebStore::apply_transaction(self, tx_update).await
    }

    // NOTES
    // --------------------------------------------------------------------------------------------
    async fn get_input_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<InputNoteRecord>, StoreError> {
        WebStore::get_input_notes(self, filter).await
    }

    async fn get_output_notes(
        &self,
        note_filter: NoteFilter,
    ) -> Result<Vec<OutputNoteRecord>, StoreError> {
        WebStore::get_output_notes(self, note_filter).await
    }

    async fn upsert_input_notes(&self, notes: &[InputNoteRecord]) -> Result<(), StoreError> {
        WebStore::upsert_input_notes(self, notes).await
    }

    // CHAIN DATA
    // --------------------------------------------------------------------------------------------

    async fn insert_block_header(
        &self,
        block_header: &BlockHeader,
        partial_blockchain_peaks: MmrPeaks,
        has_client_notes: bool,
    ) -> Result<(), StoreError> {
        WebStore::insert_block_header(self, block_header, partial_blockchain_peaks, has_client_notes)
            .await
    }

    async fn get_block_headers(
        &self,
        block_numbers: &BTreeSet<BlockNumber>,
    ) -> Result<Vec<(BlockHeader, BlockRelevance)>, StoreError> {
        WebStore::get_block_headers(self, block_numbers).await
    }

    async fn get_tracked_block_headers(&self) -> Result<Vec<BlockHeader>, StoreError> {
        WebStore::get_tracked_block_headers(self).await
    }

    async fn get_partial_blockchain_nodes(
        &self,
        filter: PartialBlockchainFilter,
    ) -> Result<BTreeMap<InOrderIndex, Word>, StoreError> {
        WebStore::get_partial_blockchain_nodes(self, filter).await
    }

    async fn insert_partial_blockchain_nodes(
        &self,
        nodes: &[(InOrderIndex, Word)],
    ) -> Result<(), StoreError> {
        WebStore::insert_partial_blockchain_nodes(self, nodes).await
    }

    async fn get_partial_blockchain_peaks_by_block_num(
        &self,
        block_num: BlockNumber,
    ) -> Result<MmrPeaks, StoreError> {
        WebStore::get_partial_blockchain_peaks_by_block_num(self, block_num).await
    }

    async fn prune_irrelevant_blocks(&self) -> Result<(), StoreError> {
        WebStore::prune_irrelevant_blocks(self).await
    }

    // ACCOUNTS
    // --------------------------------------------------------------------------------------------

    async fn insert_account(
        &self,
        account: &Account,
        initial_address: Address,
    ) -> Result<(), StoreError> {
        WebStore::insert_account(self, account, initial_address).await
    }

    async fn update_account(&self, new_account_state: &Account) -> Result<(), StoreError> {
        WebStore::update_account(self, new_account_state).await
    }

    async fn get_account_ids(&self) -> Result<Vec<AccountId>, StoreError> {
        WebStore::get_account_ids(self).await
    }

    async fn get_account_headers(&self) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
        WebStore::get_account_headers(self).await
    }

    async fn get_account_header(
        &self,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        WebStore::get_account_header(self, account_id).await
    }

    async fn get_account_header_by_commitment(
        &self,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError> {
        WebStore::get_account_header_by_commitment(self, account_commitment).await
    }

    async fn get_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        WebStore::get_account(self, account_id).await
    }

    async fn upsert_foreign_account_code(
        &self,
        account_id: AccountId,
        code: AccountCode,
    ) -> Result<(), StoreError> {
        WebStore::upsert_foreign_account_code(self, account_id, code).await
    }

    async fn get_foreign_account_code(
        &self,
        account_ids: Vec<AccountId>,
    ) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
        WebStore::get_foreign_account_code(self, account_ids).await
    }

    async fn get_unspent_input_note_nullifiers(&self) -> Result<Vec<Nullifier>, StoreError> {
        WebStore::get_unspent_input_note_nullifiers(self).await
    }

    async fn get_account_vault(&self, account_id: AccountId) -> Result<AssetVault, StoreError> {
        WebStore::get_account_vault(self, account_id).await
    }

    async fn get_account_storage(
        &self,
        account_id: AccountId,
    ) -> Result<AccountStorage, StoreError> {
        WebStore::get_account_storage(self, account_id).await
    }

    async fn get_addresses_by_account_id(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<Address>, StoreError> {
        WebStore::get_account_addresses(self, account_id).await
    }

    // SETTINGS
    // --------------------------------------------------------------------------------------------

    async fn set_setting(&self, key: String, value: Vec<u8>) -> Result<(), StoreError> {
        WebStore::set_setting(self, key, value).await
    }

    async fn get_setting(&self, key: String) -> Result<Option<Vec<u8>>, StoreError> {
        WebStore::get_setting(self, key).await
    }

    async fn remove_setting(&self, key: String) -> Result<(), StoreError> {
        WebStore::remove_setting(self, key).await
    }

    async fn list_setting_keys(&self) -> Result<Vec<String>, StoreError> {
        WebStore::list_setting_keys(self).await
    }
}

// UTILS
// ================================================================================================

/// Returns the current UTC timestamp as `u64` (non-leap seconds since Unix epoch).
pub(crate) fn current_timestamp_u64() -> u64 {
    let now = chrono::Utc::now();
    u64::try_from(now.timestamp()).expect("timestamp is always after epoch")
}

/// Helper function to decode a base64 string to a `Vec<u8>`.
pub(crate) fn base64_to_vec_u8_required<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let base64_str: String = Deserialize::deserialize(deserializer)?;
    general_purpose::STANDARD
        .decode(&base64_str)
        .map_err(|e| Error::custom(format!("Base64 decode error: {e}")))
}

/// Helper function to decode a base64 string to an `Option<Vec<u8>>`.
pub(crate) fn base64_to_vec_u8_optional<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let base64_str: Option<String> = Option::deserialize(deserializer)?;
    match base64_str {
        Some(str) => general_purpose::STANDARD
            .decode(&str)
            .map(Some)
            .map_err(|e| Error::custom(format!("Base64 decode error: {e}"))),
        None => Ok(None),
    }
}
