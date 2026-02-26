//! Storage-related database operations for accounts.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::string::ToString;
use std::vec::Vec;

use miden_client::Word;
use miden_client::account::{
    AccountDelta,
    AccountHeader,
    StorageMap,
    StorageSlot,
    StorageSlotContent,
    StorageSlotName,
};
use miden_client::store::StoreError;
use miden_protocol::crypto::merkle::MerkleError;
use rusqlite::types::Value;
use rusqlite::{Connection, Transaction, params};

use crate::account::helpers::query_storage_slots;
use crate::smt_forest::AccountSmtForest;
use crate::sql_error::SqlResultExt;
use crate::{SqliteStore, insert_sql, subst};

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
        let updated_map_names = delta
            .storage()
            .maps()
            .map(|(slot_name, _)| Value::Text(slot_name.to_string()))
            .collect::<Vec<Value>>();

        query_storage_slots(
            conn,
            "commitment = ? AND slot_name IN rarray(?)",
            params![header.storage_commitment().to_hex(), Rc::new(updated_map_names)],
        )?
        .into_iter()
        .map(|slot| {
            let (slot_name, content) = slot.into_parts();
            let StorageSlotContent::Map(map) = content else {
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

    /// Inserts storage slots into the database for a given storage commitment.
    pub(crate) fn insert_storage_slots<'a>(
        tx: &Transaction<'_>,
        commitment: Word,
        account_storage: impl Iterator<Item = &'a StorageSlot>,
    ) -> Result<(), StoreError> {
        const SLOT_QUERY: &str = insert_sql!(
            account_storage {
                commitment,
                slot_name,
                slot_value,
                slot_type
            } | REPLACE
        );
        const MAP_ENTRY_QUERY: &str =
            insert_sql!(storage_map_entries { root, key, value } | REPLACE);

        let mut slot_stmt = tx.prepare_cached(SLOT_QUERY).into_store_error()?;
        let mut map_entry_stmt = tx.prepare_cached(MAP_ENTRY_QUERY).into_store_error()?;
        let commitment_hex = commitment.to_hex();

        for slot in account_storage {
            slot_stmt
                .execute(params![
                    &commitment_hex,
                    slot.name().to_string(),
                    slot.value().to_hex(),
                    slot.slot_type() as u8
                ])
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
    ///
    /// Changed map roots in `account_roots` are replaced in place with their new values.
    pub(crate) fn apply_account_storage_delta(
        smt_forest: &mut AccountSmtForest,
        account_roots: &mut [Word],
        mut updated_storage_maps: BTreeMap<StorageSlotName, StorageMap>,
        delta: &AccountDelta,
    ) -> Result<Vec<StorageSlot>, StoreError> {
        // Apply storage delta. This map will contain all updated storage slots, both values and
        // maps. It gets initialized with value type updates which contain the new value and
        // don't depend on previous state.
        let mut updated_storage_slots: Vec<StorageSlot> = delta
            .storage()
            .values()
            .map(|(slot_name, slot)| StorageSlot::with_value(slot_name.clone(), *slot))
            .collect();

        // For storage map deltas, we only updated the keys in the delta, this is why we need the
        // previously retrieved storage maps.
        for (slot_name, map_delta) in delta.storage().maps() {
            let mut map = updated_storage_maps.remove(slot_name).unwrap_or_default();
            let old_root = map.root();
            let entries: Vec<(Word, Word)> =
                map_delta.entries().iter().map(|(key, value)| ((*key).into(), *value)).collect();

            for (key, value) in &entries {
                map.insert(*key, *value)?;
            }

            let expected_root = map.root();
            let actual_root = smt_forest.update_storage_map_nodes(old_root, entries.into_iter())?;
            if actual_root != expected_root {
                return Err(StoreError::MerkleStoreError(MerkleError::ConflictingRoots {
                    expected_root,
                    actual_root,
                }));
            }

            let root = account_roots
                .iter_mut()
                .find(|r| **r == old_root)
                .ok_or(StoreError::AccountStorageRootNotFound(old_root))?;
            *root = expected_root;

            updated_storage_slots.push(StorageSlot::with_map(slot_name.clone(), map));
        }

        Ok(updated_storage_slots)
    }
}
