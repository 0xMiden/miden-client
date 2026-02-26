//! Storage-related database operations for accounts.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::string::ToString;
use std::vec::Vec;

use miden_client::account::{
    AccountDelta,
    AccountId,
    StorageMap,
    StorageSlot,
    StorageSlotContent,
    StorageSlotName,
    StorageSlotType,
};
use miden_client::store::StoreError;
use miden_client::{EMPTY_WORD, Word};
use rusqlite::types::Value;
use rusqlite::{Transaction, params};

use crate::smt_forest::AccountSmtForest;
use crate::sql_error::SqlResultExt;
use crate::{SqliteStore, insert_sql, subst, u64_to_value};

impl SqliteStore {
    // READER METHODS
    // --------------------------------------------------------------------------------------------

    /// Fetches the current root values for storage maps that will be updated by the account delta.
    ///
    /// Only queries the slot values (roots) from the latest storage table, avoiding the need to
    /// load full storage map entries into memory. The `AccountSmtForest` handles the actual
    /// Merkle tree operations.
    pub(crate) fn get_storage_map_roots_for_delta(
        conn: &rusqlite::Connection,
        account_id: AccountId,
        delta: &AccountDelta,
    ) -> Result<BTreeMap<StorageSlotName, Word>, StoreError> {
        let map_slot_names: Vec<Value> = delta
            .storage()
            .maps()
            .map(|(slot_name, _)| Value::Text(slot_name.to_string()))
            .collect();

        if map_slot_names.is_empty() {
            return Ok(BTreeMap::new());
        }

        const QUERY: &str = "SELECT slot_name, slot_value FROM latest_account_storage \
                             WHERE account_id = ? AND slot_name IN rarray(?)";

        conn.prepare(QUERY)
            .into_store_error()?
            .query_map(params![account_id.to_hex(), Rc::new(map_slot_names)], |row| {
                let name: String = row.get(0)?;
                let value: String = row.get(1)?;
                Ok((name, value))
            })
            .into_store_error()?
            .map(|result| {
                let (name, value) = result.into_store_error()?;
                let slot_name = StorageSlotName::new(name)
                    .map_err(|err| StoreError::ParsingError(err.to_string()))?;
                Ok((slot_name, Word::try_from(value)?))
            })
            .collect()
    }

    // MUTATOR/WRITER METHODS
    // --------------------------------------------------------------------------------------------

    /// Inserts storage slots into both latest and historical tables for a given
    /// (`account_id`, `nonce`).
    pub(crate) fn insert_storage_slots<'a>(
        tx: &Transaction<'_>,
        account_id: AccountId,
        nonce: u64,
        account_storage: impl Iterator<Item = &'a StorageSlot>,
    ) -> Result<(), StoreError> {
        const LATEST_SLOT_QUERY: &str = insert_sql!(
            latest_account_storage {
                account_id,
                slot_name,
                slot_value,
                slot_type
            } | REPLACE
        );
        const HISTORICAL_SLOT_QUERY: &str = insert_sql!(
            historical_account_storage {
                account_id,
                nonce,
                slot_name,
                slot_value,
                slot_type
            } | REPLACE
        );
        const LATEST_MAP_ENTRY_QUERY: &str =
            insert_sql!(latest_storage_map_entries { account_id, slot_name, key, value } | REPLACE);
        const HISTORICAL_MAP_ENTRY_QUERY: &str = insert_sql!(
            historical_storage_map_entries { account_id, nonce, slot_name, key, value } | REPLACE
        );

        let mut latest_slot_stmt = tx.prepare_cached(LATEST_SLOT_QUERY).into_store_error()?;
        let mut hist_slot_stmt = tx.prepare_cached(HISTORICAL_SLOT_QUERY).into_store_error()?;
        let mut latest_map_stmt = tx.prepare_cached(LATEST_MAP_ENTRY_QUERY).into_store_error()?;
        let mut hist_map_stmt = tx.prepare_cached(HISTORICAL_MAP_ENTRY_QUERY).into_store_error()?;
        let account_id_hex = account_id.to_hex();
        let nonce_val = u64_to_value(nonce);

        for slot in account_storage {
            let slot_name_str = slot.name().to_string();
            let slot_value_hex = slot.value().to_hex();
            let slot_type_val = slot.slot_type() as u8;

            latest_slot_stmt
                .execute(params![&account_id_hex, &slot_name_str, &slot_value_hex, slot_type_val])
                .into_store_error()?;

            hist_slot_stmt
                .execute(params![
                    &account_id_hex,
                    &nonce_val,
                    &slot_name_str,
                    &slot_value_hex,
                    slot_type_val,
                ])
                .into_store_error()?;

            if let StorageSlotContent::Map(map) = slot.content() {
                for (key, value) in map.entries() {
                    latest_map_stmt
                        .execute(params![
                            &account_id_hex,
                            &slot_name_str,
                            key.to_hex(),
                            value.to_hex(),
                        ])
                        .into_store_error()?;

                    hist_map_stmt
                        .execute(params![
                            &account_id_hex,
                            &nonce_val,
                            &slot_name_str,
                            key.to_hex(),
                            value.to_hex(),
                        ])
                        .into_store_error()?;
                }
            }
        }

        Ok(())
    }

    /// Writes only the changed storage slots to historical and updates latest via INSERT OR
    /// REPLACE.
    ///
    /// For both latest and historical tables: writes slot values and only the delta's changed
    /// map entries. Removed map entries (`EMPTY_WORD`) are deleted from latest and stored as NULL
    /// in historical.
    pub(crate) fn write_storage_delta(
        tx: &Transaction<'_>,
        account_id: AccountId,
        nonce: u64,
        updated_slots: &BTreeMap<StorageSlotName, (Word, StorageSlotType)>,
        delta: &AccountDelta,
    ) -> Result<(), StoreError> {
        const LATEST_SLOT_QUERY: &str = insert_sql!(
            latest_account_storage {
                account_id,
                slot_name,
                slot_value,
                slot_type
            } | REPLACE
        );
        const HISTORICAL_SLOT_QUERY: &str = insert_sql!(
            historical_account_storage {
                account_id,
                nonce,
                slot_name,
                slot_value,
                slot_type
            } | REPLACE
        );
        const LATEST_MAP_ENTRY_QUERY: &str =
            insert_sql!(latest_storage_map_entries { account_id, slot_name, key, value } | REPLACE);
        const HISTORICAL_MAP_ENTRY_QUERY: &str = insert_sql!(
            historical_storage_map_entries { account_id, nonce, slot_name, key, value } | REPLACE
        );
        const DELETE_LATEST_MAP_ENTRY: &str = "DELETE FROM latest_storage_map_entries WHERE account_id = ? AND slot_name = ? AND key = ?";

        let mut latest_slot_stmt = tx.prepare_cached(LATEST_SLOT_QUERY).into_store_error()?;
        let mut hist_slot_stmt = tx.prepare_cached(HISTORICAL_SLOT_QUERY).into_store_error()?;
        let mut latest_map_stmt = tx.prepare_cached(LATEST_MAP_ENTRY_QUERY).into_store_error()?;
        let mut hist_map_stmt = tx.prepare_cached(HISTORICAL_MAP_ENTRY_QUERY).into_store_error()?;
        let account_id_hex = account_id.to_hex();
        let nonce_val = u64_to_value(nonce);

        // Collect the delta's changed map entries for efficient lookup
        let delta_map_entries: BTreeMap<&StorageSlotName, Vec<(Word, Word)>> = delta
            .storage()
            .maps()
            .map(|(slot_name, map_delta)| {
                let entries: Vec<(Word, Word)> = map_delta
                    .entries()
                    .iter()
                    .map(|(key, value)| ((*key).into(), *value))
                    .collect();
                (slot_name, entries)
            })
            .collect();

        for (slot_name, (value, slot_type)) in updated_slots {
            let slot_name_str = slot_name.to_string();
            let slot_value_hex = value.to_hex();
            let slot_type_val = *slot_type as u8;

            // Update latest slot
            latest_slot_stmt
                .execute(params![&account_id_hex, &slot_name_str, &slot_value_hex, slot_type_val])
                .into_store_error()?;

            // Write slot to historical
            hist_slot_stmt
                .execute(params![
                    &account_id_hex,
                    &nonce_val,
                    &slot_name_str,
                    &slot_value_hex,
                    slot_type_val,
                ])
                .into_store_error()?;

            if let Some(changed_entries) = delta_map_entries.get(slot_name) {
                for (key, value) in changed_entries {
                    // Latest: write only the delta entries (REPLACE for updates, DELETE
                    // for removals)
                    if *value == EMPTY_WORD {
                        tx.execute(
                            DELETE_LATEST_MAP_ENTRY,
                            params![&account_id_hex, &slot_name_str, key.to_hex()],
                        )
                        .into_store_error()?;
                    } else {
                        latest_map_stmt
                            .execute(params![
                                &account_id_hex,
                                &slot_name_str,
                                key.to_hex(),
                                value.to_hex(),
                            ])
                            .into_store_error()?;
                    }

                    // Historical: write ONLY the delta's changed entries.
                    // Use NULL as tombstone for removed entries (EMPTY_WORD) so that
                    // rebuild_latest_for_account can filter them out, matching the
                    // behavior of StorageMap::entries() which excludes zero-valued
                    // entries.
                    let value_param: Option<String> = if *value == EMPTY_WORD {
                        None
                    } else {
                        Some(value.to_hex())
                    };
                    hist_map_stmt
                        .execute(params![
                            &account_id_hex,
                            &nonce_val,
                            &slot_name_str,
                            key.to_hex(),
                            value_param,
                        ])
                        .into_store_error()?;
                }
            }
        }

        Ok(())
    }

    /// Applies storage delta changes to the account state, computing new roots via the SMT forest.
    ///
    /// Value-type slot updates are taken directly from the delta. For map-type slots, the old
    /// root is used to update the SMT forest with the delta entries, producing the new root.
    /// Full storage maps are never loaded into memory — the `AccountSmtForest` handles all
    /// Merkle tree operations.
    pub(crate) fn apply_account_storage_delta(
        smt_forest: &mut AccountSmtForest,
        old_map_roots: &BTreeMap<StorageSlotName, Word>,
        delta: &AccountDelta,
    ) -> Result<BTreeMap<StorageSlotName, (Word, StorageSlotType)>, StoreError> {
        let mut updated_slots: BTreeMap<StorageSlotName, (Word, StorageSlotType)> = delta
            .storage()
            .values()
            .map(|(slot_name, value)| (slot_name.clone(), (*value, StorageSlotType::Value)))
            .collect();

        let default_map_root = StorageMap::default().root();

        for (slot_name, map_delta) in delta.storage().maps() {
            let old_root = old_map_roots.get(slot_name).copied().unwrap_or(default_map_root);
            let entries: Vec<(Word, Word)> =
                map_delta.entries().iter().map(|(key, value)| ((*key).into(), *value)).collect();

            let new_root = smt_forest.update_storage_map_nodes(old_root, entries.into_iter())?;
            updated_slots.insert(slot_name.clone(), (new_root, StorageSlotType::Map));
        }

        Ok(updated_slots)
    }
}
