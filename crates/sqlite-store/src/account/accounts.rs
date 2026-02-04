//! Account-related database operations.

use std::collections::BTreeMap;
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
    PrunableAccountData,
    PrunableAccountState,
    StoreError,
};
use miden_client::sync::NoteTagRecord;
use miden_client::transaction::TransactionDetails;
use miden_client::utils::Serializable;
use miden_client::{AccountError, Word};
use miden_protocol::account::{AccountStorageHeader, StorageMapWitness, StorageSlotHeader};
use miden_protocol::asset::{AssetVaultKey, PartialVault};
use miden_protocol::crypto::merkle::MerkleError;
use rusqlite::types::Value;
use rusqlite::{Connection, Transaction, named_params, params};

use crate::account::helpers::{
    SerializedHeaderData,
    parse_accounts,
    query_account_addresses,
    query_account_code,
    query_account_headers,
    query_storage_maps,
    query_storage_slots,
    query_storage_values,
    query_vault_assets,
};
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
            .expect("no binding parameters used in query")
            .map(|result| {
                let id: String = result.map_err(|e| StoreError::ParsingError(e.to_string()))?;
                Ok(AccountId::from_hex(&id).expect("account id is valid"))
            })
            .collect::<Result<Vec<AccountId>, StoreError>>()
    }

    pub(crate) fn get_account_headers(
        conn: &mut Connection,
    ) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
        const QUERY: &str = "
            SELECT
                a.id,
                a.nonce,
                a.vault_root,
                a.storage_commitment,
                a.code_commitment,
                a.account_seed,
                a.locked
            FROM accounts AS a
            JOIN (
                SELECT id, MAX(nonce) AS nonce
                FROM accounts
                GROUP BY id
            ) AS latest
            ON a.id = latest.id
            AND a.nonce = latest.nonce
            ORDER BY a.id;
            ";

        conn.prepare_cached(QUERY)
            .into_store_error()?
            .query_map(params![], |row| {
                let id: String = row.get(0)?;
                let nonce: u64 = column_value_as_u64(row, 1)?;
                let vault_root: String = row.get(2)?;
                let storage_commitment: String = row.get(3)?;
                let code_commitment: String = row.get(4)?;
                let account_seed: Option<Vec<u8>> = row.get(5)?;
                let locked: bool = row.get(6)?;

                Ok(SerializedHeaderData {
                    id,
                    nonce,
                    vault_root,
                    storage_commitment,
                    code_commitment,
                    account_seed,
                    locked,
                })
            })
            .into_store_error()?
            .map(|result| parse_accounts(result.into_store_error()?))
            .collect::<Result<Vec<(AccountHeader, AccountStatus)>, StoreError>>()
    }

    pub(crate) fn get_account_header(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        Ok(query_account_headers(
            conn,
            "id = ? ORDER BY nonce DESC LIMIT 1",
            params![account_id.to_hex()],
        )?
        .pop())
    }

    pub(crate) fn get_account_header_by_commitment(
        conn: &mut Connection,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError> {
        let account_commitment_str: String = account_commitment.to_string();
        Ok(
            query_account_headers(conn, "account_commitment = ?", params![account_commitment_str])?
                .pop()
                .map(|(header, _)| header),
        )
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

        let slots = query_storage_slots(
            conn,
            "commitment = ?",
            params![header.storage_commitment().to_hex()],
        )?
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

        let addresses = query_account_addresses(conn, header.id())?;
        let account_data = AccountRecordData::Full(account);
        Ok(Some(AccountRecord::new(account_data, status, addresses)))
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

        let storage_values = query_storage_values(
            conn,
            "commitment = ?",
            params![header.storage_commitment().to_hex()],
        )?;

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
        let addresses = query_account_addresses(conn, header.id())?;
        Ok(Some(AccountRecord::new(account_record_data, status, addresses)))
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
            .expect("no binding parameters used in query")
            .map(|result| {
                result.map_err(|err| StoreError::ParsingError(err.to_string())).and_then(
                    |(id, code): (String, Vec<u8>)| {
                        Ok((
                            AccountId::from_hex(&id).map_err(|err| {
                                StoreError::AccountError(
                                    AccountError::FinalAccountHeaderIdParsingFailed(err),
                                )
                            })?,
                            AccountCode::from_bytes(&code).map_err(StoreError::AccountError)?,
                        ))
                    },
                )
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
            "root = (SELECT vault_root FROM accounts WHERE id = ? ORDER BY nonce DESC LIMIT 1)",
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
        let (where_clause, params) = match filter {
            AccountStorageFilter::All => (
                "commitment = (SELECT storage_commitment FROM accounts WHERE id = ? ORDER BY nonce DESC LIMIT 1)",
                params![account_id.to_hex()],
            ),
            AccountStorageFilter::Root(root) => (
                "commitment = (SELECT storage_commitment FROM accounts WHERE id = ? ORDER BY nonce DESC LIMIT 1) AND slot_value = ?",
                params![account_id.to_hex(), root.to_hex()],
            ),
            AccountStorageFilter::SlotName(slot_name) => (
                "commitment = (SELECT storage_commitment FROM accounts WHERE id = ? ORDER BY nonce DESC LIMIT 1) AND slot_name = ?",
                params![account_id.to_hex(), slot_name.to_string()],
            ),
        };

        let slots = query_storage_slots(conn, where_clause, params)?.into_values().collect();

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

        let mut storage_values = query_storage_values(
            conn,
            "commitment = ? AND slot_name = ?",
            params![header.storage_commitment().to_hex(), slot_name.to_string()],
        )?;
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

        Self::insert_storage_slots(
            &tx,
            account.storage().to_commitment(),
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
        const QUERY: &str = "SELECT id FROM accounts WHERE id = ?";
        if conn
            .prepare(QUERY)
            .into_store_error()?
            .query_map(params![new_account_state.id().to_hex()], |row| row.get(0))
            .into_store_error()?
            .map(|result| {
                result.map_err(|err| StoreError::ParsingError(err.to_string())).and_then(
                    |id: String| {
                        AccountId::from_hex(&id).map_err(|err| {
                            StoreError::AccountError(
                                AccountError::FinalAccountHeaderIdParsingFailed(err),
                            )
                        })
                    },
                )
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

        Self::insert_storage_slots(
            tx,
            final_account_state.storage_commitment(),
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

        // Query all SMT roots before deletion so we can pop them from the forest
        let smt_roots = Self::get_smt_roots_by_account_commitment(tx, &account_hash_params)?;

        const DELETE_QUERY: &str = "DELETE FROM accounts WHERE account_commitment IN rarray(?)";
        tx.execute(DELETE_QUERY, params![account_hash_params]).into_store_error()?;

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
        Self::insert_storage_slots(
            tx,
            new_account_state.storage().to_commitment(),
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
        const QUERY: &str = "UPDATE accounts SET locked = true WHERE id = :account_id AND NOT EXISTS (SELECT 1 FROM accounts WHERE id = :account_id AND account_commitment = :digest)";
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
                WHERE root = (SELECT vault_root FROM accounts WHERE account_commitment = ?)
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

        if init_account_header.storage_commitment() != final_account_header.storage_commitment() {
            const STORAGE_QUERY: &str = "
                INSERT OR IGNORE INTO account_storage (
                    commitment,
                    slot_name,
                    slot_value,
                    slot_type
                )
                SELECT
                    ?, -- new commitment
                    slot_name,
                    slot_value,
                    slot_type
                FROM account_storage
                WHERE commitment = (SELECT storage_commitment FROM accounts WHERE account_commitment = ?)
                ";

            tx.execute(
                STORAGE_QUERY,
                params![
                    final_account_header.storage_commitment().to_hex(),
                    init_account_header.commitment().to_hex()
                ],
            )
            .into_store_error()?;
        }

        Ok(())
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
        SELECT vault_root, storage_commitment
        FROM accounts
        WHERE id = ?1
        ORDER BY nonce DESC
        LIMIT 1
    ";

        const STORAGE_MAP_ROOTS_QUERY: &str = r"
        SELECT slot_value
        FROM account_storage
        WHERE commitment = ?1
          AND slot_type = ?2
          AND slot_value IS NOT NULL
    ";

        let map_slot_type = StorageSlotType::Map as u8;

        // 1) Fetch latest vault root + storage commitment.
        let (vault_root, storage_commitment): (String, String) = tx
            .query_row(LATEST_ACCOUNT_QUERY, params![account_id.to_hex()], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .into_store_error()?;

        let mut roots = Vec::new();

        // Always include the vault root.
        if let Ok(root) = Word::try_from(vault_root.as_str()) {
            roots.push(root);
        }

        // 2) Fetch storage map roots for the latest storage commitment.
        let mut stmt = tx.prepare(STORAGE_MAP_ROOTS_QUERY).into_store_error()?;
        let iter = stmt
            .query_map(params![storage_commitment, map_slot_type], |row| row.get::<_, String>(0))
            .into_store_error()?;

        roots.extend(iter.filter_map(Result::ok).filter_map(|r| Word::try_from(r.as_str()).ok()));

        Ok(roots)
    }

    /// Returns all SMT roots (vault root + storage map roots) for the given account commitments.
    fn get_smt_roots_by_account_commitment(
        tx: &Transaction<'_>,
        account_hash_params: &Rc<Vec<Value>>,
    ) -> Result<Vec<Word>, StoreError> {
        const ROOTS_QUERY: &str = "
            SELECT vault_root FROM accounts WHERE account_commitment IN rarray(?1)
            UNION ALL
            SELECT slot_value FROM account_storage
            WHERE commitment IN (
                SELECT storage_commitment FROM accounts WHERE account_commitment IN rarray(?1)
            ) AND slot_type = ?2";

        let map_slot_type = StorageSlotType::Map as u8;
        let mut stmt = tx.prepare(ROOTS_QUERY).into_store_error()?;
        let roots = stmt
            .query_map(params![account_hash_params, map_slot_type], |row| row.get::<_, String>(0))
            .into_store_error()?
            .filter_map(Result::ok)
            .filter_map(|r| Word::try_from(r.as_str()).ok())
            .collect();

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

        const QUERY: &str = insert_sql!(
            accounts {
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

        tx.execute(
            QUERY,
            params![
                id,
                code_commitment,
                storage_commitment,
                vault_root,
                nonce,
                account_seed,
                commitment,
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

    // PRUNING METHODS
    // --------------------------------------------------------------------------------------------

    /// Prunes old account states, keeping:
    /// - The latest state (highest nonce)
    /// - The initial state (account_seed IS NOT NULL)
    /// - Any state that is the final_account_state of a pending transaction
    ///
    /// # Safety
    ///
    /// This method contains multiple safety checks:
    /// 1. Explicitly finds what to KEEP (latest + initial + pending tx states)
    /// 2. Only deletes states that are NOT in the keep list
    /// 3. Verifies after deletion that the latest state is still accessible
    /// 4. All operations are in a single transaction (atomic rollback on failure)
    pub(crate) fn prune_account_history(
        conn: &mut Connection,
        smt_forest: &Arc<RwLock<AccountSmtForest>>,
        account_id: AccountId,
    ) -> Result<PrunableAccountData, StoreError> {
        let id_hex = account_id.to_hex();

        // Start a transaction - all changes will be rolled back if anything fails
        let tx = conn.transaction().into_store_error()?;

        // =====================================================================
        // STEP 1: Find the latest state (this MUST be kept)
        // =====================================================================
        const LATEST_STATE_QUERY: &str = "
            SELECT account_commitment, nonce, vault_root, storage_commitment
            FROM accounts
            WHERE id = ?
            ORDER BY nonce DESC
            LIMIT 1
        ";

        let latest_state: Option<(String, i64, String, String)> = tx
            .query_row(LATEST_STATE_QUERY, params![&id_hex], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .ok();

        // If there's no state, there's nothing to prune
        let Some((latest_commitment, _, _, _)) = latest_state else {
            tx.commit().into_store_error()?;
            return Ok(PrunableAccountData::default());
        };

        // =====================================================================
        // STEP 2: Get all states referenced by pending transactions
        // We must preserve BOTH:
        // - init_account_state: the rollback point if the transaction is discarded
        // - final_account_state: the state if the transaction commits
        // =====================================================================
        const PENDING_TX_STATES_QUERY: &str = "
            SELECT details FROM transactions
            WHERE status_variant = 0
        ";
        let mut pending_protected_states: Vec<String> = Vec::new();
        for details_blob in tx
            .prepare(PENDING_TX_STATES_QUERY)
            .into_store_error()?
            .query_map([], |row| {
                let details_blob: Vec<u8> = row.get(0)?;
                Ok(details_blob)
            })
            .into_store_error()?
            .filter_map(|r| r.ok())
        {
            use miden_client::utils::Deserializable;
            if let Ok(details) = TransactionDetails::read_from_bytes(&details_blob) {
                if details.account_id == account_id {
                    // Protect BOTH init and final states
                    pending_protected_states.push(details.init_account_state.to_hex());
                    pending_protected_states.push(details.final_account_state.to_hex());
                }
            }
        }
        // Deduplicate (in case init of one tx = final of another)
        pending_protected_states.sort();
        pending_protected_states.dedup();

        // =====================================================================
        // STEP 3: Find all states that CAN be pruned
        // SAFETY: Exclude the latest state
        // SAFETY: Exclude initial state (account_seed IS NOT NULL)
        // SAFETY: Exclude states referenced by pending transactions (both init and final)
        // =====================================================================
        let prunable_rows: Vec<(String, i64, String, String)> = {
            let prunable_states_query = if pending_protected_states.is_empty() {
                "SELECT account_commitment, nonce, vault_root, storage_commitment
                 FROM accounts
                 WHERE id = ?
                   AND account_seed IS NULL
                   AND account_commitment != ?".to_string()
            } else {
                format!(
                    "SELECT account_commitment, nonce, vault_root, storage_commitment
                     FROM accounts
                     WHERE id = ?
                       AND account_seed IS NULL
                       AND account_commitment != ?
                       AND account_commitment NOT IN ({})",
                    pending_protected_states.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", ")
                )
            };

            let mut stmt = tx.prepare(&prunable_states_query).into_store_error()?;
            stmt.query_map(params![&id_hex, &latest_commitment], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .into_store_error()?
            .filter_map(|r| r.ok())
            .collect()
        };

        // If nothing to prune, we're done
        if prunable_rows.is_empty() {
            tx.commit().into_store_error()?;
            return Ok(PrunableAccountData::default());
        }

        // =====================================================================
        // STEP 4: SAFETY ASSERTIONS before deletion
        // =====================================================================

        // SAFETY: Verify latest state is NOT in the prunable list
        for (commitment, _, _, _) in &prunable_rows {
            assert_ne!(
                commitment, &latest_commitment,
                "SAFETY VIOLATION: Latest state found in prunable list!"
            );
            // SAFETY: Verify pending transaction states are NOT in the prunable list
            assert!(
                !pending_protected_states.contains(commitment),
                "SAFETY VIOLATION: Pending transaction state found in prunable list!"
            );
        }

        // Build the list of commitments to delete
        let commitments_to_delete: Vec<String> =
            prunable_rows.iter().map(|(c, _, _, _)| c.clone()).collect();

        // Collect SMT roots that need to be freed
        let mut roots_to_pop: Vec<Word> = Vec::new();
        for (_, _, vault_root, storage_commitment) in &prunable_rows {
            if let Ok(root) = Word::try_from(vault_root.as_str()) {
                roots_to_pop.push(root);
            }
            // Also collect storage map roots for this storage commitment
            const MAP_ROOTS_QUERY: &str = "
                SELECT slot_value FROM account_storage
                WHERE commitment = ? AND slot_type = 1 AND slot_value IS NOT NULL
            ";
            if let Ok(mut map_stmt) = tx.prepare(MAP_ROOTS_QUERY) {
                let map_roots: Vec<String> = map_stmt
                    .query_map(params![storage_commitment], |row| row.get(0))
                    .into_iter()
                    .flatten()
                    .filter_map(|r| r.ok())
                    .collect();
                for root_str in map_roots {
                    if let Ok(root) = Word::try_from(root_str.as_str()) {
                        roots_to_pop.push(root);
                    }
                }
            }
        }

        // Build prunable states for the return value
        let prunable_states: Vec<PrunableAccountState> = prunable_rows
            .iter()
            .filter_map(|(commitment, nonce, _, _)| {
                Word::try_from(commitment.as_str()).ok().map(|c| PrunableAccountState {
                    account_id,
                    nonce: *nonce as u64,
                    commitment: c,
                })
            })
            .collect();

        // =====================================================================
        // STEP 5: Delete the prunable account states
        // =====================================================================
        for commitment in &commitments_to_delete {
            const DELETE_ACCOUNT: &str = "DELETE FROM accounts WHERE account_commitment = ?";
            tx.execute(DELETE_ACCOUNT, params![commitment]).into_store_error()?;
        }

        // =====================================================================
        // STEP 6: Clean up orphaned data in related tables
        // =====================================================================

        // Delete orphaned storage rows (not referenced by any remaining account)
        const DELETE_ORPHAN_STORAGE: &str = "
            DELETE FROM account_storage
            WHERE commitment NOT IN (SELECT storage_commitment FROM accounts)
        ";
        let orphaned_storage = tx.execute(DELETE_ORPHAN_STORAGE, []).into_store_error()?;

        // Delete orphaned asset rows
        const DELETE_ORPHAN_ASSETS: &str = "
            DELETE FROM account_assets
            WHERE root NOT IN (SELECT vault_root FROM accounts)
        ";
        let orphaned_assets = tx.execute(DELETE_ORPHAN_ASSETS, []).into_store_error()?;

        // Delete orphaned storage map entries
        const DELETE_ORPHAN_MAPS: &str = "
            DELETE FROM storage_map_entries
            WHERE root NOT IN (
                SELECT slot_value FROM account_storage WHERE slot_type = 1
            )
        ";
        let orphaned_maps = tx.execute(DELETE_ORPHAN_MAPS, []).into_store_error()?;

        // =====================================================================
        // STEP 7: VERIFICATION - Ensure latest state is still accessible
        // =====================================================================
        const VERIFY_LATEST: &str = "
            SELECT account_commitment FROM accounts
            WHERE id = ?
            ORDER BY nonce DESC
            LIMIT 1
        ";
        let verified_latest: Option<String> = tx
            .query_row(VERIFY_LATEST, params![&id_hex], |row| row.get(0))
            .ok();

        // SAFETY: Verify the latest state still exists
        assert_eq!(
            verified_latest.as_ref(),
            Some(&latest_commitment),
            "SAFETY VIOLATION: Latest state was deleted!"
        );

        // =====================================================================
        // STEP 8: Commit transaction and free SMT memory
        // =====================================================================
        tx.commit().into_store_error()?;

        // Note: We don't call smt_forest.pop_roots() here because:
        // 1. The SMT forest is loaded at startup with only the current account state
        // 2. Old pruned states' roots may not be in the forest
        // 3. Calling pop_roots on non-existent roots would cause a panic
        //
        // The memory for these old roots was never loaded, so there's nothing to free.
        // If we need to optimize memory in the future, we should track which roots
        // are actually in the forest before attempting to pop them.
        let _ = (smt_forest, roots_to_pop); // Suppress unused warnings

        Ok(PrunableAccountData {
            states: prunable_states,
            orphaned_storage_rows: orphaned_storage,
            orphaned_asset_rows: orphaned_assets,
            orphaned_map_entries: orphaned_maps,
        })
    }
}
