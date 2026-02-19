//! Helper functions for account operations.

use std::collections::BTreeMap;

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
use miden_client::store::{AccountStatus, AccountStorageFilter, StoreError};
use miden_client::{Deserializable, Word};
use rusqlite::{Connection, Params, params, params_from_iter};

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

pub(crate) fn query_latest_account_headers(
    conn: &Connection,
    where_clause: &str,
    params: impl Params,
) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
    query_account_headers_from_table(conn, "latest_account_headers", where_clause, params)
}

pub(crate) fn query_historical_account_headers(
    conn: &Connection,
    where_clause: &str,
    params: impl Params,
) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
    query_account_headers_from_table(conn, "historical_account_headers", where_clause, params)
}

fn query_account_headers_from_table(
    conn: &Connection,
    table: &str,
    where_clause: &str,
    params: impl Params,
) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
    let query = format!(
        "SELECT id, nonce, vault_root, storage_commitment, code_commitment, account_seed, locked \
         FROM {table} WHERE {where_clause}"
    );
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

// TODO: this function will probably be refactored to receive more complex where clauses and
// return multiple mast forests
pub(crate) fn query_account_code(
    conn: &Connection,
    commitment: Word,
) -> Result<Option<AccountCode>, StoreError> {
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
    account_id: AccountId,
) -> Result<Vec<Asset>, StoreError> {
    const VAULT_QUERY: &str = "SELECT asset FROM latest_account_assets WHERE account_id = ?";

    conn.prepare(VAULT_QUERY)
        .into_store_error()?
        .query_map(params![account_id.to_hex()], |row| {
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

pub(crate) fn query_storage_slots(
    conn: &Connection,
    account_id: AccountId,
    filter: &AccountStorageFilter,
) -> Result<BTreeMap<StorageSlotName, StorageSlot>, StoreError> {
    let account_id_hex = account_id.to_hex();

    // Build storage values query with filter pushed to SQL
    let base_query =
        "SELECT slot_name, slot_value, slot_type FROM latest_account_storage WHERE account_id = ?1";
    let mut values_params: Vec<String> = vec![account_id_hex];
    let query = match filter {
        AccountStorageFilter::All => base_query.to_string(),
        AccountStorageFilter::SlotName(name) => {
            values_params.push(name.to_string());
            format!("{base_query} AND slot_name = ?2")
        },
        AccountStorageFilter::Root(root) => {
            values_params.push(root.to_hex());
            format!("{base_query} AND slot_value = ?2")
        },
    };

    let mut stmt = conn.prepare(&query).into_store_error()?;
    let storage_values = stmt
        .query_map(params_from_iter(values_params.iter()), |row| {
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

    // For SlotName filter, also restrict map entries query to avoid loading unneeded maps
    let map_filter = match filter {
        AccountStorageFilter::SlotName(name) => Some(name.to_string()),
        _ => None,
    };

    let has_map_slots = storage_values.iter().any(|(_, _, t)| *t == StorageSlotType::Map);
    let mut storage_maps = if has_map_slots {
        query_storage_maps(conn, account_id, map_filter.as_deref())?
    } else {
        BTreeMap::new()
    };

    Ok(storage_values
        .into_iter()
        .map(|(slot_name, value, slot_type)| {
            let key = slot_name.clone();
            let slot = match slot_type {
                StorageSlotType::Value => StorageSlot::with_value(slot_name, value),
                StorageSlotType::Map => StorageSlot::with_map(
                    slot_name.clone(),
                    storage_maps.remove(&slot_name).unwrap_or(StorageMap::new()),
                ),
            };
            (key, slot)
        })
        .collect())
}

pub(crate) fn query_storage_maps(
    conn: &Connection,
    account_id: AccountId,
    slot_name_filter: Option<&str>,
) -> Result<BTreeMap<StorageSlotName, StorageMap>, StoreError> {
    let account_id_hex = account_id.to_hex();
    let base_query =
        "SELECT slot_name, key, value FROM latest_storage_map_entries WHERE account_id = ?1";
    let mut map_params: Vec<String> = vec![account_id_hex];
    let query = match slot_name_filter {
        Some(name) => {
            map_params.push(name.to_string());
            format!("{base_query} AND slot_name = ?2")
        },
        None => base_query.to_string(),
    };

    let mut stmt = conn.prepare(&query).into_store_error()?;
    let map_entries = stmt
        .query_map(params_from_iter(map_params.iter()), |row| {
            let slot_name: String = row.get(0)?;
            let key: String = row.get(1)?;
            let value: String = row.get(2)?;

            Ok((slot_name, key, value))
        })
        .into_store_error()?
        .map(|result| {
            let (slot_name, key, value) = result.into_store_error()?;
            let slot_name = StorageSlotName::new(slot_name)
                .map_err(|err| StoreError::ParsingError(err.to_string()))?;
            Ok((slot_name, Word::try_from(key)?, Word::try_from(value)?))
        })
        .collect::<Result<Vec<(StorageSlotName, Word, Word)>, StoreError>>()?;

    let mut maps = BTreeMap::new();
    for (slot_name, key, value) in map_entries {
        let map = maps.entry(slot_name).or_insert_with(StorageMap::new);
        map.insert(key, value)?;
    }

    Ok(maps)
}

pub(crate) fn query_storage_values(
    conn: &Connection,
    account_id: AccountId,
) -> Result<BTreeMap<StorageSlotName, (StorageSlotType, Word)>, StoreError> {
    const STORAGE_QUERY: &str =
        "SELECT slot_name, slot_value, slot_type FROM latest_account_storage WHERE account_id = ?";

    conn.prepare(STORAGE_QUERY)
        .into_store_error()?
        .query_map(params![account_id.to_hex()], |row| {
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
