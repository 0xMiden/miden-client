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
use miden_client::note::{BlockNumber, NoteScript, Nullifier};
use miden_client::store::{
    AccountRecord,
    AccountStatus,
    AccountStorageFilter,
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
        self.get_note_tags().await
    }

    async fn add_note_tag(&self, tag: NoteTagRecord) -> Result<bool, StoreError> {
        self.add_note_tag(tag).await
    }

    async fn remove_note_tag(&self, tag: NoteTagRecord) -> Result<usize, StoreError> {
        self.remove_note_tag(tag).await
    }

    async fn get_sync_height(&self) -> Result<BlockNumber, StoreError> {
        self.get_sync_height().await
    }

    async fn apply_state_sync(&self, state_sync_update: StateSyncUpdate) -> Result<(), StoreError> {
        self.apply_state_sync(state_sync_update).await
    }

    // TRANSACTIONS
    // --------------------------------------------------------------------------------------------

    async fn get_transactions(
        &self,
        transaction_filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, StoreError> {
        self.get_transactions(transaction_filter).await
    }

    async fn apply_transaction(&self, tx_update: TransactionStoreUpdate) -> Result<(), StoreError> {
        self.apply_transaction(tx_update).await
    }

    // NOTES
    // --------------------------------------------------------------------------------------------
    async fn get_input_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<InputNoteRecord>, StoreError> {
        self.get_input_notes(filter).await
    }

    async fn get_output_notes(
        &self,
        note_filter: NoteFilter,
    ) -> Result<Vec<OutputNoteRecord>, StoreError> {
        self.get_output_notes(note_filter).await
    }

    async fn upsert_input_notes(&self, notes: &[InputNoteRecord]) -> Result<(), StoreError> {
        self.upsert_input_notes(notes).await
    }

    async fn get_note_script(&self, script_root: Word) -> Result<NoteScript, StoreError> {
        self.get_note_script(script_root).await
    }

    async fn upsert_note_scripts(&self, note_scripts: &[NoteScript]) -> Result<(), StoreError> {
        self.upsert_note_scripts(note_scripts).await
    }

    // CHAIN DATA
    // --------------------------------------------------------------------------------------------

    async fn insert_block_header(
        &self,
        block_header: &BlockHeader,
        partial_blockchain_peaks: MmrPeaks,
        has_client_notes: bool,
    ) -> Result<(), StoreError> {
        self.insert_block_header(block_header, partial_blockchain_peaks, has_client_notes)
            .await
    }

    async fn get_block_headers(
        &self,
        block_numbers: &BTreeSet<BlockNumber>,
    ) -> Result<Vec<(BlockHeader, BlockRelevance)>, StoreError> {
        self.get_block_headers(block_numbers).await
    }

    async fn get_tracked_block_headers(&self) -> Result<Vec<BlockHeader>, StoreError> {
        self.get_tracked_block_headers().await
    }

    async fn get_partial_blockchain_nodes(
        &self,
        filter: PartialBlockchainFilter,
    ) -> Result<BTreeMap<InOrderIndex, Word>, StoreError> {
        self.get_partial_blockchain_nodes(filter).await
    }

    async fn insert_partial_blockchain_nodes(
        &self,
        nodes: &[(InOrderIndex, Word)],
    ) -> Result<(), StoreError> {
        self.insert_partial_blockchain_nodes(nodes).await
    }

    async fn get_partial_blockchain_peaks_by_block_num(
        &self,
        block_num: BlockNumber,
    ) -> Result<MmrPeaks, StoreError> {
        self.get_partial_blockchain_peaks_by_block_num(block_num).await
    }

    async fn prune_irrelevant_blocks(&self) -> Result<(), StoreError> {
        self.prune_irrelevant_blocks().await
    }

    // ACCOUNTS
    // --------------------------------------------------------------------------------------------

    async fn insert_account(
        &self,
        account: &Account,
        initial_address: Address,
    ) -> Result<(), StoreError> {
        self.insert_account(account, initial_address).await
    }

    async fn update_account(&self, new_account_state: &Account) -> Result<(), StoreError> {
        self.update_account(new_account_state).await
    }

    async fn get_account_ids(&self) -> Result<Vec<AccountId>, StoreError> {
        self.get_account_ids().await
    }

    async fn get_account_headers(&self) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
        self.get_account_headers().await
    }

    async fn get_account_header(
        &self,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        self.get_account_header(account_id).await
    }

    async fn get_account_header_by_commitment(
        &self,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError> {
        self.get_account_header_by_commitment(account_commitment).await
    }

    async fn get_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        self.get_account(account_id).await
    }

    async fn get_minimal_partial_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        self.get_minimal_partial_account(account_id).await
    }

    async fn upsert_foreign_account_code(
        &self,
        account_id: AccountId,
        code: AccountCode,
    ) -> Result<(), StoreError> {
        self.upsert_foreign_account_code(account_id, code).await
    }

    async fn get_foreign_account_code(
        &self,
        account_ids: Vec<AccountId>,
    ) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
        self.get_foreign_account_code(account_ids).await
    }

    async fn get_unspent_input_note_nullifiers(&self) -> Result<Vec<Nullifier>, StoreError> {
        self.get_unspent_input_note_nullifiers().await
    }

    async fn get_account_vault(&self, account_id: AccountId) -> Result<AssetVault, StoreError> {
        self.get_account_vault(account_id).await
    }

    async fn get_account_storage(
        &self,
        account_id: AccountId,
        filter: AccountStorageFilter,
    ) -> Result<AccountStorage, StoreError> {
        self.get_account_storage(account_id, filter).await
    }

    async fn get_addresses_by_account_id(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<Address>, StoreError> {
        self.get_account_addresses(account_id).await
    }

    async fn insert_address(
        &self,
        address: Address,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        let derived_note_tag = address.to_note_tag();
        let note_tag_record = NoteTagRecord::with_account_source(derived_note_tag, account_id);
        let success = self.add_note_tag(note_tag_record).await?;
        if !success {
            return Err(StoreError::NoteTagAlreadyTracked(u64::from(derived_note_tag.as_u32())));
        }
        self.insert_address(address, &account_id).await
    }

    async fn remove_address(
        &self,
        address: Address,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        let derived_note_tag = address.to_note_tag();
        let note_tag_record = NoteTagRecord::with_account_source(derived_note_tag, account_id);
        self.remove_note_tag(note_tag_record).await?;
        self.remove_address(address).await
    }

    // SETTINGS
    // --------------------------------------------------------------------------------------------

    async fn set_setting(&self, key: String, value: Vec<u8>) -> Result<(), StoreError> {
        self.set_setting(key, value).await
    }

    async fn get_setting(&self, key: String) -> Result<Option<Vec<u8>>, StoreError> {
        self.get_setting(key).await
    }

    async fn remove_setting(&self, key: String) -> Result<(), StoreError> {
        self.remove_setting(key).await
    }

    async fn list_setting_keys(&self) -> Result<Vec<String>, StoreError> {
        self.list_setting_keys().await
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
