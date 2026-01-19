//! Storage-related database operations for accounts.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::string::{String, ToString};
use std::vec::Vec;

use miden_client::Word;
use miden_client::account::{
    AccountDelta,
    AccountHeader,
    StorageMap,
    StorageSlot,
    StorageSlotContent,
    StorageSlotName,
    StorageSlotType,
};
use miden_client::store::StoreError;
use miden_protocol::crypto::merkle::MerkleError;
use rusqlite::types::Value;
use rusqlite::{Connection, Params, Transaction, params};

use crate::smt_forest::AccountSmtForest;
use crate::sql_error::SqlResultExt;
use crate::{SqliteStore, insert_sql, subst};

impl SqliteStore {
    // STORAGE HELPERS
    // --------------------------------------------------------------------------------------------

    pub(crate) fn insert_storage_slots<'a>(
        tx: &Transaction<'_>,
        commitment: Word,
        account_storage: impl Iterator<Item = &'a StorageSlot>,
    ) -> Result<(), StoreError> {
        for slot in account_storage {
            const QUERY: &str = insert_sql!(
                account_storage {
                    commitment,
                    slot_name,
                    slot_value,
                    slot_type
                } | REPLACE
            );

            tx.execute(
                QUERY,
                params![
                    commitment.to_hex(),
                    slot.name().to_string(),
                    slot.value().to_hex(),
                    slot.slot_type() as u8
                ],
            )
            .into_store_error()?;

            if let StorageSlotContent::Map(map) = slot.content() {
                const MAP_QUERY: &str =
                    insert_sql!(storage_map_entries { root, key, value } | REPLACE);
                for (key, value) in map.entries() {
                    // Insert each entry of the storage map
                    tx.execute(
                        MAP_QUERY,
                        params![map.root().to_hex(), key.to_hex(), value.to_hex()],
                    )
                    .into_store_error()?;
                }
            }
        }

        Ok(())
    }

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
}

// QUERY HELPERS
// ================================================================================================

pub(crate) fn query_storage_slots(
    conn: &Connection,
    where_clause: &str,
    params: impl Params,
) -> Result<BTreeMap<StorageSlotName, StorageSlot>, StoreError> {
    const STORAGE_QUERY: &str = "SELECT slot_name, slot_value, slot_type FROM account_storage";

    let query = format!("{STORAGE_QUERY} WHERE {where_clause}");
    let storage_values = conn
        .prepare(&query)
        .into_store_error()?
        .query_map(params, |row| {
            let slot_name: String = row.get(0)?;
            let value: String = row.get(1)?;
            let slot_type: u8 = row.get(2)?;
            Ok((slot_name, value, slot_type))
        })
        .into_store_error()?
        .map(|result| {
            let (slot_name, value, slot_type) = result.into_store_error()?;
            let slot_name = StorageSlotName::new(slot_name)
                .map_err(|err| StoreError::ParsingError(err.to_string()))?;
            let slot_type = StorageSlotType::try_from(slot_type)
                .map_err(|e| StoreError::ParsingError(e.to_string()))?;
            Ok((slot_name, Word::try_from(value)?, slot_type))
        })
        .collect::<Result<Vec<(StorageSlotName, Word, StorageSlotType)>, StoreError>>()?;

    let possible_roots: Vec<Value> =
        storage_values.iter().map(|(_, value, _)| Value::from(value.to_hex())).collect();

    let mut storage_maps =
        query_storage_maps(conn, "root IN rarray(?)", [Rc::new(possible_roots)])?;

    Ok(storage_values
        .into_iter()
        .map(|(slot_name, value, slot_type)| {
            let slot = match slot_type {
                StorageSlotType::Value => StorageSlot::with_value(slot_name.clone(), value),
                StorageSlotType::Map => StorageSlot::with_map(
                    slot_name.clone(),
                    storage_maps.remove(&value).unwrap_or_default(),
                ),
            };
            (slot_name, slot)
        })
        .collect())
}

pub(crate) fn query_storage_maps(
    conn: &Connection,
    where_clause: &str,
    params: impl Params,
) -> Result<BTreeMap<Word, StorageMap>, StoreError> {
    const STORAGE_MAP_SELECT: &str = "SELECT root, key, value FROM storage_map_entries";
    let query = format!("{STORAGE_MAP_SELECT} WHERE {where_clause}");

    let map_entries = conn
        .prepare(&query)
        .into_store_error()?
        .query_map(params, |row| {
            let root: String = row.get(0)?;
            let key: String = row.get(1)?;
            let value: String = row.get(2)?;

            Ok((root, key, value))
        })
        .into_store_error()?
        .map(|result| {
            let (root, key, value) = result.into_store_error()?;
            Ok((Word::try_from(root)?, Word::try_from(key)?, Word::try_from(value)?))
        })
        .collect::<Result<Vec<(Word, Word, Word)>, StoreError>>()?;

    let mut maps = BTreeMap::new();
    for (root, key, value) in map_entries {
        let map = maps.entry(root).or_insert_with(StorageMap::new);
        map.insert(key, value)?;
    }

    Ok(maps)
}

pub(crate) fn query_storage_values(
    conn: &Connection,
    where_clause: &str,
    params: impl Params,
) -> Result<BTreeMap<StorageSlotName, (StorageSlotType, Word)>, StoreError> {
    const STORAGE_QUERY: &str = "SELECT slot_name, slot_value, slot_type FROM account_storage";

    let query = format!("{STORAGE_QUERY} WHERE {where_clause}");
    conn.prepare(&query)
        .into_store_error()?
        .query_map(params, |row| {
            let slot_name: String = row.get(0)?;
            let value: String = row.get(1)?;
            let slot_type: u8 = row.get(2)?;
            Ok((slot_name, value, slot_type))
        })
        .into_store_error()?
        .map(|result| {
            let (slot_name, value, slot_type) = result.into_store_error()?;
            let slot_name = StorageSlotName::new(slot_name)
                .map_err(|err| StoreError::ParsingError(err.to_string()))?;
            let slot_type = StorageSlotType::try_from(slot_type)
                .map_err(|e| StoreError::ParsingError(e.to_string()))?;
            Ok((slot_name, (slot_type, Word::try_from(value)?)))
        })
        .collect()
}
