#![allow(clippy::items_after_statements)]

use miden_client::store::StoreError;
use rusqlite::{Connection, Transaction, params};

use super::SqliteStore;
use crate::sql_error::SqlResultExt;

impl SqliteStore {
    pub(super) fn get_note_transport_cursor(conn: &mut Connection) -> Result<u64, StoreError> {
        const QUERY: &str = "SELECT cursor FROM note_transport_cursor";

        conn.prepare(QUERY)
            .into_store_error()?
            .query_row([], |row| row.get(0))
            .into_store_error()
    }

    pub(super) fn update_note_transport_cursor(
        conn: &mut Connection,
        cursor: u64,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;
        update_note_transport_cursor(&tx, cursor)?;

        tx.commit().into_store_error()?;

        Ok(())
    }
}

pub(super) fn update_note_transport_cursor(
    tx: &Transaction<'_>,
    cursor: u64,
) -> Result<(), StoreError> {
    const QUERY: &str = "UPDATE note_transport_cursor SET cursor = ?";
    tx.execute(QUERY, params![cursor]).into_store_error()?;

    Ok(())
}
