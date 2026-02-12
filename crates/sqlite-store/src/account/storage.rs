//! Storage-related database operations for accounts.

use std::collections::BTreeMap;
use std::string::ToString;
use std::vec::Vec;

use miden_client::account::{
    AccountDelta,
    AccountId,
    StorageMap,
    StorageSlot,
    StorageSlotContent,
    StorageSlotName,
};
use miden_client::store::StoreError;
use miden_client::{EMPTY_WORD, Word};
use miden_protocol::crypto::merkle::MerkleError;
use rusqlite::{Transaction, params};

use crate::account::helpers::query_storage_maps;
use crate::smt_forest::AccountSmtForest;
use crate::sql_error::SqlResultExt;
use crate::{SqliteStore, insert_sql, subst, u64_to_value};

impl SqliteStore {
    // READER METHODS
    // --------------------------------------------------------------------------------------------

    /// Fetches the relevant storage maps inside the account's storage that will be updated by the
    /// account delta.
    pub(crate) fn get_account_storage_maps_for_delta(
        conn: &rusqlite::Connection,
        account_id: AccountId,
        delta: &AccountDelta,
    ) -> Result<BTreeMap<StorageSlotName, StorageMap>, StoreError> {
        let mut all_maps = query_storage_maps(conn, account_id)?;

        let updated_names: Vec<StorageSlotName> =
            delta.storage().maps().map(|(slot_name, _)| slot_name.clone()).collect();

        // Retain only the maps that are being updated by the delta.
        all_maps.retain(|name, _| updated_names.contains(name));

        Ok(all_maps)
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
    /// For latest tables: updates slot values and replaces all map entries for affected slots.
    /// For historical tables: writes slot values and only the delta's changed map entries.
    pub(crate) fn write_storage_delta(
        tx: &Transaction<'_>,
        account_id: AccountId,
        nonce: u64,
        updated_storage_slots: &BTreeMap<StorageSlotName, StorageSlot>,
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

        for slot in updated_storage_slots.values() {
            let slot_name_str = slot.name().to_string();
            let slot_value_hex = slot.value().to_hex();
            let slot_type_val = slot.slot_type() as u8;

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

            if let StorageSlotContent::Map(map) = slot.content() {
                // Latest: delete old entries and insert the full updated map
                const DELETE_LATEST_MAP: &str =
                    "DELETE FROM latest_storage_map_entries WHERE account_id = ? AND slot_name = ?";
                tx.execute(DELETE_LATEST_MAP, params![&account_id_hex, &slot_name_str])
                    .into_store_error()?;

                for (key, value) in map.entries() {
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
                // behavior of StorageMap::entries() which excludes zero-valued entries.
                if let Some(changed_entries) = delta_map_entries.get(slot.name()) {
                    for (key, value) in changed_entries {
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
        }

        Ok(())
    }

    /// Applies storage delta changes to the account state, updating storage slots and maps.
    ///
    /// All updated storage map entries are validated against the SMT forest to ensure consistency.
    /// If the computed root doesn't match the expected root, an error is returned.
    pub(crate) fn apply_account_storage_delta(
        smt_forest: &mut AccountSmtForest,
        mut updated_storage_maps: BTreeMap<StorageSlotName, StorageMap>,
        delta: &AccountDelta,
    ) -> Result<BTreeMap<StorageSlotName, StorageSlot>, StoreError> {
        // Apply storage delta. This map will contain all updated storage slots, both values and
        // maps. It gets initialized with value type updates which contain the new value and
        // don't depend on previous state.
        let mut updated_storage_slots: BTreeMap<StorageSlotName, StorageSlot> = delta
            .storage()
            .values()
            .map(|(slot_name, slot)| {
                (slot_name.clone(), StorageSlot::with_value(slot_name.clone(), *slot))
            })
            .collect();

        // For storage map deltas, we only updated the keys in the delta, this is why we need the
        // previously retrieved storage maps.
        for (slot_name, map_delta) in delta.storage().maps() {
            let mut map = updated_storage_maps.remove(slot_name).unwrap_or_default();
            let map_root = map.root();
            let entries: Vec<(Word, Word)> =
                map_delta.entries().iter().map(|(key, value)| ((*key).into(), *value)).collect();

            for (key, value) in &entries {
                map.insert(*key, *value)?;
            }

            let expected_root = map.root();
            let new_root = smt_forest.update_storage_map_nodes(map_root, entries.into_iter())?;
            if new_root != expected_root {
                return Err(StoreError::MerkleStoreError(MerkleError::ConflictingRoots {
                    expected_root,
                    actual_root: new_root,
                }));
            }

            updated_storage_slots
                .insert(slot_name.clone(), StorageSlot::with_map(slot_name.clone(), map));
        }

        Ok(updated_storage_slots)
    }
}
