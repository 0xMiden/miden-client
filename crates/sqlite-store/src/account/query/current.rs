//! Query helpers for latest/current account state.

use std::collections::BTreeMap;

use miden_client::Word;
use miden_client::account::{AccountId, StorageSlotName, StorageSlotType};
use miden_client::store::StoreError;
use rusqlite::{Connection, params};

use crate::account::query::domain::LatestStorageSlotRow;
use crate::account::query::parse::parse_word;
use crate::sql_error::SqlResultExt;

pub(crate) fn query_latest_storage_values(
    conn: &Connection,
    account_id: AccountId,
) -> Result<BTreeMap<StorageSlotName, (StorageSlotType, Word)>, StoreError> {
    const QUERY: &str =
        "SELECT slot_name, slot_value, slot_type FROM account_storage_latest WHERE account_id = ?";

    conn.prepare_cached(QUERY)
        .into_store_error()?
        .query_map(params![account_id.to_hex()], LatestStorageSlotRow::from_row)
        .into_store_error()?
        .map(|result| result.into_store_error().and_then(LatestStorageSlotRow::parse))
        .collect()
}

pub(crate) fn query_latest_storage_map_roots(
    conn: &Connection,
    account_id: AccountId,
) -> Result<Vec<Word>, StoreError> {
    const QUERY: &str = "
        SELECT slot_value
        FROM account_storage_latest
        WHERE account_id = ?1 AND slot_type = ?2";

    let map_slot_type = StorageSlotType::Map as u8;
    conn.prepare_cached(QUERY)
        .into_store_error()?
        .query_map(params![account_id.to_hex(), map_slot_type], |row| row.get::<_, String>(0))
        .into_store_error()?
        .map(|result| result.into_store_error().and_then(parse_word))
        .collect()
}
