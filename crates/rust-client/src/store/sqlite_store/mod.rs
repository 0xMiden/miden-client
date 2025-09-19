//! This module provides an SQLite-backed implementation of the [Store] trait.
//!
//! [`SqliteStore`] enables the persistence of accounts, transactions, notes, block headers, and MMR
//! nodes using an `SQLite` database.
//! It is compiled only when the `sqlite` feature flag is enabled.

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use std::path::PathBuf;
use std::string::{String, ToString};
use std::sync::Arc;

use db_management::pool_manager::{Pool, SqlitePoolManager};
use db_management::utils::apply_migrations;
use miden_objects::Word;
use miden_objects::account::{
    Account,
    AccountCode,
    AccountHeader,
    AccountId,
    AccountIdPrefix,
    AccountStorage,
    StorageMapWitness,
};
use miden_objects::asset::{Asset, AssetVault};
use miden_objects::block::{BlockHeader, BlockNumber};
use miden_objects::crypto::merkle::{InOrderIndex, MerklePath, MerkleStore, MmrPeaks};
use miden_objects::note::{NoteTag, Nullifier};
use miden_tx::utils::sync::RwLock;
use rusqlite::Connection;
use rusqlite::types::Value;
use tonic::async_trait;

use super::{
    AccountRecord,
    AccountStatus,
    BlockRelevance,
    InputNoteRecord,
    NoteFilter,
    OutputNoteRecord,
    PartialBlockchainFilter,
    Store,
    TransactionFilter,
};
use crate::store::StoreError;
use crate::store::sqlite_store::db_management::utils::{get_value, remove_value, set_value};
use crate::store::sqlite_store::merkle_store::{insert_asset_nodes, insert_storage_map_nodes};
use crate::sync::{NoteTagRecord, StateSyncUpdate};
use crate::transaction::{TransactionRecord, TransactionStoreUpdate};

mod account;
mod chain_data;
mod db_management;
mod errors;
mod merkle_store;
mod note;
mod sync;
mod transaction;

// SQLITE STORE
// ================================================================================================

/// Represents a pool of connections with an `SQLite` database. The pool is used to interact
/// concurrently with the underlying database in a safe and efficient manner.
///
/// Current table definitions can be found at `store.sql` migration file.
pub struct SqliteStore {
    pub(crate) pool: Pool,
    merkle_store: Arc<RwLock<MerkleStore>>,
}

impl SqliteStore {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new instance of [Store] instantiated with the specified configuration options.
    pub async fn new(database_filepath: PathBuf) -> Result<Self, StoreError> {
        let sqlite_pool_manager = SqlitePoolManager::new(database_filepath.clone());
        let pool = Pool::builder(sqlite_pool_manager)
            .build()
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        let conn = pool.get().await.map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        conn.interact(apply_migrations)
            .await
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        let store = SqliteStore {
            pool,
            merkle_store: Arc::new(RwLock::new(MerkleStore::new())),
        };

        // Initialize merkle store
        for id in store.get_account_ids().await? {
            let vault = store.get_account_vault(id).await?;
            let storage = store.get_account_storage(id).await?;

            let mut merkle_store = store.merkle_store.write();
            insert_asset_nodes(&mut merkle_store, &vault);
            insert_storage_map_nodes(&mut merkle_store, &storage);
        }

        Ok(store)
    }

    /// Interacts with the database by executing the provided function on a connection from the
    /// pool.
    ///
    /// This function is a helper method which simplifies the process of making queries to the
    /// database. It acquires a connection from the pool and executes the provided function,
    /// returning the result.
    async fn interact_with_connection<F, R>(&self, f: F) -> Result<R, StoreError>
    where
        F: FnOnce(&mut Connection) -> Result<R, StoreError> + Send + 'static,
        R: Send + 'static,
    {
        self.pool
            .get()
            .await
            .map_err(|err| StoreError::DatabaseError(err.to_string()))?
            .interact(f)
            .await
            .map_err(|err| StoreError::DatabaseError(err.to_string()))?
    }
}

// SQLite implementation of the Store trait
//
// To simplify, all implementations rely on inner SqliteStore functions that map 1:1 by name
// This way, the actual implementations are grouped by entity types in their own sub-modules
#[async_trait]
impl Store for SqliteStore {
    fn get_current_timestamp(&self) -> Option<u64> {
        let now = chrono::Utc::now();
        Some(u64::try_from(now.timestamp()).expect("timestamp is always after epoch"))
    }

    async fn get_note_tags(&self) -> Result<Vec<NoteTagRecord>, StoreError> {
        self.interact_with_connection(SqliteStore::get_note_tags).await
    }

    async fn get_unique_note_tags(&self) -> Result<BTreeSet<NoteTag>, StoreError> {
        self.interact_with_connection(SqliteStore::get_unique_note_tags).await
    }

    async fn add_note_tag(&self, tag: NoteTagRecord) -> Result<bool, StoreError> {
        self.interact_with_connection(move |conn| SqliteStore::add_note_tag(conn, tag))
            .await
    }

    async fn remove_note_tag(&self, tag: NoteTagRecord) -> Result<usize, StoreError> {
        self.interact_with_connection(move |conn| SqliteStore::remove_note_tag(conn, tag))
            .await
    }

    async fn get_sync_height(&self) -> Result<BlockNumber, StoreError> {
        self.interact_with_connection(SqliteStore::get_sync_height).await
    }

    async fn apply_state_sync(&self, state_sync_update: StateSyncUpdate) -> Result<(), StoreError> {
        let merkle_store = self.merkle_store.clone();
        self.interact_with_connection(move |conn| {
            SqliteStore::apply_state_sync(conn, &merkle_store, state_sync_update)
        })
        .await
    }

    async fn get_transactions(
        &self,
        transaction_filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, StoreError> {
        self.interact_with_connection(move |conn| {
            SqliteStore::get_transactions(conn, &transaction_filter)
        })
        .await
    }

    async fn apply_transaction(&self, tx_update: TransactionStoreUpdate) -> Result<(), StoreError> {
        let merkle_store = self.merkle_store.clone();
        self.interact_with_connection(move |conn| {
            SqliteStore::apply_transaction(conn, &merkle_store, &tx_update)
        })
        .await
    }

    async fn get_input_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<InputNoteRecord>, StoreError> {
        self.interact_with_connection(move |conn| SqliteStore::get_input_notes(conn, &filter))
            .await
    }

    async fn get_output_notes(
        &self,
        note_filter: NoteFilter,
    ) -> Result<Vec<OutputNoteRecord>, StoreError> {
        self.interact_with_connection(move |conn| SqliteStore::get_output_notes(conn, &note_filter))
            .await
    }

    async fn upsert_input_notes(&self, notes: &[InputNoteRecord]) -> Result<(), StoreError> {
        let notes = notes.to_vec();
        self.interact_with_connection(move |conn| SqliteStore::upsert_input_notes(conn, &notes))
            .await
    }

    async fn insert_block_header(
        &self,
        block_header: &BlockHeader,
        partial_blockchain_peaks: MmrPeaks,
        has_client_notes: bool,
    ) -> Result<(), StoreError> {
        let block_header = block_header.clone();
        self.interact_with_connection(move |conn| {
            SqliteStore::insert_block_header(
                conn,
                &block_header,
                &partial_blockchain_peaks,
                has_client_notes,
            )
        })
        .await
    }

    async fn prune_irrelevant_blocks(&self) -> Result<(), StoreError> {
        self.interact_with_connection(SqliteStore::prune_irrelevant_blocks).await
    }

    async fn get_block_headers(
        &self,
        block_numbers: &BTreeSet<BlockNumber>,
    ) -> Result<Vec<(BlockHeader, BlockRelevance)>, StoreError> {
        let block_numbers = block_numbers.clone();
        Ok(self
            .interact_with_connection(move |conn| {
                SqliteStore::get_block_headers(conn, &block_numbers)
            })
            .await?)
    }

    async fn get_tracked_block_headers(&self) -> Result<Vec<BlockHeader>, StoreError> {
        self.interact_with_connection(SqliteStore::get_tracked_block_headers).await
    }

    async fn get_partial_blockchain_nodes(
        &self,
        filter: PartialBlockchainFilter,
    ) -> Result<BTreeMap<InOrderIndex, Word>, StoreError> {
        self.interact_with_connection(move |conn| {
            SqliteStore::get_partial_blockchain_nodes(conn, &filter)
        })
        .await
    }

    async fn insert_partial_blockchain_nodes(
        &self,
        nodes: &[(InOrderIndex, Word)],
    ) -> Result<(), StoreError> {
        let nodes = nodes.to_vec();
        self.interact_with_connection(move |conn| {
            SqliteStore::insert_partial_blockchain_nodes(conn, &nodes)
        })
        .await
    }

    async fn get_partial_blockchain_peaks_by_block_num(
        &self,
        block_num: BlockNumber,
    ) -> Result<MmrPeaks, StoreError> {
        self.interact_with_connection(move |conn| {
            SqliteStore::get_partial_blockchain_peaks_by_block_num(conn, block_num)
        })
        .await
    }

    async fn insert_account(
        &self,
        account: &Account,
        account_seed: Option<Word>,
    ) -> Result<(), StoreError> {
        let cloned_account = account.clone();
        let merkle_store = self.merkle_store.clone();

        self.interact_with_connection(move |conn| {
            SqliteStore::insert_account(conn, &merkle_store, &cloned_account, account_seed)
        })
        .await
    }

    async fn update_account(&self, account: &Account) -> Result<(), StoreError> {
        let cloned_account = account.clone();
        let merkle_store = self.merkle_store.clone();

        self.interact_with_connection(move |conn| {
            SqliteStore::update_account(conn, &merkle_store, &cloned_account)
        })
        .await
    }

    async fn get_account_ids(&self) -> Result<Vec<AccountId>, StoreError> {
        self.interact_with_connection(SqliteStore::get_account_ids).await
    }

    async fn get_account_headers(&self) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
        self.interact_with_connection(SqliteStore::get_account_headers).await
    }

    async fn get_account_header(
        &self,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        self.interact_with_connection(move |conn| SqliteStore::get_account_header(conn, account_id))
            .await
    }

    async fn get_account_header_by_commitment(
        &self,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError> {
        self.interact_with_connection(move |conn| {
            SqliteStore::get_account_header_by_commitment(conn, account_commitment)
        })
        .await
    }

    async fn get_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        self.interact_with_connection(move |conn| SqliteStore::get_account(conn, account_id))
            .await
    }

    async fn upsert_foreign_account_code(
        &self,
        account_id: AccountId,
        code: AccountCode,
    ) -> Result<(), StoreError> {
        self.interact_with_connection(move |conn| {
            SqliteStore::upsert_foreign_account_code(conn, account_id, &code)
        })
        .await
    }

    async fn get_foreign_account_code(
        &self,
        account_ids: Vec<AccountId>,
    ) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
        self.interact_with_connection(move |conn| {
            SqliteStore::get_foreign_account_code(conn, account_ids)
        })
        .await
    }

    async fn set_value(&self, key: String, value: Vec<u8>) -> Result<(), StoreError> {
        self.interact_with_connection(move |conn| set_value(conn, &key, &value)).await
    }

    async fn get_value(&self, key: String) -> Result<Option<Vec<u8>>, StoreError> {
        self.interact_with_connection(move |conn| get_value(conn, &key)).await
    }

    async fn remove_value(&self, key: String) -> Result<(), StoreError> {
        self.interact_with_connection(move |conn| remove_value(conn, &key)).await
    }

    async fn get_unspent_input_note_nullifiers(&self) -> Result<Vec<Nullifier>, StoreError> {
        self.interact_with_connection(SqliteStore::get_unspent_input_note_nullifiers)
            .await
    }

    async fn get_account_vault(&self, account_id: AccountId) -> Result<AssetVault, StoreError> {
        self.interact_with_connection(move |conn| SqliteStore::get_account_vault(conn, account_id))
            .await
    }

    async fn get_account_asset(
        &self,
        account_id: AccountId,
        faucet_id_prefix: AccountIdPrefix,
    ) -> Result<Option<(Asset, MerklePath)>, StoreError> {
        let merkle_store = self.merkle_store.clone();
        self.interact_with_connection(move |conn| {
            SqliteStore::get_account_asset(conn, &merkle_store, account_id, faucet_id_prefix)
        })
        .await
    }

    async fn get_account_storage(
        &self,
        account_id: AccountId,
    ) -> Result<AccountStorage, StoreError> {
        self.interact_with_connection(move |conn| {
            SqliteStore::get_account_storage(conn, account_id)
        })
        .await
    }

    async fn get_account_map_item(
        &self,
        account_id: AccountId,
        index: u8,
        key: Word,
    ) -> Result<(Word, StorageMapWitness), StoreError> {
        let merkle_store = self.merkle_store.clone();

        self.interact_with_connection(move |conn| {
            SqliteStore::get_account_map_item(conn, &merkle_store, account_id, index, key)
        })
        .await
    }
}

// UTILS
// ================================================================================================

/// Gets a `u64` value from the database.
///
/// `Sqlite` uses `i64` as its internal representation format, and so when retrieving
/// we need to make sure we cast as `u64` to get the original value
pub fn column_value_as_u64<I: rusqlite::RowIndex>(
    row: &rusqlite::Row<'_>,
    index: I,
) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    #[allow(
        clippy::cast_sign_loss,
        reason = "We store u64 as i64 as sqlite only allows the latter."
    )]
    Ok(value as u64)
}

/// Converts a `u64` into a [Value].
///
/// `Sqlite` uses `i64` as its internal representation format. Note that the `as` operator performs
/// a lossless conversion from `u64` to `i64`.
pub fn u64_to_value(v: u64) -> Value {
    #[allow(
        clippy::cast_possible_wrap,
        reason = "We store u64 as i64 as sqlite only allows the latter."
    )]
    Value::Integer(v as i64)
}

// TESTS
// ================================================================================================

#[cfg(test)]
pub mod tests {
    use std::boxed::Box;

    use super::SqliteStore;
    use crate::store::Store;
    use crate::testing::common::create_test_store_path;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn is_send_sync() {
        assert_send_sync::<SqliteStore>();
        assert_send_sync::<Box<dyn Store>>();
    }

    // Function that returns a `Send` future from a dynamic trait that must be `Sync`.
    async fn dyn_trait_send_fut(store: Box<dyn Store>) {
        // This wouldn't compile if `get_tracked_block_headers` doesn't return a `Send` future.
        let res = store.get_tracked_block_headers().await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn future_is_send() {
        let client = SqliteStore::new(create_test_store_path()).await.unwrap();
        let client: Box<SqliteStore> = client.into();
        tokio::task::spawn(async move { dyn_trait_send_fut(client).await });
    }

    pub(crate) async fn create_test_store() -> SqliteStore {
        SqliteStore::new(create_test_store_path()).await.unwrap()
    }
}
