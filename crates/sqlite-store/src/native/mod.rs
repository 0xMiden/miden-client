//! Native (rusqlite) backend for `SqlConnection`.
//!
//! This module wraps `rusqlite::Connection` and `rusqlite::Transaction` to implement
//! the [`SqlConnection`] trait, enabling the shared Store logic to execute SQL.

use std::vec::Vec;

use miden_client::store::StoreError;
use rusqlite::types::{ToSqlOutput, ValueRef};

use crate::sql_error::SqlResultExt;
use crate::sql_types::{SqlConnection, SqlParam, SqlRow, SqlValue};

// RUSQLITE CONNECTION WRAPPER
// ================================================================================================

/// Wraps a `&rusqlite::Connection` to implement [`SqlConnection`].
pub(crate) struct RusqliteConnection<'a>(pub(crate) &'a rusqlite::Connection);

impl SqlConnection for RusqliteConnection<'_> {
    fn execute(&self, sql: &str, params: &[SqlParam]) -> Result<usize, StoreError> {
        let rusqlite_params = to_rusqlite_params(params);
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            rusqlite_params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();

        self.0.execute(sql, param_refs.as_slice()).into_store_error()
    }

    fn query_all(&self, sql: &str, params: &[SqlParam]) -> Result<Vec<SqlRow>, StoreError> {
        let rusqlite_params = to_rusqlite_params(params);
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            rusqlite_params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();

        let mut stmt = self.0.prepare(sql).into_store_error()?;
        let col_count = stmt.column_count();
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| row_to_sql_row(row, col_count))
            .into_store_error()?
            .map(super::sql_error::SqlResultExt::into_store_error)
            .collect::<Result<Vec<SqlRow>, StoreError>>()?;

        Ok(rows)
    }

    fn query_one(&self, sql: &str, params: &[SqlParam]) -> Result<Option<SqlRow>, StoreError> {
        let rusqlite_params = to_rusqlite_params(params);
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            rusqlite_params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();

        let mut stmt = self.0.prepare(sql).into_store_error()?;
        let col_count = stmt.column_count();
        let mut rows = stmt
            .query_map(param_refs.as_slice(), |row| row_to_sql_row(row, col_count))
            .into_store_error()?;

        match rows.next() {
            Some(row) => Ok(Some(row.into_store_error()?)),
            None => Ok(None),
        }
    }
}

/// Wraps a `&rusqlite::Transaction` to implement [`SqlConnection`].
///
/// Since `rusqlite::Transaction` derefs to `Connection`, the implementation is identical.
pub(crate) struct RusqliteTransaction<'a>(pub(crate) &'a rusqlite::Transaction<'a>);

impl SqlConnection for RusqliteTransaction<'_> {
    fn execute(&self, sql: &str, params: &[SqlParam]) -> Result<usize, StoreError> {
        let rusqlite_params = to_rusqlite_params(params);
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            rusqlite_params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();

        self.0.execute(sql, param_refs.as_slice()).into_store_error()
    }

    fn query_all(&self, sql: &str, params: &[SqlParam]) -> Result<Vec<SqlRow>, StoreError> {
        let conn: &rusqlite::Connection = self.0;
        RusqliteConnection(conn).query_all(sql, params)
    }

    fn query_one(&self, sql: &str, params: &[SqlParam]) -> Result<Option<SqlRow>, StoreError> {
        let conn: &rusqlite::Connection = self.0;
        RusqliteConnection(conn).query_one(sql, params)
    }
}

// CONVERSION HELPERS
// ================================================================================================

/// An owned rusqlite parameter value that implements `ToSql`.
enum OwnedSqlParam {
    Null,
    Integer(i64),
    Text(String),
    Blob(Vec<u8>),
}

impl rusqlite::types::ToSql for OwnedSqlParam {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            OwnedSqlParam::Null => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Null)),
            OwnedSqlParam::Integer(v) => {
                Ok(ToSqlOutput::Owned(rusqlite::types::Value::Integer(*v)))
            },
            OwnedSqlParam::Text(s) => Ok(ToSqlOutput::Borrowed(ValueRef::Text(s.as_bytes()))),
            OwnedSqlParam::Blob(b) => Ok(ToSqlOutput::Borrowed(ValueRef::Blob(b))),
        }
    }
}

/// Convert `&[SqlParam]` to a Vec of owned rusqlite-compatible params.
fn to_rusqlite_params(params: &[SqlParam]) -> Vec<OwnedSqlParam> {
    params
        .iter()
        .map(|p| match p {
            SqlParam::Null => OwnedSqlParam::Null,
            SqlParam::Integer(v) => OwnedSqlParam::Integer(*v),
            SqlParam::Text(s) => OwnedSqlParam::Text(s.clone()),
            SqlParam::Blob(b) => OwnedSqlParam::Blob(b.clone()),
        })
        .collect()
}

/// Convert a `rusqlite::Row` to a `SqlRow`.
fn row_to_sql_row(row: &rusqlite::Row<'_>, col_count: usize) -> rusqlite::Result<SqlRow> {
    let mut values = Vec::with_capacity(col_count);
    for i in 0..col_count {
        let value_ref = row.get_ref(i)?;
        let value = match value_ref {
            ValueRef::Null => SqlValue::Null,
            ValueRef::Integer(v) => SqlValue::Integer(v),
            ValueRef::Real(_) => {
                // We don't use floats in our schema, but handle gracefully
                SqlValue::Null
            },
            ValueRef::Text(bytes) => {
                let s = std::str::from_utf8(bytes).map_err(rusqlite::Error::Utf8Error)?;
                SqlValue::Text(s.to_string())
            },
            ValueRef::Blob(bytes) => SqlValue::Blob(bytes.to_vec()),
        };
        values.push(value);
    }
    Ok(SqlRow(values))
}
