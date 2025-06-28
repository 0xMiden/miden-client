//! This module provides an SQLite-backed implementation of the [Store] trait.
//!
//! [`SqliteStore`] enables the persistence of accounts, transactions, notes, block headers, and MMR
//! nodes using an `SQLite` database.
//! It is compiled only when the `sqlite` feature flag is enabled.

mod account;
mod chain_data;
mod db_management;
mod errors;
mod note;
mod sync;
mod transaction;

#[cfg(not(target_arch = "wasm32"))]
mod store_native;
#[cfg(not(target_arch = "wasm32"))]
use db_management::SqlitePool;

#[cfg(target_arch = "wasm32")]
mod store_web;
#[cfg(target_arch = "wasm32")]
pub use store_web::WebStore;

use rusqlite::types::Value;

// SQLITE STORE
// ================================================================================================

/// Represents a pool of connections with an `SQLite` database. The pool is used to interact
/// concurrently with the underlying database in a safe and efficient manner.
///
/// Current table definitions can be found at `store.sql` migration file.
pub struct SqliteStore {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) pool: SqlitePool,
}

// UTILS
// ================================================================================================

/// Gets a `u64` value from the database.
///
/// `Sqlite` uses `i64` as its internal representation format, and so when retrieving
/// we need to make sure we cast as `u64` to get the original value
pub fn column_value_as_u64<I: rusqlite::RowIndex>(
    row: &rusqlite::Row<'_>,
    index: I,
) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    #[allow(
        clippy::cast_sign_loss,
        reason = "We store u64 as i64 as sqlite only allows the latter."
    )]
    Ok(value as u64)
}

/// Converts a `u64` into a [Value].
///
/// `Sqlite` uses `i64` as its internal representation format. Note that the `as` operator performs
/// a lossless conversion from `u64` to `i64`.
pub fn u64_to_value(v: u64) -> Value {
    #[allow(
        clippy::cast_possible_wrap,
        reason = "We store u64 as i64 as sqlite only allows the latter."
    )]
    Value::Integer(v as i64)
}
