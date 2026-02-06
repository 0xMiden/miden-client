//! Storage-related database operations for accounts.

use std::collections::BTreeMap;
use std::string::ToString;
use std::vec::Vec;

use miden_client::Word;
use miden_client::account::{
    AccountDelta,
    AccountHeader,
    AccountId,
    StorageMap,
    StorageSlot,
    StorageSlotContent,
    StorageSlotName,
};
use miden_client::store::StoreError;
use miden_protocol::crypto::merkle::MerkleError;
use rusqlite::{Connection, Transaction, params};

use crate::account::helpers::{
    build_storage_slots_from_values,
    query_storage_values_at_or_before_nonce_for_slots,
};
use crate::smt_forest::AccountSmtForest;
use crate::sql_error::SqlResultExt;
use crate::{SqliteStore, insert_sql, subst, u64_to_value};

impl SqliteStore {
    // READER METHODS
    // --------------------------------------------------------------------------------------------

    /// Fetches the relevant storage maps inside the account's storage that will be updated by the
    /// account delta.
    pub(crate) fn get_account_storage_maps_for_delta(
        conn: &Connection,
        header: &AccountHeader,
        delta: &AccountDelta,
    ) -> Result<BTreeMap<StorageSlotName, StorageMap>, StoreError> {
        let updated_map_names: Vec<StorageSlotName> =
            delta.storage().maps().map(|(slot_name, _)| slot_name.clone()).collect();
        if updated_map_names.is_empty() {
            return Ok(BTreeMap::new());
        }

        let storage_values = query_storage_values_at_or_before_nonce_for_slots(
            conn,
            header.id(),
            header.nonce().as_int(),
            updated_map_names.iter(),
        )?;
        let slots = build_storage_slots_from_values(conn, storage_values)?;

        slots
            .into_iter()
            .map(|(slot_name, slot)| {
                let StorageSlotContent::Map(map) = slot.into_parts().1 else {
                    return Err(StoreError::AccountError(
                        miden_client::AccountError::StorageSlotNotMap(slot_name),
                    ));
                };

                Ok((slot_name, map))
            })
            .collect()
    }

    // MUTATOR/WRITER METHODS
    // --------------------------------------------------------------------------------------------

    /// Inserts storage slot updates for an account at a specific nonce and refreshes the
    /// materialized latest rows used by read paths.
    pub(crate) fn upsert_account_storage_slot_updates<'a>(
        tx: &Transaction<'_>,
        account_id: AccountId,
        nonce: u64,
        account_storage: impl Iterator<Item = &'a StorageSlot>,
    ) -> Result<(), StoreError> {
        const DELTA_QUERY: &str = insert_sql!(
            account_storage_deltas {
                account_id,
                nonce,
                slot_name,
                slot_value,
                slot_type
            } | REPLACE
        );
        const LATEST_QUERY: &str = "
            INSERT INTO account_storage_latest (
                account_id,
                slot_name,
                slot_value,
                slot_type,
                nonce
            ) VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(account_id, slot_name)
            DO UPDATE SET
                slot_value = excluded.slot_value,
                slot_type = excluded.slot_type,
                nonce = excluded.nonce
            WHERE excluded.nonce >= account_storage_latest.nonce";
        const MAP_ENTRY_QUERY: &str =
            insert_sql!(storage_map_entries { root, key, value } | REPLACE);

        let mut delta_stmt = tx.prepare_cached(DELTA_QUERY).into_store_error()?;
        let mut latest_stmt = tx.prepare_cached(LATEST_QUERY).into_store_error()?;
        let mut map_entry_stmt = tx.prepare_cached(MAP_ENTRY_QUERY).into_store_error()?;

        let account_id_hex = account_id.to_hex();
        let nonce_value = u64_to_value(nonce);

        for slot in account_storage {
            let slot_name = slot.name().to_string();
            let slot_value = slot.value().to_hex();
            let slot_type = slot.slot_type() as u8;

            delta_stmt
                .execute(params![&account_id_hex, &nonce_value, &slot_name, &slot_value, slot_type])
                .into_store_error()?;

            latest_stmt
                .execute(params![&account_id_hex, &slot_name, &slot_value, slot_type, &nonce_value])
                .into_store_error()?;

            if let StorageSlotContent::Map(map) = slot.content() {
                let root_hex = map.root().to_hex();
                for (key, value) in map.entries() {
                    map_entry_stmt
                        .execute(params![&root_hex, key.to_hex(), value.to_hex()])
                        .into_store_error()?;
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
