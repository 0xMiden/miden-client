#![allow(clippy::items_after_statements)]

use miden_client::account::{AccountCode, AccountHeader, AccountId};
use miden_client::note::BlockNumber;
use miden_client::store::{StoreError, WatchedAccountRecord};
use miden_client::utils::{Deserializable, Serializable};
use miden_protocol::account::AccountStorageHeader;
use rusqlite::{Connection, Transaction, params};

use crate::SqliteStore;
use crate::column_value_as_u64;
use crate::sql_error::SqlResultExt;

impl SqliteStore {
    pub(crate) fn get_watched_accounts(
        conn: &mut Connection,
    ) -> Result<Vec<WatchedAccountRecord>, StoreError> {
        const QUERY: &str = "SELECT w.id, w.account_header, w.code_commitment, w.storage_header, \
                             w.last_synced_block, c.code \
                             FROM watched_accounts w \
                             JOIN account_code c ON w.code_commitment = c.commitment";

        conn.prepare_cached(QUERY)
            .into_store_error()?
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let header_bytes: Vec<u8> = row.get(1)?;
                let storage_header_bytes: Vec<u8> = row.get(3)?;
                let last_synced_block: u64 = column_value_as_u64(row, 4)?;
                let code_bytes: Vec<u8> = row.get(5)?;

                Ok((id, header_bytes, storage_header_bytes, last_synced_block, code_bytes))
            })
            .expect("no binding parameters used in query")
            .map(|result| {
                let (id, header_bytes, storage_header_bytes, last_synced_block, code_bytes) =
                    result.into_store_error()?;

                let account_id = AccountId::from_hex(&id)
                    .map_err(|e| StoreError::ParsingError(e.to_string()))?;
                let header = AccountHeader::read_from_bytes(&header_bytes)?;
                let code = AccountCode::from_bytes(&code_bytes)?;
                let storage_header = AccountStorageHeader::read_from_bytes(&storage_header_bytes)?;

                Ok(WatchedAccountRecord {
                    account_id,
                    header,
                    code,
                    storage_header,
                    last_synced_block: BlockNumber::from(
                        u32::try_from(last_synced_block)
                            .expect("block number fits in u32"),
                    ),
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()
    }

    pub(crate) fn get_watched_account(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<Option<WatchedAccountRecord>, StoreError> {
        const QUERY: &str = "SELECT w.id, w.account_header, w.code_commitment, w.storage_header, \
                             w.last_synced_block, c.code \
                             FROM watched_accounts w \
                             JOIN account_code c ON w.code_commitment = c.commitment \
                             WHERE w.id = ?";

        conn.prepare_cached(QUERY)
            .into_store_error()?
            .query_map(params![account_id.to_hex()], |row| {
                let id: String = row.get(0)?;
                let header_bytes: Vec<u8> = row.get(1)?;
                let storage_header_bytes: Vec<u8> = row.get(3)?;
                let last_synced_block: u64 = column_value_as_u64(row, 4)?;
                let code_bytes: Vec<u8> = row.get(5)?;

                Ok((id, header_bytes, storage_header_bytes, last_synced_block, code_bytes))
            })
            .into_store_error()?
            .map(|result| {
                let (id, header_bytes, storage_header_bytes, last_synced_block, code_bytes) =
                    result.into_store_error()?;

                let account_id = AccountId::from_hex(&id)
                    .map_err(|e| StoreError::ParsingError(e.to_string()))?;
                let header = AccountHeader::read_from_bytes(&header_bytes)?;
                let code = AccountCode::from_bytes(&code_bytes)?;
                let storage_header = AccountStorageHeader::read_from_bytes(&storage_header_bytes)?;

                Ok(WatchedAccountRecord {
                    account_id,
                    header,
                    code,
                    storage_header,
                    last_synced_block: BlockNumber::from(
                        u32::try_from(last_synced_block)
                            .expect("block number fits in u32"),
                    ),
                })
            })
            .next()
            .transpose()
    }

    pub(crate) fn upsert_watched_account(
        conn: &mut Connection,
        record: &WatchedAccountRecord,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;
        upsert_watched_account_tx(&tx, record)?;
        tx.commit().into_store_error()
    }

    pub(crate) fn remove_watched_account(
        conn: &mut Connection,
        account_id: AccountId,
    ) -> Result<(), StoreError> {
        const QUERY: &str = "DELETE FROM watched_accounts WHERE id = ?";

        conn.execute(QUERY, params![account_id.to_hex()])
            .into_store_error()?;

        Ok(())
    }
}

/// Upserts a watched account within an existing transaction.
pub(crate) fn upsert_watched_account_tx(
    tx: &Transaction<'_>,
    record: &WatchedAccountRecord,
) -> Result<(), StoreError> {
    // Ensure the account code is stored
    SqliteStore::insert_account_code(tx, &record.code)?;

    const QUERY: &str = "INSERT OR REPLACE INTO watched_accounts \
                         (id, account_header, code_commitment, storage_header, last_synced_block) \
                         VALUES (?, ?, ?, ?, ?)";

    tx.execute(
        QUERY,
        params![
            record.account_id.to_hex(),
            record.header.to_bytes(),
            record.code.commitment().to_hex(),
            record.storage_header.to_bytes(),
            i64::from(record.last_synced_block.as_u32()),
        ],
    )
    .into_store_error()?;

    Ok(())
}
