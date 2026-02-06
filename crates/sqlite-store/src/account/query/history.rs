//! Query helpers for historical account state.

use std::collections::BTreeMap;

use miden_client::Word;
use miden_client::account::{AccountId, StorageSlotName, StorageSlotType};
use miden_client::store::StoreError;
use rusqlite::{CachedStatement, Connection, params};

use crate::account::query::domain::StorageSlotValueRow;
use crate::account::query::parse::parse_word;
use crate::sql_error::SqlResultExt;
use crate::u64_to_value;

fn query_slot_value_at_or_before_nonce(
    stmt: &mut CachedStatement<'_>,
    account_id_hex: &str,
    slot_name: &StorageSlotName,
    nonce: u64,
) -> Result<Option<(StorageSlotType, Word)>, StoreError> {
    let mut rows = stmt
        .query(params![account_id_hex, slot_name.to_string(), u64_to_value(nonce)])
        .into_store_error()?;
    let Some(row) = rows.next().into_store_error()? else {
        return Ok(None);
    };

    let slot_row = StorageSlotValueRow::from_row(row).into_store_error()?;
    slot_row.parse().map(Some)
}

pub(crate) fn query_storage_values_at_or_before_nonce_for_slots<'a>(
    conn: &Connection,
    account_id: AccountId,
    nonce: u64,
    slot_names: impl Iterator<Item = &'a StorageSlotName>,
) -> Result<BTreeMap<StorageSlotName, (StorageSlotType, Word)>, StoreError> {
    const QUERY: &str = "
        SELECT slot_value, slot_type
        FROM account_storage_deltas
        WHERE account_id = ?1 AND slot_name = ?2 AND nonce <= ?3
        ORDER BY nonce DESC
        LIMIT 1";

    let account_id_hex = account_id.to_hex();
    let mut stmt = conn.prepare_cached(QUERY).into_store_error()?;
    let mut result = BTreeMap::new();

    for slot_name in slot_names {
        if let Some(slot_value) =
            query_slot_value_at_or_before_nonce(&mut stmt, &account_id_hex, slot_name, nonce)?
        {
            result.insert(slot_name.clone(), slot_value);
        }
    }

    Ok(result)
}

pub(crate) fn query_storage_map_roots_at_or_before_nonce(
    conn: &Connection,
    account_id: AccountId,
    nonce: u64,
) -> Result<Vec<Word>, StoreError> {
    const QUERY: &str = "
        SELECT d.slot_value
        FROM account_storage_deltas d
        JOIN (
            SELECT slot_name, MAX(nonce) AS max_nonce
            FROM account_storage_deltas
            WHERE account_id = ?1 AND nonce <= ?2
            GROUP BY slot_name
        ) latest
        ON d.slot_name = latest.slot_name AND d.nonce = latest.max_nonce
        WHERE d.account_id = ?1 AND d.slot_type = ?3";

    let map_slot_type = StorageSlotType::Map as u8;
    conn.prepare_cached(QUERY)
        .into_store_error()?
        .query_map(params![account_id.to_hex(), u64_to_value(nonce), map_slot_type], |row| {
            row.get::<_, String>(0)
        })
        .into_store_error()?
        .map(|result| result.into_store_error().and_then(parse_word))
        .collect()
}
