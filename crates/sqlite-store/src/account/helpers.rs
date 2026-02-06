//! Helper functions for account operations.

use std::collections::BTreeMap;
use std::rc::Rc;

use miden_client::account::{
    AccountCode,
    AccountHeader,
    AccountId,
    Address,
    StorageMap,
    StorageSlot,
    StorageSlotName,
    StorageSlotType,
};
use miden_client::asset::Asset;
use miden_client::store::{AccountStatus, StoreError};
use miden_client::{Deserializable, Word};
use rusqlite::types::Value;
use rusqlite::{Connection, Params, params};

use crate::column_value_as_u64;
use crate::sql_error::SqlResultExt;

pub(crate) struct SerializedHeaderData {
    pub id: String,
    pub nonce: u64,
    pub vault_root: String,
    pub storage_commitment: String,
    pub code_commitment: String,
    pub account_seed: Option<Vec<u8>>,
    pub locked: bool,
}

/// Parse an account header from the provided serialized data.
pub(crate) fn parse_accounts(
    serialized_account_parts: SerializedHeaderData,
) -> Result<(AccountHeader, AccountStatus), StoreError> {
    let SerializedHeaderData {
        id,
        nonce,
        vault_root,
        storage_commitment,
        code_commitment,
        account_seed,
        locked,
    } = serialized_account_parts;
    let account_seed = account_seed.map(|seed| Word::read_from_bytes(&seed[..])).transpose()?;

    let status = match (account_seed, locked) {
        (seed, true) => AccountStatus::Locked { seed },
        (Some(seed), _) => AccountStatus::New { seed },
        _ => AccountStatus::Tracked,
    };

    Ok((
        AccountHeader::new(
            AccountId::from_hex(&id).expect("Conversion from stored AccountID should not panic"),
            miden_client::Felt::new(nonce),
            Word::try_from(&vault_root)?,
            Word::try_from(&storage_commitment)?,
            Word::try_from(&code_commitment)?,
        ),
        status,
    ))
}

pub(crate) fn query_account_headers(
    conn: &Connection,
    table_name: &str,
    where_clause: &str,
    params: impl Params,
) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
    const SELECT_QUERY: &str = "SELECT id, nonce, vault_root, storage_commitment, code_commitment, account_seed, locked FROM";
    let query = format!("{SELECT_QUERY} {table_name} WHERE {where_clause}");
    conn.prepare(&query)
        .into_store_error()?
        .query_map(params, |row| {
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

pub(crate) fn query_account_code(
    conn: &Connection,
    commitment: Word,
) -> Result<Option<AccountCode>, StoreError> {
    // TODO: this function will probably be refactored to receive more complex where clauses and
    // return multiple mast forests
    const CODE_QUERY: &str = "SELECT code FROM account_code WHERE commitment = ?";

    conn.prepare_cached(CODE_QUERY)
        .into_store_error()?
        .query_map(params![commitment.to_hex()], |row| {
            let code: Vec<u8> = row.get(0)?;
            Ok(code)
        })
        .into_store_error()?
        .map(|result| {
            let bytes: Vec<u8> = result.into_store_error()?;
            Ok(AccountCode::from_bytes(&bytes)?)
        })
        .next()
        .transpose()
}

pub(crate) fn query_account_addresses(
    conn: &Connection,
    account_id: AccountId,
) -> Result<Vec<Address>, StoreError> {
    const ADDRESS_QUERY: &str = "SELECT address FROM addresses WHERE account_id = ?";

    conn.prepare_cached(ADDRESS_QUERY)
        .into_store_error()?
        .query_map(params![account_id.to_hex()], |row| {
            let address: Vec<u8> = row.get(0)?;
            Ok(address)
        })
        .into_store_error()?
        .map(|result| {
            let serialized_address = result.into_store_error()?;
            let address = Address::read_from_bytes(&serialized_address)?;
            Ok(address)
        })
        .collect::<Result<Vec<Address>, StoreError>>()
}

pub(crate) fn query_vault_assets(
    conn: &Connection,
    where_clause: &str,
    params: impl Params,
) -> Result<Vec<Asset>, StoreError> {
    const VAULT_QUERY: &str = "SELECT asset FROM account_assets";

    let query = format!("{VAULT_QUERY} WHERE {where_clause}");
    conn.prepare(&query)
        .into_store_error()?
        .query_map(params, |row| {
            let asset: String = row.get(0)?;
            Ok(asset)
        })
        .into_store_error()?
        .map(|result| {
            let asset_str: String = result.into_store_error()?;
            let word = Word::try_from(asset_str)?;
            Ok(Asset::try_from(word)?)
        })
        .collect::<Result<Vec<Asset>, StoreError>>()
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

pub(crate) fn build_storage_slots_from_values(
    conn: &Connection,
    storage_values: BTreeMap<StorageSlotName, (StorageSlotType, Word)>,
) -> Result<BTreeMap<StorageSlotName, StorageSlot>, StoreError> {
    let map_roots: Vec<Value> = storage_values
        .values()
        .filter(|(slot_type, _)| *slot_type == StorageSlotType::Map)
        .map(|(_, value)| Value::from(value.to_hex()))
        .collect();
    let mut storage_maps = if map_roots.is_empty() {
        BTreeMap::new()
    } else {
        query_storage_maps(conn, "root IN rarray(?)", [Rc::new(map_roots)])?
    };

    Ok(storage_values
        .into_iter()
        .map(|(slot_name, (slot_type, value))| {
            let key = slot_name.clone();
            let slot = match slot_type {
                StorageSlotType::Value => StorageSlot::with_value(slot_name, value),
                StorageSlotType::Map => StorageSlot::with_map(
                    slot_name,
                    storage_maps.remove(&value).unwrap_or_else(StorageMap::new),
                ),
            };
            (key, slot)
        })
        .collect())
}
