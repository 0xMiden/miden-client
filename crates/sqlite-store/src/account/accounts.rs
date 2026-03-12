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
    StorageMapKey,
    StorageSlotName,
    StorageSlotType,
};
use miden_client::asset::{Asset, AssetVault, AssetWitness, FungibleAsset};
use miden_client::store::{
    AccountRecord,
    AccountRecordData,
    AccountSmtForest,
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
use rusqlite::{Connection, OptionalExtension, Transaction, named_params, params};

use crate::account::helpers::{
    query_account_addresses,
    query_account_code,
    query_historical_account_headers,
    query_latest_account_headers,
    query_storage_slots,
    query_storage_values,
    query_vault_assets,
};
use crate::sql_error::SqlResultExt;
use crate::sync::{add_note_tag_tx, remove_note_tag_tx};
use crate::{SqliteStore, column_value_as_u64, insert_sql, subst, u64_to_value};

impl SqliteStore {
    // READER METHODS
    // --------------------------------------------------------------------------------------------

    pub(crate) fn get_account_ids(conn: &mut Connection) -> Result<Vec<AccountId>, StoreError> {
        const QUERY: &str = "SELECT id FROM latest_account_headers";

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
        query_latest_account_headers(conn, "1=1 ORDER BY id", params![])
    }

    pub(crate) fn get_account_header(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        Ok(query_latest_account_headers(conn, "id = ?", params![account_id.to_hex()])?.pop())
    }

    pub(crate) fn get_account_header_by_commitment(
        conn: &mut Connection,
        account_commitment: Word,
    ) -> Result<Option<AccountHeader>, StoreError> {
        let account_commitment_str: String = account_commitment.to_string();
        Ok(query_historical_account_headers(
            conn,
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

        let assets = query_vault_assets(conn, account_id)?;
        let vault = AssetVault::new(&assets)?;

        let slots = query_storage_slots(conn, account_id, &AccountStorageFilter::All)?
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

        let storage_values = query_storage_values(conn, account_id)?;

        // Storage maps are always minimal here (just roots, no entries).
        // New accounts that need full storage data are handled by the DataStore layer,
        // which fetches the full account via `get_account()` when nonce == 0.
        for (slot_name, (slot_type, value)) in storage_values {
            storage_header.push(StorageSlotHeader::new(slot_name.clone(), slot_type, value));
            if slot_type == StorageSlotType::Map {
                maps.push(PartialStorageMap::new(value));
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
        let assets = query_vault_assets(conn, account_id)?;
        Ok(AssetVault::new(&assets)?)
    }

    /// Retrieves the full storage for a specific account.
    pub fn get_account_storage(
        conn: &Connection,
        account_id: AccountId,
        filter: &AccountStorageFilter,
    ) -> Result<AccountStorage, StoreError> {
        let slots = query_storage_slots(conn, account_id, filter)?.into_values().collect();
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
        key: StorageMapKey,
    ) -> Result<(Word, StorageMapWitness), StoreError> {
        let header = Self::get_account_header(conn, account_id)?
            .ok_or(StoreError::AccountDataNotFound(account_id))?
            .0;

        let mut storage_values = query_storage_values(conn, account_id)?;
        let (slot_type, map_root) = storage_values
            .remove(&slot_name)
            .ok_or(StoreError::AccountStorageRootNotFound(header.storage_commitment()))?;
        if slot_type != StorageSlotType::Map {
            return Err(StoreError::AccountError(AccountError::StorageSlotNotMap(slot_name)));
        }

        let smt_forest = smt_forest.read().expect("smt_forest read lock not poisoned");
        let witness = smt_forest.get_storage_map_item_witness(map_root, key)?;
        let item = witness.get(key).unwrap_or(miden_client::EMPTY_WORD);

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
            query_latest_account_headers(conn, "id = ?", params![account_id.to_hex()])?
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

        let account_id = account.id();

        Self::insert_storage_slots(&tx, account_id, account.storage().slots().iter())?;

        Self::insert_assets(&tx, account_id, account.vault().assets())?;
        Self::insert_account_header(&tx, &account.into(), account.seed(), None)?;

        Self::insert_address(&tx, initial_address, account.id())?;

        tx.commit().into_store_error()?;

        let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
        smt_forest.insert_and_register_account_state(
            account.id(),
            account.vault(),
            account.storage(),
        )?;

        Ok(())
    }

    pub(crate) fn update_account(
        conn: &mut Connection,
        smt_forest: &Arc<RwLock<AccountSmtForest>>,
        new_account_state: &Account,
    ) -> Result<(), StoreError> {
        const QUERY: &str = "SELECT id FROM latest_account_headers WHERE id = ?";
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
    /// Archives old values from latest to historical and updates latest via INSERT OR REPLACE.
    pub(crate) fn apply_account_delta(
        tx: &Transaction<'_>,
        smt_forest: &mut AccountSmtForest,
        init_account_state: &AccountHeader,
        final_account_state: &AccountHeader,
        updated_fungible_assets: BTreeMap<AccountIdPrefix, FungibleAsset>,
        old_map_roots: &BTreeMap<StorageSlotName, Word>,
        delta: &AccountDelta,
    ) -> Result<(), StoreError> {
        let account_id = final_account_state.id();

        // Archive old header and insert the new one
        Self::insert_account_header(tx, final_account_state, None, Some(init_account_state))?;

        Self::apply_account_vault_delta(
            tx,
            smt_forest,
            account_id,
            init_account_state,
            final_account_state,
            updated_fungible_assets,
            delta,
        )?;

        // Build the final roots from the init state's registered roots:
        // - Replace vault root with the final one
        // - Replace changed map roots with their new values (done by apply_account_storage_delta)
        // - Unchanged map roots continue as they were
        let mut final_roots = smt_forest
            .get_roots(&init_account_state.id())
            .cloned()
            .ok_or(StoreError::AccountDataNotFound(init_account_state.id()))?;

        // First element is always the vault root
        if let Some(vault_root) = final_roots.first_mut() {
            *vault_root = final_account_state.vault_root();
        }

        let default_map_root = StorageMap::default().root();
        let updated_storage_slots =
            Self::apply_account_storage_delta(smt_forest, old_map_roots, delta)?;

        // Update map roots in final_roots with new values from the delta
        for (slot_name, (new_root, slot_type)) in &updated_storage_slots {
            if *slot_type == StorageSlotType::Map {
                let old_root = old_map_roots.get(slot_name).copied().unwrap_or(default_map_root);
                if let Some(root) = final_roots.iter_mut().find(|r| **r == old_root) {
                    *root = *new_root;
                } else {
                    // New map slot not in the old roots — append it
                    final_roots.push(*new_root);
                }
            }
        }

        Self::write_storage_delta(
            tx,
            account_id,
            final_account_state.nonce().as_int(),
            &updated_storage_slots,
            delta,
        )?;

        smt_forest.stage_roots(final_account_state.id(), final_roots);

        Ok(())
    }

    /// Undoes discarded account states by directly restoring old values from historical.
    ///
    /// For each discarded state, the old values from historical are restored to latest.
    /// Entries with NULL old values (genuinely new entries) are deleted from latest.
    /// Multiple nonces per account are undone in reverse order (most recent first).
    pub(crate) fn undo_account_state(
        tx: &Transaction<'_>,
        smt_forest: &mut AccountSmtForest,
        discarded_states: &[(AccountId, Word)],
    ) -> Result<(), StoreError> {
        if discarded_states.is_empty() {
            return Ok(());
        }

        let commitment_params = Rc::new(
            discarded_states
                .iter()
                .map(|(_, commitment)| Value::from(commitment.to_hex()))
                .collect::<Vec<_>>(),
        );

        // Resolve (account_id, nonce) pairs for the discarded states.
        // Old headers are in historical and the most recent discarded state is in
        // latest, so we query both tables.
        let mut id_nonce_pairs: Vec<(String, u64)> = Vec::new();
        for query in [
            "SELECT id, nonce FROM latest_account_headers WHERE account_commitment IN rarray(?)",
            "SELECT id, nonce FROM historical_account_headers WHERE account_commitment IN rarray(?)",
        ] {
            id_nonce_pairs.extend(
                tx.prepare(query)
                    .into_store_error()?
                    .query_map(params![commitment_params.clone()], |row| {
                        let id: String = row.get(0)?;
                        let nonce: u64 = column_value_as_u64(row, 1)?;
                        Ok((id, nonce))
                    })
                    .into_store_error()?
                    .filter_map(Result::ok),
            );
        }

        // Group nonces by account and sort descending (undo most recent first)
        let mut nonces_by_account: BTreeMap<String, Vec<u64>> = BTreeMap::new();
        for (id, nonce) in &id_nonce_pairs {
            nonces_by_account.entry(id.clone()).or_default().push(*nonce);
        }
        for nonces in nonces_by_account.values_mut() {
            nonces.sort_unstable();
            nonces.dedup();
            nonces.reverse();
        }

        for (account_id_hex, nonces) in &nonces_by_account {
            Self::undo_account_nonces(tx, account_id_hex, nonces)?;
        }

        // Discard each rolled-back state from the in-memory forest
        for (account_id, _) in discarded_states {
            smt_forest.discard_roots(*account_id);
        }

        Ok(())
    }

    /// Undoes all nonces for a single account: restores old values, restores old header,
    /// and cleans up consumed historical entries.
    fn undo_account_nonces(
        tx: &Transaction<'_>,
        account_id_hex: &str,
        nonces: &[u64],
    ) -> Result<(), StoreError> {
        // Undo each nonce in reverse order: restore old values, delete new entries
        for &nonce in nonces {
            let nonce_val = u64_to_value(nonce);
            Self::restore_old_values_for_nonce(tx, account_id_hex, &nonce_val)?;
        }

        // Restore old header from the earliest discarded nonce
        let min_nonce = *nonces.last().unwrap();
        let min_nonce_val = u64_to_value(min_nonce);

        let old_header_exists: bool = tx
            .query_row(
                "SELECT COUNT(*) FROM historical_account_headers \
                 WHERE id = ? AND replaced_at_nonce = ?",
                params![account_id_hex, &min_nonce_val],
                |row| row.get::<_, i64>(0),
            )
            .into_store_error()?
            > 0;

        if old_header_exists {
            tx.execute(
                "INSERT OR REPLACE INTO latest_account_headers \
                 (id, account_commitment, code_commitment, storage_commitment, \
                  vault_root, nonce, account_seed, locked) \
                 SELECT id, account_commitment, code_commitment, storage_commitment, \
                        vault_root, nonce, account_seed, locked \
                 FROM historical_account_headers \
                 WHERE id = ? AND replaced_at_nonce = ?",
                params![account_id_hex, &min_nonce_val],
            )
            .into_store_error()?;
        } else {
            // No previous state — delete the account entirely
            for table in [
                "DELETE FROM latest_account_headers WHERE id = ?",
                "DELETE FROM latest_account_storage WHERE account_id = ?",
                "DELETE FROM latest_storage_map_entries WHERE account_id = ?",
                "DELETE FROM latest_account_assets WHERE account_id = ?",
            ] {
                tx.execute(table, params![account_id_hex]).into_store_error()?;
            }
        }

        // Delete all consumed historical entries at the discarded nonces
        let nonce_params = Rc::new(nonces.iter().map(|n| u64_to_value(*n)).collect::<Vec<_>>());
        for table in [
            "historical_account_storage",
            "historical_storage_map_entries",
            "historical_account_assets",
        ] {
            tx.execute(
                &format!(
                    "DELETE FROM {table} WHERE account_id = ? AND replaced_at_nonce IN rarray(?)"
                ),
                params![account_id_hex, nonce_params.clone()],
            )
            .into_store_error()?;
        }
        tx.execute(
            "DELETE FROM historical_account_headers \
             WHERE id = ? AND replaced_at_nonce IN rarray(?)",
            params![account_id_hex, nonce_params],
        )
        .into_store_error()?;

        Ok(())
    }

    /// Restores old values from historical entries for a given nonce,
    /// undoing the changes made when that nonce was written.
    fn restore_old_values_for_nonce(
        tx: &Transaction<'_>,
        account_id_hex: &str,
        nonce_val: &rusqlite::types::Value,
    ) -> Result<(), StoreError> {
        // Restore storage slots with non-NULL old values
        tx.execute(
            "INSERT OR REPLACE INTO latest_account_storage \
             (account_id, slot_name, slot_value, slot_type) \
             SELECT account_id, slot_name, old_slot_value, slot_type \
             FROM historical_account_storage \
             WHERE account_id = ? AND replaced_at_nonce = ? AND old_slot_value IS NOT NULL",
            params![account_id_hex, nonce_val],
        )
        .into_store_error()?;

        // Delete storage slots that were new (NULL old value)
        tx.execute(
            "DELETE FROM latest_account_storage \
             WHERE account_id = ?1 AND slot_name IN (\
                 SELECT slot_name FROM historical_account_storage \
                 WHERE account_id = ?1 AND replaced_at_nonce = ?2 AND old_slot_value IS NULL\
             )",
            params![account_id_hex, nonce_val],
        )
        .into_store_error()?;

        // Restore map entries with non-NULL old values
        tx.execute(
            "INSERT OR REPLACE INTO latest_storage_map_entries \
             (account_id, slot_name, key, value) \
             SELECT account_id, slot_name, key, old_value \
             FROM historical_storage_map_entries \
             WHERE account_id = ? AND replaced_at_nonce = ? AND old_value IS NOT NULL",
            params![account_id_hex, nonce_val],
        )
        .into_store_error()?;

        // Delete map entries that were new (NULL old value)
        tx.execute(
            "DELETE FROM latest_storage_map_entries \
             WHERE account_id = ?1 AND EXISTS (\
                 SELECT 1 FROM historical_storage_map_entries h \
                 WHERE h.account_id = latest_storage_map_entries.account_id \
                   AND h.slot_name = latest_storage_map_entries.slot_name \
                   AND h.key = latest_storage_map_entries.key \
                   AND h.replaced_at_nonce = ?2 AND h.old_value IS NULL\
             )",
            params![account_id_hex, nonce_val],
        )
        .into_store_error()?;

        // Restore assets with non-NULL old values
        tx.execute(
            "INSERT OR REPLACE INTO latest_account_assets \
             (account_id, vault_key, faucet_id_prefix, asset) \
             SELECT account_id, vault_key, faucet_id_prefix, old_asset \
             FROM historical_account_assets \
             WHERE account_id = ? AND replaced_at_nonce = ? AND old_asset IS NOT NULL",
            params![account_id_hex, nonce_val],
        )
        .into_store_error()?;

        // Delete assets that were new (NULL old value)
        tx.execute(
            "DELETE FROM latest_account_assets \
             WHERE account_id = ?1 AND vault_key IN (\
                 SELECT vault_key FROM historical_account_assets \
                 WHERE account_id = ?1 AND replaced_at_nonce = ?2 AND old_asset IS NULL\
             )",
            params![account_id_hex, nonce_val],
        )
        .into_store_error()?;

        Ok(())
    }

    /// Replaces the account state with a completely new one from the network.
    ///
    /// Replaces the account state entirely: archives old state to historical,
    /// clears latest, inserts new state to latest only.
    pub(crate) fn update_account_state(
        tx: &Transaction<'_>,
        smt_forest: &mut AccountSmtForest,
        new_account_state: &Account,
    ) -> Result<(), StoreError> {
        let account_id = new_account_state.id();
        let account_id_hex = account_id.to_hex();
        let nonce_val = u64_to_value(new_account_state.nonce().as_int());

        // Insert and register account state in the SMT forest (handles old root cleanup)
        smt_forest.insert_and_register_account_state(
            account_id,
            new_account_state.vault(),
            new_account_state.storage(),
        )?;

        // Read old header before overwriting
        let old_header = query_latest_account_headers(tx, "id = ?", params![account_id.to_hex()])?
            .into_iter()
            .next()
            .map(|(header, _)| header);

        // Archive all old entries from latest → historical
        tx.execute(
            "INSERT OR REPLACE INTO historical_account_storage \
             (account_id, replaced_at_nonce, slot_name, old_slot_value, slot_type) \
             SELECT account_id, ?, slot_name, slot_value, slot_type \
             FROM latest_account_storage WHERE account_id = ?",
            params![&nonce_val, &account_id_hex],
        )
        .into_store_error()?;
        tx.execute(
            "INSERT OR REPLACE INTO historical_storage_map_entries \
             (account_id, replaced_at_nonce, slot_name, key, old_value) \
             SELECT account_id, ?, slot_name, key, value \
             FROM latest_storage_map_entries WHERE account_id = ?",
            params![&nonce_val, &account_id_hex],
        )
        .into_store_error()?;
        tx.execute(
            "INSERT OR REPLACE INTO historical_account_assets \
             (account_id, replaced_at_nonce, vault_key, faucet_id_prefix, old_asset) \
             SELECT account_id, ?, vault_key, faucet_id_prefix, asset \
             FROM latest_account_assets WHERE account_id = ?",
            params![&nonce_val, &account_id_hex],
        )
        .into_store_error()?;

        // Delete all latest entries for this account
        tx.execute(
            "DELETE FROM latest_account_storage WHERE account_id = ?",
            params![&account_id_hex],
        )
        .into_store_error()?;
        tx.execute(
            "DELETE FROM latest_storage_map_entries WHERE account_id = ?",
            params![&account_id_hex],
        )
        .into_store_error()?;
        tx.execute(
            "DELETE FROM latest_account_assets WHERE account_id = ?",
            params![&account_id_hex],
        )
        .into_store_error()?;

        // Insert all new entries into latest only
        Self::insert_storage_slots(tx, account_id, new_account_state.storage().slots().iter())?;
        Self::insert_assets(tx, account_id, new_account_state.vault().assets())?;

        // Write NULL historical entries for genuinely new entries that didn't exist
        // in the old state (INSERT OR IGNORE skips entries already archived above)
        tx.execute(
            "INSERT OR IGNORE INTO historical_account_storage \
             (account_id, replaced_at_nonce, slot_name, old_slot_value, slot_type) \
             SELECT account_id, ?, slot_name, NULL, slot_type \
             FROM latest_account_storage WHERE account_id = ?",
            params![&nonce_val, &account_id_hex],
        )
        .into_store_error()?;
        tx.execute(
            "INSERT OR IGNORE INTO historical_storage_map_entries \
             (account_id, replaced_at_nonce, slot_name, key, old_value) \
             SELECT account_id, ?, slot_name, key, NULL \
             FROM latest_storage_map_entries WHERE account_id = ?",
            params![&nonce_val, &account_id_hex],
        )
        .into_store_error()?;
        tx.execute(
            "INSERT OR IGNORE INTO historical_account_assets \
             (account_id, replaced_at_nonce, vault_key, faucet_id_prefix, old_asset) \
             SELECT account_id, ?, vault_key, faucet_id_prefix, NULL \
             FROM latest_account_assets WHERE account_id = ?",
            params![&nonce_val, &account_id_hex],
        )
        .into_store_error()?;

        // Insert account header (archives old header to historical)
        Self::insert_account_header(tx, &new_account_state.into(), None, old_header.as_ref())?;

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
        const LOCK_CONDITION: &str = "WHERE id = :account_id AND NOT EXISTS (SELECT 1 FROM historical_account_headers WHERE id = :account_id AND account_commitment = :digest)";
        let account_id_hex = account_id.to_hex();
        let digest_str = mismatched_digest.to_string();
        let params = named_params! {
            ":account_id": account_id_hex,
            ":digest": digest_str
        };

        let query = format!("UPDATE latest_account_headers SET locked = true {LOCK_CONDITION}");
        tx.execute(&query, params).into_store_error()?;

        // Also lock historical rows so that undo_account_state preserves the lock.
        let query = format!("UPDATE historical_account_headers SET locked = true {LOCK_CONDITION}");
        tx.execute(&query, params).into_store_error()?;

        Ok(())
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

    /// Inserts a new account header into the latest table.
    ///
    /// If `old_header` is provided, the old header is archived to the historical table.
    /// For initial inserts (no previous state), pass `None` for `old_header`.
    fn insert_account_header(
        tx: &Transaction<'_>,
        new_header: &AccountHeader,
        account_seed: Option<Word>,
        old_header: Option<&AccountHeader>,
    ) -> Result<(), StoreError> {
        // Archive the old header to historical before overwriting latest.
        if let Some(old) = old_header {
            let old_id = old.id().to_hex();
            let old_code_commitment = old.code_commitment().to_string();
            let old_storage_commitment = old.storage_commitment().to_string();
            let old_vault_root = old.vault_root().to_string();
            let old_nonce = u64_to_value(old.nonce().as_int());
            let old_commitment = old.to_commitment().to_string();
            let replaced_at_nonce = u64_to_value(new_header.nonce().as_int());

            // Read the old seed from latest (if any)
            let old_seed: Option<Vec<u8>> = tx
                .query_row(
                    "SELECT account_seed FROM latest_account_headers WHERE id = ?",
                    params![&old_id],
                    |row| row.get(0),
                )
                .optional()
                .into_store_error()?
                .flatten();

            const HISTORICAL_QUERY: &str = insert_sql!(
                historical_account_headers {
                    id,
                    code_commitment,
                    storage_commitment,
                    vault_root,
                    nonce,
                    account_seed,
                    account_commitment,
                    locked,
                    replaced_at_nonce
                } | REPLACE
            );

            tx.execute(
                HISTORICAL_QUERY,
                params![
                    old_id,
                    old_code_commitment,
                    old_storage_commitment,
                    old_vault_root,
                    old_nonce,
                    old_seed,
                    old_commitment,
                    false,
                    replaced_at_nonce,
                ],
            )
            .into_store_error()?;
        }

        // Write the new header to latest.
        let id = new_header.id().to_hex();
        let code_commitment = new_header.code_commitment().to_string();
        let storage_commitment = new_header.storage_commitment().to_string();
        let vault_root = new_header.vault_root().to_string();
        let nonce = u64_to_value(new_header.nonce().as_int());
        let commitment = new_header.to_commitment().to_string();
        let account_seed = account_seed.map(|seed| seed.to_bytes());

        const LATEST_QUERY: &str = insert_sql!(
            latest_account_headers {
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
            LATEST_QUERY,
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
