//! Account-related database operations.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::string::{String, ToString};
use std::sync::{Arc, RwLock};
use std::vec::Vec;

use miden_client::account::{
    Account,
    AccountCode,
    AccountDelta,
    AccountHeader,
    AccountId,
    AccountIdPrefix,
    AccountStorage,
    Address,
    PartialAccount,
    PartialStorage,
    PartialStorageMap,
    StorageMap,
    StorageSlotName,
    StorageSlotType,
};
use miden_client::asset::{Asset, AssetVault, AssetWitness, FungibleAsset};
use miden_client::store::{
    AccountRecord,
    AccountRecordData,
    AccountStatus,
    AccountStorageFilter,
    StoreError,
};
use miden_client::sync::NoteTagRecord;
use miden_client::utils::Serializable;
use miden_client::{AccountError, Word};
use miden_protocol::account::{AccountStorageHeader, StorageMapWitness, StorageSlotHeader};
use miden_protocol::asset::{AssetVaultKey, PartialVault};
use miden_protocol::crypto::merkle::MerkleError;
use rusqlite::types::Value;
use rusqlite::{CachedStatement, Connection, Transaction, named_params, params};

use crate::account::helpers::{
    build_storage_slots_from_values,
    query_account_addresses,
    query_account_code,
    query_account_headers,
    query_storage_maps,
    query_vault_assets,
};
use crate::account::query::current::{query_latest_storage_map_roots, query_latest_storage_values};
use crate::account::query::domain::StorageSlotValueNonceRow;
use crate::account::query::history::query_storage_map_roots_at_or_before_nonce;
use crate::smt_forest::AccountSmtForest;
use crate::sql_error::SqlResultExt;
use crate::sync::{add_note_tag_tx, remove_note_tag_tx};
use crate::{SqliteStore, column_value_as_u64, insert_sql, subst, u64_to_value};

impl SqliteStore {
    // READER METHODS
    // --------------------------------------------------------------------------------------------

    pub(crate) fn get_account_ids(conn: &mut Connection) -> Result<Vec<AccountId>, StoreError> {
        const QUERY: &str = "SELECT id FROM tracked_accounts";

        conn.prepare_cached(QUERY)
            .into_store_error()?
            .query_map([], |row| row.get(0))
            .into_store_error()?
            .map(|result| {
                let id: String = result.into_store_error()?;
                Self::parse_account_id(&id)
            })
            .collect::<Result<Vec<AccountId>, StoreError>>()
    }

    pub(crate) fn get_account_headers(
        conn: &mut Connection,
    ) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
        query_account_headers(conn, "accounts_latest", "1 = 1 ORDER BY id", params![])
    }

    pub(crate) fn get_account_header(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        Ok(
            query_account_headers(conn, "accounts_latest", "id = ?", params![account_id.to_hex()])?
                .pop(),
        )
    }

    pub(crate) fn get_account_header_by_commitment(
        conn: &mut Connection,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError> {
        let account_commitment_str: String = account_commitment.to_string();
        Ok(query_account_headers(
            conn,
            "accounts_history",
            "account_commitment = ?",
            params![account_commitment_str],
        )?
        .pop()
        .map(|(header, _)| header))
    }

    /// Retrieves a complete account record with full vault and storage data.
    pub(crate) fn get_account(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        let Some((header, status)) = Self::get_account_header(conn, account_id)? else {
            return Ok(None);
        };

        let assets = query_vault_assets(conn, "root = ?", params![header.vault_root().to_hex()])?;
        let vault = AssetVault::new(&assets)?;

        let slots =
            build_storage_slots_from_values(conn, query_latest_storage_values(conn, header.id())?)?
                .into_values()
                .collect();

        let storage = AccountStorage::new(slots)?;

        let Some(account_code) = query_account_code(conn, header.code_commitment())? else {
            return Ok(None);
        };

        let account = Account::new_unchecked(
            header.id(),
            vault,
            storage,
            account_code,
            header.nonce(),
            status.seed().copied(),
        );

        let account_data = AccountRecordData::Full(account);
        Ok(Some(AccountRecord::new(account_data, status)))
    }

    /// Retrieves a minimal partial account record with storage and vault witnesses.
    pub(crate) fn get_minimal_partial_account(
        conn: &mut Connection,
        smt_forest: &Arc<RwLock<AccountSmtForest>>,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, StoreError> {
        let Some((header, status)) = Self::get_account_header(conn, account_id)? else {
            return Ok(None);
        };

        // Partial vault retrieval
        let partial_vault = PartialVault::new(header.vault_root());

        // Partial storage retrieval
        let mut storage_header = Vec::new();
        let mut maps = vec![];

        let storage_values = query_latest_storage_values(conn, header.id())?;

        // Collect all map roots for a single batched query
        let map_roots: Vec<Value> = storage_values
            .iter()
            .filter(|(_, (slot_type, _))| *slot_type == StorageSlotType::Map)
            .map(|(_, (_, value))| Value::from(value.to_hex()))
            .collect();

        // Fetch all storage maps in a single query
        let mut all_storage_maps = if map_roots.is_empty() {
            BTreeMap::new()
        } else {
            query_storage_maps(conn, "root IN rarray(?)", [Rc::new(map_roots)])?
        };

        for (slot_name, (slot_type, value)) in storage_values {
            storage_header.push(StorageSlotHeader::new(slot_name.clone(), slot_type, value));
            if slot_type == StorageSlotType::Map {
                let mut partial_storage_map = PartialStorageMap::new(value);

                if let Some(map) = all_storage_maps.remove(&value) {
                    let smt_forest = smt_forest.read().expect("smt_forest read lock not poisoned");
                    for (k, _v) in map.entries() {
                        let witness = smt_forest.get_storage_map_item_witness(value, *k)?;
                        partial_storage_map.add(witness).map_err(StoreError::MerkleStoreError)?;
                    }
                }

                maps.push(partial_storage_map);
            }
        }
        storage_header.sort_by_key(StorageSlotHeader::id);
        let storage_header =
            AccountStorageHeader::new(storage_header).map_err(StoreError::AccountError)?;
        let partial_storage =
            PartialStorage::new(storage_header, maps).map_err(StoreError::AccountError)?;

        let Some(account_code) = query_account_code(conn, header.code_commitment())? else {
            return Ok(None);
        };

        let partial_account = PartialAccount::new(
            header.id(),
            header.nonce(),
            account_code,
            partial_storage,
            partial_vault,
            status.seed().copied(),
        )?;
        let account_record_data = AccountRecordData::Partial(partial_account);
        Ok(Some(AccountRecord::new(account_record_data, status)))
    }

    pub fn get_foreign_account_code(
        conn: &mut Connection,
        account_ids: Vec<AccountId>,
    ) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
        let params: Vec<Value> =
            account_ids.into_iter().map(|id| Value::from(id.to_hex())).collect();
        const QUERY: &str = "
            SELECT account_id, code
            FROM foreign_account_code JOIN account_code ON foreign_account_code.code_commitment = account_code.commitment
            WHERE account_id IN rarray(?)";

        conn.prepare_cached(QUERY)
            .into_store_error()?
            .query_map([Rc::new(params)], |row| Ok((row.get(0)?, row.get(1)?)))
            .into_store_error()?
            .map(|result| {
                let (id, code): (String, Vec<u8>) = result.into_store_error()?;
                Ok((
                    Self::parse_final_header_account_id(&id)?,
                    AccountCode::from_bytes(&code).map_err(StoreError::AccountError)?,
                ))
            })
            .collect::<Result<BTreeMap<AccountId, AccountCode>, _>>()
    }

    /// Retrieves the full asset vault for a specific account.
    pub fn get_account_vault(
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<AssetVault, StoreError> {
        let assets = query_vault_assets(
            conn,
            "root = (SELECT vault_root FROM accounts_latest WHERE id = ?)",
            params![account_id.to_hex()],
        )?;

        Ok(AssetVault::new(&assets)?)
    }

    /// Retrieves the full storage for a specific account.
    pub fn get_account_storage(
        conn: &Connection,
        account_id: AccountId,
        filter: &AccountStorageFilter,
    ) -> Result<AccountStorage, StoreError> {
        let mut storage_values = query_latest_storage_values(conn, account_id)?;
        match filter {
            AccountStorageFilter::All => {},
            AccountStorageFilter::Root(root) => {
                storage_values.retain(|_, (_, value)| value == root);
            },
            AccountStorageFilter::SlotName(slot_name) => {
                storage_values.retain(|name, _| name == slot_name);
            },
        }

        let slots = build_storage_slots_from_values(conn, storage_values)?.into_values().collect();

        Ok(AccountStorage::new(slots)?)
    }

    /// Fetches a specific asset from the account's vault without the need of loading the entire
    /// vault. The witness is retrieved from the [`AccountSmtForest`].
    pub(crate) fn get_account_asset(
        conn: &mut Connection,
        smt_forest: &Arc<RwLock<AccountSmtForest>>,
        account_id: AccountId,
        vault_key: AssetVaultKey,
    ) -> Result<Option<(Asset, AssetWitness)>, StoreError> {
        let header = Self::get_account_header(conn, account_id)?
            .ok_or(StoreError::AccountDataNotFound(account_id))?
            .0;

        let smt_forest = smt_forest.read().expect("smt_forest read lock not poisoned");
        match smt_forest.get_asset_and_witness(header.vault_root(), vault_key) {
            Ok((asset, witness)) => Ok(Some((asset, witness))),
            Err(StoreError::MerkleStoreError(MerkleError::UntrackedKey(_))) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Retrieves a specific item from the account's storage map without loading the entire storage.
    /// The witness is retrieved from the [`AccountSmtForest`].
    pub(crate) fn get_account_map_item(
        conn: &mut Connection,
        smt_forest: &Arc<RwLock<AccountSmtForest>>,
        account_id: AccountId,
        slot_name: StorageSlotName,
        key: Word,
    ) -> Result<(Word, StorageMapWitness), StoreError> {
        let header = Self::get_account_header(conn, account_id)?
            .ok_or(StoreError::AccountDataNotFound(account_id))?
            .0;

        let mut storage_values = query_latest_storage_values(conn, account_id)?;
        storage_values.retain(|stored_name, _| stored_name == &slot_name);
        let (slot_type, map_root) = storage_values
            .remove(&slot_name)
            .ok_or(StoreError::AccountStorageRootNotFound(header.storage_commitment()))?;
        if slot_type != StorageSlotType::Map {
            return Err(StoreError::AccountError(AccountError::StorageSlotNotMap(slot_name)));
        }

        let smt_forest = smt_forest.read().expect("smt_forest read lock not poisoned");
        let witness = smt_forest.get_storage_map_item_witness(map_root, key)?;
        let item = witness.get(&key).unwrap_or(miden_client::EMPTY_WORD);

        Ok((item, witness))
    }

    pub(crate) fn get_account_addresses(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Vec<Address>, StoreError> {
        query_account_addresses(conn, account_id)
    }

    /// Retrieves the account code for a specific account by ID.
    pub(crate) fn get_account_code_by_id(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Option<AccountCode>, StoreError> {
        let Some((header, _)) =
            query_account_headers(conn, "accounts_latest", "id = ?", params![account_id.to_hex()])?
                .into_iter()
                .next()
        else {
            return Ok(None);
        };

        query_account_code(conn, header.code_commitment())
    }

    // MUTATOR/WRITER METHODS
    // --------------------------------------------------------------------------------------------

    pub(crate) fn insert_account(
        conn: &mut Connection,
        smt_forest: &Arc<RwLock<AccountSmtForest>>,
        account: &Account,
        initial_address: &Address,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;

        Self::insert_account_code(&tx, account.code())?;

        Self::upsert_account_storage_slot_updates(
            &tx,
            account.id(),
            account.nonce().as_int(),
            account.storage().slots().iter(),
        )?;

        Self::insert_assets(&tx, account.vault().root(), account.vault().assets())?;
        Self::insert_account_header(&tx, &account.into(), account.seed())?;

        Self::insert_address(&tx, initial_address, account.id())?;

        tx.commit().into_store_error()?;

        let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
        smt_forest.insert_account_state(account.vault(), account.storage())?;

        Ok(())
    }

    pub(crate) fn update_account(
        conn: &mut Connection,
        smt_forest: &Arc<RwLock<AccountSmtForest>>,
        new_account_state: &Account,
    ) -> Result<(), StoreError> {
        const QUERY: &str = "SELECT id FROM accounts_latest WHERE id = ?";
        if conn
            .prepare(QUERY)
            .into_store_error()?
            .query_map(params![new_account_state.id().to_hex()], |row| row.get(0))
            .into_store_error()?
            .map(|result| {
                let id: String = result.into_store_error()?;
                Self::parse_final_header_account_id(&id)
            })
            .next()
            .is_none()
        {
            return Err(StoreError::AccountDataNotFound(new_account_state.id()));
        }

        let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
        let tx = conn.transaction().into_store_error()?;
        Self::update_account_state(&tx, &mut smt_forest, new_account_state)?;
        tx.commit().into_store_error()
    }

    pub fn upsert_foreign_account_code(
        conn: &mut Connection,
        account_id: AccountId,
        code: &AccountCode,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;

        Self::insert_account_code(&tx, code)?;

        const QUERY: &str =
            insert_sql!(foreign_account_code { account_id, code_commitment } | REPLACE);

        tx.execute(QUERY, params![account_id.to_hex(), code.commitment().to_string()])
            .into_store_error()?;

        Self::insert_account_code(&tx, code)?;
        tx.commit().into_store_error()
    }

    pub(crate) fn insert_address(
        tx: &Transaction<'_>,
        address: &Address,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        let derived_note_tag = address.to_note_tag();
        let note_tag_record = NoteTagRecord::with_account_source(derived_note_tag, account_id);

        add_note_tag_tx(tx, &note_tag_record)?;
        Self::insert_address_internal(tx, address, account_id)?;

        Ok(())
    }

    pub(crate) fn remove_address(
        conn: &mut Connection,
        address: &Address,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        let derived_note_tag = address.to_note_tag();
        let note_tag_record = NoteTagRecord::with_account_source(derived_note_tag, account_id);

        let tx = conn.transaction().into_store_error()?;
        remove_note_tag_tx(&tx, note_tag_record)?;
        Self::remove_address_internal(&tx, address)?;

        tx.commit().into_store_error()
    }

    /// Inserts an [`AccountCode`].
    pub(crate) fn insert_account_code(
        tx: &Transaction<'_>,
        account_code: &AccountCode,
    ) -> Result<(), StoreError> {
        const QUERY: &str = insert_sql!(account_code { commitment, code } | IGNORE);
        tx.execute(QUERY, params![account_code.commitment().to_hex(), account_code.to_bytes()])
            .into_store_error()?;
        Ok(())
    }

    /// Applies the account delta to the account state, updating the vault and storage maps.
    ///
    /// The apply delta operation strats by copying over the initial account state (vault and
    /// storage) and then applying the delta on top of it. The storage and vault elements are
    /// overwritten in the new state. In the cases where the delta depends on previous state (e.g.
    /// adding or subtracting fungible assets), the previous state needs to be provided via the
    /// `updated_fungible_assets` and `updated_storage_maps` parameters.
    pub(crate) fn apply_account_delta(
        tx: &Transaction<'_>,
        smt_forest: &mut AccountSmtForest,
        init_account_state: &AccountHeader,
        final_account_state: &AccountHeader,
        updated_fungible_assets: BTreeMap<AccountIdPrefix, FungibleAsset>,
        updated_storage_maps: BTreeMap<StorageSlotName, StorageMap>,
        delta: &AccountDelta,
    ) -> Result<(), StoreError> {
        // Copy over the storage and vault from the previous state. Non-relevant data will not be
        // modified.
        Self::copy_account_state(tx, init_account_state, final_account_state)?;

        Self::apply_account_vault_delta(
            tx,
            smt_forest,
            init_account_state,
            final_account_state,
            updated_fungible_assets,
            delta,
        )?;

        let updated_storage_slots =
            Self::apply_account_storage_delta(smt_forest, updated_storage_maps, delta)?;

        Self::upsert_account_storage_slot_updates(
            tx,
            final_account_state.id(),
            final_account_state.nonce().as_int(),
            updated_storage_slots.values(),
        )?;

        Ok(())
    }

    /// Removes account states with the specified hashes from the database and pops their
    /// SMT roots from the forest to free up memory.
    ///
    /// This is used to rollback account changes when a transaction is discarded,
    /// effectively undoing the account state changes that were applied by the transaction.
    ///
    /// Note: This is not part of the Store trait and is only used internally by the `SQLite` store
    /// implementation to handle transaction rollbacks.
    pub(crate) fn undo_account_state(
        tx: &Transaction<'_>,
        smt_forest: &mut AccountSmtForest,
        account_commitments: &[Word],
    ) -> Result<(), StoreError> {
        if account_commitments.is_empty() {
            return Ok(());
        }

        let account_hash_params = Rc::new(
            account_commitments.iter().map(|h| Value::from(h.to_hex())).collect::<Vec<_>>(),
        );
        let deleted_account_states =
            Self::get_account_state_refs_by_commitment(tx, &account_hash_params)?;

        // Query all SMT roots before deletion so we can pop them from the forest
        let smt_roots = Self::get_smt_roots_by_account_commitment(tx, &account_hash_params)?;

        const DELETE_QUERY: &str =
            "DELETE FROM accounts_history WHERE account_commitment IN rarray(?)";
        tx.execute(DELETE_QUERY, params![account_hash_params]).into_store_error()?;

        if !deleted_account_states.is_empty() {
            Self::delete_storage_deltas_for_account_states(tx, &deleted_account_states)?;

            let affected_accounts: BTreeSet<AccountId> =
                deleted_account_states.iter().map(|(account_id, _)| *account_id).collect();
            for account_id in affected_accounts {
                Self::refresh_latest_account_header(tx, account_id)?;
                Self::refresh_latest_account_storage(tx, account_id)?;
            }
        }

        // Pop the roots from the forest to release memory for nodes that are no longer reachable
        smt_forest.pop_roots(smt_roots);

        Ok(())
    }

    /// Updates the account state in the database to a new complete account state.
    ///
    /// This function replaces the current account state with a completely new one. It:
    /// - Inserts the new account header
    /// - Stores all storage slots and their maps
    /// - Stores all vault assets
    /// - Updates the SMT forest with the new state
    /// - Pops old SMT roots from the forest to free memory
    ///
    /// This is typically used for replacing a full account state, as opposed to applying
    /// incremental deltas via `apply_account_delta`.
    ///
    /// # Arguments
    /// * `tx` - Database transaction
    /// * `smt_forest` - SMT forest for updating and pruning
    /// * `new_account_state` - The new complete account state to persist
    ///
    /// # Returns
    /// `Ok(())` if the update was successful, or an error if any operation fails.
    pub(crate) fn update_account_state(
        tx: &Transaction<'_>,
        smt_forest: &mut AccountSmtForest,
        new_account_state: &Account,
    ) -> Result<(), StoreError> {
        // Get old SMT roots before updating so we can prune them after
        let old_roots = Self::get_smt_roots_by_account_id(tx, new_account_state.id())?;

        smt_forest.insert_account_state(new_account_state.vault(), new_account_state.storage())?;
        Self::upsert_account_storage_slot_updates(
            tx,
            new_account_state.id(),
            new_account_state.nonce().as_int(),
            new_account_state.storage().slots().iter(),
        )?;
        Self::insert_assets(
            tx,
            new_account_state.vault().root(),
            new_account_state.vault().assets(),
        )?;
        Self::insert_account_header(tx, &new_account_state.into(), None)?;

        // Pop old roots to free memory for nodes no longer reachable
        smt_forest.pop_roots(old_roots);

        Ok(())
    }

    /// Locks the account if the mismatched digest doesn't belong to a previous account state (stale
    /// data).
    pub(crate) fn lock_account_on_unexpected_commitment(
        tx: &Transaction<'_>,
        account_id: &AccountId,
        mismatched_digest: &Word,
    ) -> Result<(), StoreError> {
        // Mismatched digests may be due to stale network data. If the mismatched digest is
        // tracked in the db and corresponds to the mismatched account, it means we
        // got a past update and shouldn't lock the account.
        const QUERY: &str = "UPDATE accounts_latest SET locked = true WHERE id = :account_id AND \
            NOT EXISTS (SELECT 1 FROM accounts_history WHERE id = :account_id AND account_commitment = :digest)";
        tx.execute(
            QUERY,
            named_params! {
                ":account_id": account_id.to_hex(),
                ":digest": mismatched_digest.to_string()
            },
        )
        .into_store_error()?;
        Ok(())
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

    fn parse_account_id(id: &str) -> Result<AccountId, StoreError> {
        AccountId::from_hex(id).map_err(StoreError::AccountIdError)
    }

    fn parse_final_header_account_id(id: &str) -> Result<AccountId, StoreError> {
        AccountId::from_hex(id).map_err(|err| {
            StoreError::AccountError(AccountError::FinalAccountHeaderIdParsingFailed(err))
        })
    }

    fn query_latest_account_nonce(
        tx: &Transaction<'_>,
        account_id_hex: &str,
    ) -> Result<Option<u64>, StoreError> {
        const QUERY: &str = "
            SELECT nonce
            FROM accounts_latest
            WHERE id = ?1";

        tx.prepare_cached(QUERY)
            .into_store_error()?
            .query_map(params![account_id_hex], |row| column_value_as_u64(row, 0))
            .into_store_error()?
            .next()
            .transpose()
            .into_store_error()
    }

    fn clear_storage_tracking_for_account(
        tx: &Transaction<'_>,
        account_id_hex: &str,
    ) -> Result<(), StoreError> {
        const DELETE_LATEST_QUERY: &str = "DELETE FROM account_storage_latest WHERE account_id = ?";
        const DELETE_DELTAS_QUERY: &str = "DELETE FROM account_storage_deltas WHERE account_id = ?";

        tx.execute(DELETE_LATEST_QUERY, params![account_id_hex]).into_store_error()?;
        tx.execute(DELETE_DELTAS_QUERY, params![account_id_hex]).into_store_error()?;

        Ok(())
    }

    fn query_stale_latest_slot_names(
        tx: &Transaction<'_>,
        account_id_hex: &str,
        latest_nonce: u64,
    ) -> Result<Vec<String>, StoreError> {
        const QUERY: &str = "
            SELECT slot_name
            FROM account_storage_latest
            WHERE account_id = ?1 AND nonce > ?2";

        tx.prepare_cached(QUERY)
            .into_store_error()?
            .query_map(params![account_id_hex, u64_to_value(latest_nonce)], |row| {
                row.get::<_, String>(0)
            })
            .into_store_error()?
            .collect::<Result<Vec<String>, _>>()
            .into_store_error()
    }

    fn query_previous_slot_value(
        previous_slot_stmt: &mut CachedStatement<'_>,
        account_id_hex: &str,
        slot_name: &str,
        latest_nonce: u64,
    ) -> Result<Option<StorageSlotValueNonceRow>, StoreError> {
        let mut rows = previous_slot_stmt
            .query(params![account_id_hex, slot_name, u64_to_value(latest_nonce)])
            .into_store_error()?;
        let Some(row) = rows.next().into_store_error()? else {
            return Ok(None);
        };

        Ok(Some(StorageSlotValueNonceRow::from_row(row).into_store_error()?))
    }

    fn restore_stale_latest_storage_slots(
        tx: &Transaction<'_>,
        account_id_hex: &str,
        latest_nonce: u64,
        stale_slot_names: Vec<String>,
    ) -> Result<(), StoreError> {
        const SELECT_PREVIOUS_SLOT_VALUE: &str = "
            SELECT slot_value, slot_type, nonce
            FROM account_storage_deltas
            WHERE account_id = ?1 AND slot_name = ?2 AND nonce <= ?3
            ORDER BY nonce DESC
            LIMIT 1";
        const DELETE_SLOT_QUERY: &str =
            "DELETE FROM account_storage_latest WHERE account_id = ?1 AND slot_name = ?2";
        const UPSERT_SLOT_QUERY: &str = insert_sql!(
            account_storage_latest {
                account_id,
                slot_name,
                slot_value,
                slot_type,
                nonce
            } | REPLACE
        );

        let mut previous_slot_stmt =
            tx.prepare_cached(SELECT_PREVIOUS_SLOT_VALUE).into_store_error()?;
        let mut upsert_slot_stmt = tx.prepare_cached(UPSERT_SLOT_QUERY).into_store_error()?;
        let mut delete_slot_stmt = tx.prepare_cached(DELETE_SLOT_QUERY).into_store_error()?;

        for slot_name in stale_slot_names {
            if let Some(previous_slot) = Self::query_previous_slot_value(
                &mut previous_slot_stmt,
                account_id_hex,
                slot_name.as_str(),
                latest_nonce,
            )? {
                upsert_slot_stmt
                    .execute(params![
                        account_id_hex,
                        slot_name.as_str(),
                        previous_slot.value.as_str(),
                        previous_slot.slot_type,
                        u64_to_value(previous_slot.nonce)
                    ])
                    .into_store_error()?;
            } else {
                delete_slot_stmt
                    .execute(params![account_id_hex, slot_name.as_str()])
                    .into_store_error()?;
            }
        }

        Ok(())
    }

    /// Inserts the new final account header and copies over the previous account state.
    fn copy_account_state(
        tx: &Transaction<'_>,
        init_account_header: &AccountHeader,
        final_account_header: &AccountHeader,
    ) -> Result<(), StoreError> {
        Self::insert_account_header(tx, final_account_header, None)?;

        if init_account_header.vault_root() != final_account_header.vault_root() {
            const VAULT_QUERY: &str = "
                INSERT OR IGNORE INTO account_assets (
                    root,
                    vault_key,
                    faucet_id_prefix,
                    asset
                )
                SELECT
                    ?, --new root
                    vault_key,
                    faucet_id_prefix,
                    asset
                FROM account_assets
                WHERE root = (SELECT vault_root FROM accounts_history WHERE account_commitment = ?)
                ";
            tx.execute(
                VAULT_QUERY,
                params![
                    final_account_header.vault_root().to_hex(),
                    init_account_header.commitment().to_hex()
                ],
            )
            .into_store_error()?;
        }

        Ok(())
    }

    fn get_account_state_refs_by_commitment(
        tx: &Transaction<'_>,
        account_hash_params: &Rc<Vec<Value>>,
    ) -> Result<Vec<(AccountId, u64)>, StoreError> {
        const QUERY: &str = "
            SELECT id, nonce
            FROM accounts_history
            WHERE account_commitment IN rarray(?1)";

        tx.prepare_cached(QUERY)
            .into_store_error()?
            .query_map(params![account_hash_params], |row| {
                Ok((row.get::<_, String>(0)?, column_value_as_u64(row, 1)?))
            })
            .into_store_error()?
            .map(|row| {
                let (account_id_hex, nonce) = row.into_store_error()?;
                let account_id =
                    AccountId::from_hex(&account_id_hex).map_err(StoreError::AccountIdError)?;
                Ok((account_id, nonce))
            })
            .collect()
    }

    fn delete_storage_deltas_for_account_states(
        tx: &Transaction<'_>,
        account_states: &[(AccountId, u64)],
    ) -> Result<(), StoreError> {
        const DELETE_QUERY: &str =
            "DELETE FROM account_storage_deltas WHERE account_id = ?1 AND nonce = ?2";
        let mut stmt = tx.prepare_cached(DELETE_QUERY).into_store_error()?;
        for (account_id, nonce) in account_states {
            stmt.execute(params![account_id.to_hex(), u64_to_value(*nonce)])
                .into_store_error()?;
        }

        Ok(())
    }

    fn refresh_latest_account_header(
        tx: &Transaction<'_>,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        const DELETE_LATEST_QUERY: &str = "DELETE FROM accounts_latest WHERE id = ?1";
        const INSERT_LATEST_QUERY: &str = "
            INSERT INTO accounts_latest (
                id,
                account_commitment,
                code_commitment,
                storage_commitment,
                vault_root,
                nonce,
                account_seed,
                locked
            )
            SELECT
                id,
                account_commitment,
                code_commitment,
                storage_commitment,
                vault_root,
                nonce,
                account_seed,
                locked
            FROM accounts_history
            WHERE id = ?1
            ORDER BY nonce DESC
            LIMIT 1";

        let account_id_hex = account_id.to_hex();
        tx.execute(DELETE_LATEST_QUERY, params![account_id_hex.as_str()])
            .into_store_error()?;
        tx.execute(INSERT_LATEST_QUERY, params![account_id_hex.as_str()])
            .into_store_error()?;

        Ok(())
    }

    fn refresh_latest_account_storage(
        tx: &Transaction<'_>,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        let account_id_hex = account_id.to_hex();
        let Some(latest_nonce) = Self::query_latest_account_nonce(tx, &account_id_hex)? else {
            return Self::clear_storage_tracking_for_account(tx, &account_id_hex);
        };

        let stale_slot_names =
            Self::query_stale_latest_slot_names(tx, &account_id_hex, latest_nonce)?;

        if stale_slot_names.is_empty() {
            return Ok(());
        }

        Self::restore_stale_latest_storage_slots(
            tx,
            &account_id_hex,
            latest_nonce,
            stale_slot_names,
        )
    }

    /// Returns all SMT roots for a given account ID's latest state.
    ///
    /// This function retrieves all Merkle tree roots needed for the SMT forest, including:
    /// - The vault root for all asset nodes
    /// - All storage map roots for storage slot map nodes
    fn get_smt_roots_by_account_id(
        tx: &Transaction<'_>,
        account_id: AccountId,
    ) -> Result<Vec<Word>, StoreError> {
        const LATEST_ACCOUNT_QUERY: &str = r"
        SELECT vault_root
        FROM accounts_latest
        WHERE id = ?1
    ";

        // 1) Fetch latest vault root.
        let vault_root: String = tx
            .query_row(LATEST_ACCOUNT_QUERY, params![account_id.to_hex()], |row| row.get(0))
            .into_store_error()?;

        let mut roots = Vec::new();

        // Always include the vault root.
        if let Ok(root) = Word::try_from(vault_root.as_str()) {
            roots.push(root);
        }

        // 2) Fetch storage map roots from latest storage snapshot.
        roots.extend(query_latest_storage_map_roots(tx, account_id)?);

        Ok(roots)
    }

    /// Returns all SMT roots (vault root + storage map roots) for the given account commitments.
    fn get_smt_roots_by_account_commitment(
        tx: &Transaction<'_>,
        account_hash_params: &Rc<Vec<Value>>,
    ) -> Result<Vec<Word>, StoreError> {
        const ACCOUNT_STATES_QUERY: &str = "
            SELECT id, nonce, vault_root
            FROM accounts_history
            WHERE account_commitment IN rarray(?1)";

        let mut roots = Vec::new();

        let mut account_stmt = tx.prepare(ACCOUNT_STATES_QUERY).into_store_error()?;
        let account_rows = account_stmt
            .query_map(params![account_hash_params], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    column_value_as_u64(row, 1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .into_store_error()?;

        for account_row in account_rows {
            let (account_id_hex, nonce, vault_root) = account_row.into_store_error()?;
            if let Ok(vault_root) = Word::try_from(vault_root.as_str()) {
                roots.push(vault_root);
            }

            let account_id =
                AccountId::from_hex(&account_id_hex).map_err(StoreError::AccountIdError)?;
            roots.extend(query_storage_map_roots_at_or_before_nonce(tx, account_id, nonce)?);
        }

        Ok(roots)
    }

    /// Inserts a new account record into the database.
    fn insert_account_header(
        tx: &Transaction<'_>,
        account: &AccountHeader,
        account_seed: Option<Word>,
    ) -> Result<(), StoreError> {
        let id: String = account.id().to_hex();
        let code_commitment = account.code_commitment().to_string();
        let storage_commitment = account.storage_commitment().to_string();
        let vault_root = account.vault_root().to_string();
        let nonce = u64_to_value(account.nonce().as_int());
        let commitment = account.commitment().to_string();

        let account_seed = account_seed.map(|seed| seed.to_bytes());

        const INSERT_HISTORY_QUERY: &str = insert_sql!(
            accounts_history {
                id,
                code_commitment,
                storage_commitment,
                vault_root,
                nonce,
                account_seed,
                account_commitment,
                locked
            } | REPLACE
        );
        const UPSERT_LATEST_QUERY: &str = "
            INSERT INTO accounts_latest (
                id,
                account_commitment,
                code_commitment,
                storage_commitment,
                vault_root,
                nonce,
                account_seed,
                locked
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id)
            DO UPDATE SET
                account_commitment = excluded.account_commitment,
                code_commitment = excluded.code_commitment,
                storage_commitment = excluded.storage_commitment,
                vault_root = excluded.vault_root,
                nonce = excluded.nonce,
                account_seed = excluded.account_seed,
                locked = excluded.locked
            WHERE excluded.nonce >= accounts_latest.nonce";

        tx.execute(
            INSERT_HISTORY_QUERY,
            params![
                &id,
                &code_commitment,
                &storage_commitment,
                &vault_root,
                &nonce,
                &account_seed,
                &commitment,
                false,
            ],
        )
        .into_store_error()?;
        tx.execute(
            UPSERT_LATEST_QUERY,
            params![
                &id,
                &commitment,
                &code_commitment,
                &storage_commitment,
                &vault_root,
                &nonce,
                &account_seed,
                false,
            ],
        )
        .into_store_error()?;

        Self::insert_tracked_account_id_tx(tx, account.id())?;
        Ok(())
    }

    fn insert_tracked_account_id_tx(
        tx: &Transaction<'_>,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        const QUERY: &str = insert_sql!(tracked_accounts { id } | IGNORE);
        tx.execute(QUERY, params![account_id.to_hex()]).into_store_error()?;
        Ok(())
    }

    fn insert_address_internal(
        tx: &Transaction<'_>,
        address: &Address,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        const QUERY: &str = insert_sql!(addresses { address, account_id } | REPLACE);
        let serialized_address = address.to_bytes();
        tx.execute(QUERY, params![serialized_address, account_id.to_hex(),])
            .into_store_error()?;

        Ok(())
    }

    fn remove_address_internal(tx: &Transaction<'_>, address: &Address) -> Result<(), StoreError> {
        let serialized_address = address.to_bytes();

        const DELETE_QUERY: &str = "DELETE FROM addresses WHERE address = ?";
        tx.execute(DELETE_QUERY, params![serialized_address]).into_store_error()?;

        Ok(())
    }
}
