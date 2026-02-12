//! Backend-agnostic SQL types used by both native (rusqlite) and WASM (JS FFI) backends.
//!
//! The [`SqlConnection`] trait provides a minimal abstraction over SQL execution.
//! [`SqlRow`], [`SqlParam`], and [`SqlValue`] are the common types for passing data
//! between the Store implementation and the SQL backend.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::store::StoreError;

// SQL CONNECTION TRAIT
// ================================================================================================

/// A minimal abstraction over SQL execution.
///
/// Both the native (rusqlite) and WASM (JS FFI) backends implement this trait.
/// All Store method implementations are written against this trait, enabling
/// code sharing across backends.
pub(crate) trait SqlConnection {
    /// Execute a statement that modifies data (INSERT, UPDATE, DELETE).
    /// Returns the number of affected rows.
    fn execute(&self, sql: &str, params: &[SqlParam]) -> Result<usize, StoreError>;

    /// Execute a SELECT query and return all matching rows.
    fn query_all(&self, sql: &str, params: &[SqlParam]) -> Result<Vec<SqlRow>, StoreError>;

    /// Execute a SELECT query and return at most one row.
    fn query_one(&self, sql: &str, params: &[SqlParam]) -> Result<Option<SqlRow>, StoreError>;
}

// SQL PARAMETER
// ================================================================================================

/// A backend-agnostic SQL parameter value.
#[derive(Debug, Clone)]
pub(crate) enum SqlParam {
    Null,
    Integer(i64),
    Text(String),
    Blob(Vec<u8>),
}

impl From<i64> for SqlParam {
    fn from(v: i64) -> Self {
        SqlParam::Integer(v)
    }
}

impl From<u32> for SqlParam {
    fn from(v: u32) -> Self {
        SqlParam::Integer(i64::from(v))
    }
}

impl From<u64> for SqlParam {
    fn from(v: u64) -> Self {
        #[allow(clippy::cast_possible_wrap)]
        SqlParam::Integer(v as i64)
    }
}

impl From<u8> for SqlParam {
    fn from(v: u8) -> Self {
        SqlParam::Integer(i64::from(v))
    }
}

impl From<bool> for SqlParam {
    fn from(v: bool) -> Self {
        SqlParam::Integer(i64::from(v))
    }
}

impl From<String> for SqlParam {
    fn from(v: String) -> Self {
        SqlParam::Text(v)
    }
}

impl From<&str> for SqlParam {
    fn from(v: &str) -> Self {
        SqlParam::Text(v.to_string())
    }
}

impl From<Vec<u8>> for SqlParam {
    fn from(v: Vec<u8>) -> Self {
        SqlParam::Blob(v)
    }
}

impl<T: Into<SqlParam>> From<Option<T>> for SqlParam {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => v.into(),
            None => SqlParam::Null,
        }
    }
}

// SQL VALUE
// ================================================================================================

/// A value read from a SQL result row.
#[derive(Debug, Clone)]
pub(crate) enum SqlValue {
    Null,
    Integer(i64),
    Text(String),
    Blob(Vec<u8>),
}

// SQL ROW
// ================================================================================================

/// A row of values from a SQL query result.
///
/// Provides typed accessors by column index. Column indices are 0-based and correspond
/// to the order of columns in the SELECT statement.
#[derive(Debug, Clone)]
pub(crate) struct SqlRow(pub(crate) Vec<SqlValue>);

impl SqlRow {
    /// Get a text (String) value at the given column index.
    pub(crate) fn get_text(&self, idx: usize) -> Result<&str, StoreError> {
        match self.0.get(idx) {
            Some(SqlValue::Text(s)) => Ok(s.as_str()),
            Some(SqlValue::Null) => {
                Err(StoreError::DatabaseError(format!("expected text at column {idx}, got NULL")))
            },
            Some(other) => Err(StoreError::DatabaseError(format!(
                "expected text at column {idx}, got {other:?}"
            ))),
            None => Err(StoreError::DatabaseError(format!("column index {idx} out of bounds"))),
        }
    }

    /// Get an optional text (String) value at the given column index.
    #[allow(dead_code)]
    pub(crate) fn get_optional_text(&self, idx: usize) -> Result<Option<&str>, StoreError> {
        match self.0.get(idx) {
            Some(SqlValue::Text(s)) => Ok(Some(s.as_str())),
            Some(SqlValue::Null) => Ok(None),
            Some(other) => Err(StoreError::DatabaseError(format!(
                "expected text or NULL at column {idx}, got {other:?}"
            ))),
            None => Err(StoreError::DatabaseError(format!("column index {idx} out of bounds"))),
        }
    }

    /// Get a blob (Vec<u8>) value at the given column index.
    pub(crate) fn get_blob(&self, idx: usize) -> Result<&[u8], StoreError> {
        match self.0.get(idx) {
            Some(SqlValue::Blob(b)) => Ok(b.as_slice()),
            Some(SqlValue::Null) => {
                Err(StoreError::DatabaseError(format!("expected blob at column {idx}, got NULL")))
            },
            Some(other) => Err(StoreError::DatabaseError(format!(
                "expected blob at column {idx}, got {other:?}"
            ))),
            None => Err(StoreError::DatabaseError(format!("column index {idx} out of bounds"))),
        }
    }

    /// Get an optional blob value at the given column index.
    pub(crate) fn get_optional_blob(&self, idx: usize) -> Result<Option<&[u8]>, StoreError> {
        match self.0.get(idx) {
            Some(SqlValue::Blob(b)) => Ok(Some(b.as_slice())),
            Some(SqlValue::Null) => Ok(None),
            Some(other) => Err(StoreError::DatabaseError(format!(
                "expected blob or NULL at column {idx}, got {other:?}"
            ))),
            None => Err(StoreError::DatabaseError(format!("column index {idx} out of bounds"))),
        }
    }

    /// Get an i64 value at the given column index.
    pub(crate) fn get_i64(&self, idx: usize) -> Result<i64, StoreError> {
        match self.0.get(idx) {
            Some(SqlValue::Integer(v)) => Ok(*v),
            Some(SqlValue::Null) => Err(StoreError::DatabaseError(format!(
                "expected integer at column {idx}, got NULL"
            ))),
            Some(other) => Err(StoreError::DatabaseError(format!(
                "expected integer at column {idx}, got {other:?}"
            ))),
            None => Err(StoreError::DatabaseError(format!("column index {idx} out of bounds"))),
        }
    }

    /// Get an optional i64 value at the given column index.
    #[allow(dead_code)]
    pub(crate) fn get_optional_i64(&self, idx: usize) -> Result<Option<i64>, StoreError> {
        match self.0.get(idx) {
            Some(SqlValue::Integer(v)) => Ok(Some(*v)),
            Some(SqlValue::Null) => Ok(None),
            Some(other) => Err(StoreError::DatabaseError(format!(
                "expected integer or NULL at column {idx}, got {other:?}"
            ))),
            None => Err(StoreError::DatabaseError(format!("column index {idx} out of bounds"))),
        }
    }

    /// Get a u32 value at the given column index (stored as i64 in `SQLite`).
    pub(crate) fn get_u32(&self, idx: usize) -> Result<u32, StoreError> {
        let v = self.get_i64(idx)?;
        u32::try_from(v).map_err(|_| {
            StoreError::DatabaseError(format!("value {v} at column {idx} doesn't fit in u32"))
        })
    }

    /// Get a u64 value at the given column index (stored as i64 in `SQLite`).
    #[allow(clippy::cast_sign_loss)]
    pub(crate) fn get_u64(&self, idx: usize) -> Result<u64, StoreError> {
        let v = self.get_i64(idx)?;
        Ok(v as u64)
    }

    /// Get a bool value at the given column index (stored as integer 0/1 in `SQLite`).
    pub(crate) fn get_bool(&self, idx: usize) -> Result<bool, StoreError> {
        let v = self.get_i64(idx)?;
        Ok(v != 0)
    }
}
