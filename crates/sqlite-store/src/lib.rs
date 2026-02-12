//! SQLite-backed Store implementation for miden-client.
//! This crate provides `SqliteStore` and its full implementation.
//!
//! [`SqliteStore`] enables the persistence of accounts, transactions, notes, block headers, and MMR
//! nodes using an `SQLite` database.

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

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
use miden_client::note::{BlockNumber, NoteScript, NoteTag, Nullifier};
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
// Native-only imports
#[cfg(not(target_arch = "wasm32"))]
use {
    crate::smt_forest::AccountSmtForest,
    alloc::string::ToString,
    db_management::pool_manager::{Pool, SqlitePoolManager},
    db_management::utils::apply_migrations,
    miden_client::account::StorageSlotName,
    miden_client::asset::{Asset, AssetWitness},
    miden_protocol::account::StorageMapWitness,
    miden_protocol::asset::AssetVaultKey,
    rusqlite::Connection,
    rusqlite::types::Value,
    sql_error::SqlResultExt,
    std::path::PathBuf,
    std::sync::{Arc, RwLock},
};

// Shared modules (both native and WASM)
#[macro_use]
mod macros;
mod account;
mod chain_data;
mod note;
mod settings;
pub(crate) mod sql_types;
mod sync;
mod transaction;

// Native-only modules
#[cfg(not(target_arch = "wasm32"))]
mod builder;
#[cfg(not(target_arch = "wasm32"))]
mod db_management;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
mod smt_forest;
#[cfg(not(target_arch = "wasm32"))]
mod sql_error;

// WASM-only modules
#[cfg(target_arch = "wasm32")]
mod wasm;

// Public re-exports
#[cfg(not(target_arch = "wasm32"))]
pub use builder::ClientBuilderSqliteExt;
#[cfg(target_arch = "wasm32")]
pub use wasm::SqliteStore;

// SQLITE STORE
// ================================================================================================

/// Represents a pool of connections with an `SQLite` database. The pool is used to interact
/// concurrently with the underlying database in a safe and efficient manner.
///
/// Current table definitions can be found at `store.sql` migration file.
#[cfg(not(target_arch = "wasm32"))]
pub struct SqliteStore {
    pub(crate) pool: Pool,
    smt_forest: Arc<RwLock<AccountSmtForest>>,
}

// NATIVE IMPLEMENTATION
// ================================================================================================

#[cfg(not(target_arch = "wasm32"))]
impl SqliteStore {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new instance of [Store] instantiated with the specified configuration options.
    pub async fn new(database_filepath: PathBuf) -> Result<Self, StoreError> {
        let sqlite_pool_manager = SqlitePoolManager::new(database_filepath);
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
            smt_forest: Arc::new(RwLock::new(AccountSmtForest::new())),
        };

        // Initialize SMT forest
        for id in store.get_account_ids().await? {
            let vault = store.get_account_vault(id).await?;
            let storage = store.get_account_storage(id, AccountStorageFilter::All).await?;

            let mut smt_forest = store.smt_forest.write().expect("smt write lock not poisoned");
            smt_forest.insert_account_state(&vault, &storage)?;
        }

        Ok(store)
    }

    /// Interacts with the database by executing the provided function on a connection from the
    /// pool.
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

    /// Execute a closure with a [`SqlConnection`] for read-only queries.
    async fn run<F, T>(&self, f: F) -> Result<T, StoreError>
    where
        F: FnOnce(&dyn sql_types::SqlConnection) -> Result<T, StoreError> + Send + 'static,
        T: Send + 'static,
    {
        self.interact_with_connection(|conn| {
            let sql_conn = native::RusqliteConnection(conn);
            f(&sql_conn)
        })
        .await
    }

    /// Execute a closure within a SQL transaction via [`SqlConnection`].
    async fn run_in_tx<F, T>(&self, f: F) -> Result<T, StoreError>
    where
        F: FnOnce(&dyn sql_types::SqlConnection) -> Result<T, StoreError> + Send + 'static,
        T: Send + 'static,
    {
        self.interact_with_connection(|conn| {
            let tx = conn.transaction().into_store_error()?;
            let sql_conn = native::RusqliteTransaction(&tx);
            let result = f(&sql_conn)?;
            tx.commit().into_store_error()?;
            Ok(result)
        })
        .await
    }
}

// STORE TRAIT IMPLEMENTATION
// ================================================================================================

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl Store for SqliteStore {
    fn get_current_timestamp(&self) -> Option<u64> {
        Some(current_timestamp_u64())
    }

    async fn get_note_tags(&self) -> Result<Vec<NoteTagRecord>, StoreError> {
        self.run(|conn| sync::get_note_tags_shared(conn)).await
    }

    async fn get_unique_note_tags(&self) -> Result<BTreeSet<NoteTag>, StoreError> {
        self.run(|conn| sync::get_unique_note_tags_shared(conn)).await
    }

    async fn add_note_tag(&self, tag: NoteTagRecord) -> Result<bool, StoreError> {
        self.run_in_tx(move |conn| sync::add_note_tag_shared(conn, &tag)).await
    }

    async fn remove_note_tag(&self, tag: NoteTagRecord) -> Result<usize, StoreError> {
        self.run_in_tx(move |conn| sync::remove_note_tag_shared(conn, &tag)).await
    }

    async fn get_sync_height(&self) -> Result<BlockNumber, StoreError> {
        self.run(|conn| sync::get_sync_height_shared(conn)).await
    }

    // apply_state_sync: native uses SMT forest, WASM uses shared version
    #[cfg(not(target_arch = "wasm32"))]
    async fn apply_state_sync(&self, state_sync_update: StateSyncUpdate) -> Result<(), StoreError> {
        let smt_forest = self.smt_forest.clone();
        self.interact_with_connection(move |conn| {
            SqliteStore::apply_state_sync(conn, &smt_forest, state_sync_update)
        })
        .await
    }

    #[cfg(target_arch = "wasm32")]
    async fn apply_state_sync(&self, state_sync_update: StateSyncUpdate) -> Result<(), StoreError> {
        self.run_in_tx(move |conn| sync::apply_state_sync_shared(conn, state_sync_update))
            .await
    }

    async fn get_transactions(
        &self,
        transaction_filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, StoreError> {
        self.run(move |conn| transaction::get_transactions_shared(conn, &transaction_filter))
            .await
    }

    // apply_transaction: native uses SMT forest + delta, WASM uses shared version
    #[cfg(not(target_arch = "wasm32"))]
    async fn apply_transaction(&self, tx_update: TransactionStoreUpdate) -> Result<(), StoreError> {
        let smt_forest = self.smt_forest.clone();
        self.interact_with_connection(move |conn| {
            SqliteStore::apply_transaction(conn, &smt_forest, &tx_update)
        })
        .await
    }

    #[cfg(target_arch = "wasm32")]
    async fn apply_transaction(&self, tx_update: TransactionStoreUpdate) -> Result<(), StoreError> {
        self.run_in_tx(move |conn| wasm::apply_transaction_impl(conn, &tx_update)).await
    }

    async fn get_input_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<InputNoteRecord>, StoreError> {
        self.run(move |conn| note::get_input_notes_shared(conn, &filter)).await
    }

    async fn get_output_notes(
        &self,
        note_filter: NoteFilter,
    ) -> Result<Vec<OutputNoteRecord>, StoreError> {
        self.run(move |conn| note::get_output_notes_shared(conn, &note_filter)).await
    }

    async fn upsert_input_notes(&self, notes: &[InputNoteRecord]) -> Result<(), StoreError> {
        let notes = notes.to_vec();
        self.run_in_tx(move |conn| note::upsert_input_notes_shared(conn, &notes)).await
    }

    async fn get_note_script(&self, script_root: Word) -> Result<NoteScript, StoreError> {
        self.run(move |conn| note::get_note_script_shared(conn, script_root)).await
    }

    async fn upsert_note_scripts(&self, note_scripts: &[NoteScript]) -> Result<(), StoreError> {
        let note_scripts = note_scripts.to_vec();
        self.run_in_tx(move |conn| note::upsert_note_scripts_shared(conn, &note_scripts))
            .await
    }

    async fn insert_block_header(
        &self,
        block_header: &BlockHeader,
        partial_blockchain_peaks: MmrPeaks,
        has_client_notes: bool,
    ) -> Result<(), StoreError> {
        let block_header = block_header.clone();
        self.run_in_tx(move |conn| {
            chain_data::insert_block_header_shared(
                conn,
                &block_header,
                &partial_blockchain_peaks,
                has_client_notes,
            )
        })
        .await
    }

    async fn prune_irrelevant_blocks(&self) -> Result<(), StoreError> {
        self.run_in_tx(|conn| chain_data::prune_irrelevant_blocks_shared(conn)).await
    }

    async fn get_block_headers(
        &self,
        block_numbers: &BTreeSet<BlockNumber>,
    ) -> Result<Vec<(BlockHeader, BlockRelevance)>, StoreError> {
        let block_numbers = block_numbers.clone();
        self.run(move |conn| chain_data::get_block_headers_shared(conn, &block_numbers))
            .await
    }

    async fn get_tracked_block_headers(&self) -> Result<Vec<BlockHeader>, StoreError> {
        self.run(|conn| chain_data::get_tracked_block_headers_shared(conn)).await
    }

    async fn get_partial_blockchain_nodes(
        &self,
        filter: PartialBlockchainFilter,
    ) -> Result<BTreeMap<InOrderIndex, Word>, StoreError> {
        self.run(move |conn| chain_data::get_partial_blockchain_nodes_shared(conn, &filter))
            .await
    }

    async fn insert_partial_blockchain_nodes(
        &self,
        nodes: &[(InOrderIndex, Word)],
    ) -> Result<(), StoreError> {
        let nodes = nodes.to_vec();
        self.run_in_tx(move |conn| chain_data::insert_partial_blockchain_nodes_shared(conn, &nodes))
            .await
    }

    async fn get_partial_blockchain_peaks_by_block_num(
        &self,
        block_num: BlockNumber,
    ) -> Result<MmrPeaks, StoreError> {
        self.run(move |conn| {
            chain_data::get_partial_blockchain_peaks_by_block_num_shared(conn, block_num)
        })
        .await
    }

    // insert_account: native uses SMT forest, WASM uses shared version
    #[cfg(not(target_arch = "wasm32"))]
    async fn insert_account(
        &self,
        account: &Account,
        initial_address: Address,
    ) -> Result<(), StoreError> {
        let cloned_account = account.clone();
        let smt_forest = self.smt_forest.clone();

        self.interact_with_connection(move |conn| {
            SqliteStore::insert_account(conn, &smt_forest, &cloned_account, &initial_address)
        })
        .await
    }

    #[cfg(target_arch = "wasm32")]
    async fn insert_account(
        &self,
        account: &Account,
        initial_address: Address,
    ) -> Result<(), StoreError> {
        let cloned_account = account.clone();
        self.run_in_tx(move |conn| {
            account::shared::insert_account_shared(conn, &cloned_account, &initial_address)
        })
        .await
    }

    // update_account: native uses SMT forest, WASM uses shared version
    #[cfg(not(target_arch = "wasm32"))]
    async fn update_account(&self, account: &Account) -> Result<(), StoreError> {
        let cloned_account = account.clone();
        let smt_forest = self.smt_forest.clone();

        self.interact_with_connection(move |conn| {
            SqliteStore::update_account(conn, &smt_forest, &cloned_account)
        })
        .await
    }

    #[cfg(target_arch = "wasm32")]
    async fn update_account(&self, account: &Account) -> Result<(), StoreError> {
        let cloned_account = account.clone();
        self.run_in_tx(move |conn| account::shared::update_account_shared(conn, &cloned_account))
            .await
    }

    async fn get_account_ids(&self) -> Result<Vec<AccountId>, StoreError> {
        self.run(|conn| account::shared::get_account_ids_shared(conn)).await
    }

    async fn get_account_headers(&self) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
        self.run(|conn| account::shared::get_account_headers_shared(conn)).await
    }

    async fn get_account_header(
        &self,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        self.run(move |conn| account::shared::get_account_header_shared(conn, account_id))
            .await
    }

    async fn get_account_header_by_commitment(
        &self,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError> {
        self.run(move |conn| {
            account::shared::get_account_header_by_commitment_shared(conn, account_commitment)
        })
        .await
    }

    async fn get_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        self.run(move |conn| account::shared::get_account_shared(conn, account_id))
            .await
    }

    async fn get_account_code(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountCode>, StoreError> {
        self.run(move |conn| account::shared::get_account_code_by_id_shared(conn, account_id))
            .await
    }

    async fn upsert_foreign_account_code(
        &self,
        account_id: AccountId,
        code: AccountCode,
    ) -> Result<(), StoreError> {
        self.run_in_tx(move |conn| {
            account::shared::upsert_foreign_account_code_shared(conn, account_id, &code)
        })
        .await
    }

    async fn get_foreign_account_code(
        &self,
        account_ids: Vec<AccountId>,
    ) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
        self.run(move |conn| account::shared::get_foreign_account_code_shared(conn, &account_ids))
            .await
    }

    async fn set_setting(&self, key: String, value: Vec<u8>) -> Result<(), StoreError> {
        self.run(move |conn| settings::set_setting_shared(conn, &key, &value)).await
    }

    async fn get_setting(&self, key: String) -> Result<Option<Vec<u8>>, StoreError> {
        self.run(move |conn| settings::get_setting_shared(conn, &key)).await
    }

    async fn remove_setting(&self, key: String) -> Result<(), StoreError> {
        self.run(move |conn| settings::remove_setting_shared(conn, &key)).await
    }

    async fn list_setting_keys(&self) -> Result<Vec<String>, StoreError> {
        self.run(move |conn| settings::list_setting_keys_shared(conn)).await
    }

    async fn get_unspent_input_note_nullifiers(&self) -> Result<Vec<Nullifier>, StoreError> {
        self.run(|conn| note::get_unspent_input_note_nullifiers_shared(conn)).await
    }

    async fn get_account_vault(&self, account_id: AccountId) -> Result<AssetVault, StoreError> {
        self.run(move |conn| account::shared::get_account_vault_shared(conn, account_id))
            .await
    }

    // get_account_asset: native overrides with SMT-backed version; WASM uses trait default
    #[cfg(not(target_arch = "wasm32"))]
    async fn get_account_asset(
        &self,
        account_id: AccountId,
        vault_key: AssetVaultKey,
    ) -> Result<Option<(Asset, AssetWitness)>, StoreError> {
        let smt_forest = self.smt_forest.clone();
        self.interact_with_connection(move |conn| {
            SqliteStore::get_account_asset(conn, &smt_forest, account_id, vault_key)
        })
        .await
    }

    async fn get_account_storage(
        &self,
        account_id: AccountId,
        filter: AccountStorageFilter,
    ) -> Result<AccountStorage, StoreError> {
        self.run(move |conn| account::shared::get_account_storage_shared(conn, account_id, &filter))
            .await
    }

    // get_account_map_item: native overrides with SMT-backed version; WASM uses trait default
    #[cfg(not(target_arch = "wasm32"))]
    async fn get_account_map_item(
        &self,
        account_id: AccountId,
        slot_name: StorageSlotName,
        key: Word,
    ) -> Result<(Word, StorageMapWitness), StoreError> {
        let smt_forest = self.smt_forest.clone();

        self.interact_with_connection(move |conn| {
            SqliteStore::get_account_map_item(conn, &smt_forest, account_id, slot_name, key)
        })
        .await
    }

    async fn get_addresses_by_account_id(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<Address>, StoreError> {
        self.run(move |conn| account::shared::get_account_addresses_shared(conn, account_id))
            .await
    }

    async fn insert_address(
        &self,
        address: Address,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        self.run_in_tx(move |conn| {
            account::shared::insert_address_shared(conn, &address, account_id)
        })
        .await
    }

    async fn remove_address(
        &self,
        address: Address,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        self.run_in_tx(move |conn| {
            account::shared::remove_address_shared(conn, &address, account_id)
        })
        .await
    }

    // get_minimal_partial_account: native uses SMT, WASM uses shared version
    #[cfg(not(target_arch = "wasm32"))]
    async fn get_minimal_partial_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        let smt_forest = self.smt_forest.clone();

        self.interact_with_connection(move |conn| {
            SqliteStore::get_minimal_partial_account(conn, &smt_forest, account_id)
        })
        .await
    }

    #[cfg(target_arch = "wasm32")]
    async fn get_minimal_partial_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        self.run(move |conn| account::shared::get_minimal_partial_account_shared(conn, account_id))
            .await
    }
}

// UTILS
// ================================================================================================

/// Returns the current UTC timestamp as `u64` (non-leap seconds since Unix epoch).
pub(crate) fn current_timestamp_u64() -> u64 {
    let now = chrono::Utc::now();
    u64::try_from(now.timestamp()).expect("timestamp is always after epoch")
}

/// Gets a `u64` value from the database.
///
/// `Sqlite` uses `i64` as its internal representation format, and so when retrieving
/// we need to make sure we cast as `u64` to get the original value
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
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

    use miden_client::store::Store;
    use miden_client::testing::common::create_test_store_path;

    use super::SqliteStore;

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
