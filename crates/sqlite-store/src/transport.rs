#![allow(clippy::items_after_statements)]

use miden_client::store::StoreError;
use rusqlite::{Connection, Transaction, params};

use super::SqliteStore;
use crate::sql_error::SqlResultExt;

impl SqliteStore {
    pub(super) fn get_transport_layer_cursor(conn: &mut Connection) -> Result<u64, StoreError> {
        const QUERY: &str = "SELECT cursor FROM transport_layer_cursor";

        conn.prepare(QUERY)
            .into_store_error()?
            .query_row([], |row| row.get(0))
            .into_store_error()
    }

    pub(super) fn update_transport_layer_cursor(
        conn: &mut Connection,
        cursor: u64,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;
        update_transport_layer_cursor(&tx, cursor)?;

        tx.commit().into_store_error()?;

        Ok(())
    }
}

pub(super) fn update_transport_layer_cursor(
    tx: &Transaction<'_>,
    cursor: u64,
) -> Result<(), StoreError> {
    const QUERY: &str = "UPDATE transport_layer_cursor SET cursor = ?";
    tx.execute(QUERY, params![cursor]).into_store_error()?;

    Ok(())
}
