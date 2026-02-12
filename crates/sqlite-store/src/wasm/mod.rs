//! WASM (JS FFI) backend for `SqlConnection`.
//!
//! This module provides `WasmConnection` which implements [`SqlConnection`] by calling
//! into JavaScript via wasm-bindgen. The JS layer (`sql.js`) executes SQL against a
//! better-sqlite3 (Node.js) or wa-sqlite (browser) database.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use js_sys::{Array, Uint8Array};
use miden_client::note::ToInputNoteCommitments;
use miden_client::store::StoreError;
use miden_client::transaction::{TransactionRecord, TransactionStoreUpdate};
use wasm_bindgen::prelude::*;

use crate::sql_types::{SqlConnection, SqlParam, SqlRow, SqlValue};
use crate::{current_timestamp_u64, note, sync, transaction};

// JS FFI BINDINGS
// ================================================================================================

#[wasm_bindgen(module = "/src/wasm/js/sql.js")]
extern "C" {
    /// Execute a SQL statement (INSERT, UPDATE, DELETE). Returns affected row count.
    #[wasm_bindgen(js_name = sqlExecute)]
    fn sql_execute(db_id: &str, sql: &str, params: &JsValue) -> JsValue;

    /// Execute a SELECT query and return all rows as array of arrays.
    #[wasm_bindgen(js_name = sqlQueryAll)]
    fn sql_query_all(db_id: &str, sql: &str, params: &JsValue) -> JsValue;

    /// Execute a SELECT query and return at most one row (array or null).
    #[wasm_bindgen(js_name = sqlQueryOne)]
    fn sql_query_one(db_id: &str, sql: &str, params: &JsValue) -> JsValue;
}

#[wasm_bindgen(module = "/src/wasm/js/schema.js")]
extern "C" {
    /// Opens the database and registers it in the JS registry.
    #[wasm_bindgen(js_name = openDatabase)]
    fn js_open_database(db_name: &str, client_version: &str) -> js_sys::Promise;
}

/// Opens the database by calling the JS-side `openDatabase` function.
pub(crate) fn open_database(db_name: &str, client_version: &str) -> js_sys::Promise {
    js_open_database(db_name, client_version)
}

#[wasm_bindgen(module = "/src/wasm/js/utils.js")]
extern "C" {
    #[wasm_bindgen(js_name = logError)]
    fn log_error(error: JsValue, error_context: String);
}

// WASM CONNECTION
// ================================================================================================

/// A SQL connection backed by JavaScript FFI.
///
/// Wraps a `database_id` string that identifies the database in the JS-side registry.
/// All SQL operations are synchronous calls into JS (better-sqlite3 is synchronous).
pub(crate) struct WasmConnection<'a> {
    db_id: &'a str,
}

impl<'a> WasmConnection<'a> {
    pub(crate) fn new(db_id: &'a str) -> Self {
        Self { db_id }
    }
}

impl SqlConnection for WasmConnection<'_> {
    fn execute(&self, sql: &str, params: &[SqlParam]) -> Result<usize, StoreError> {
        let js_params = params_to_js(params);
        let result = sql_execute(self.db_id, sql, &js_params);

        if result.is_undefined() || result.is_null() {
            return Ok(0);
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        result.as_f64().map(|v| v as usize).ok_or_else(|| {
            StoreError::DatabaseError(format!("sqlExecute returned non-number: {result:?}"))
        })
    }

    fn query_all(&self, sql: &str, params: &[SqlParam]) -> Result<Vec<SqlRow>, StoreError> {
        let js_params = params_to_js(params);
        let result = sql_query_all(self.db_id, sql, &js_params);

        if result.is_undefined() || result.is_null() {
            return Ok(Vec::new());
        }

        let outer_array: Array = result.dyn_into().map_err(|_| {
            StoreError::DatabaseError("sqlQueryAll did not return an array".to_string())
        })?;

        let mut rows = Vec::with_capacity(outer_array.length() as usize);
        for i in 0..outer_array.length() {
            let row_val = outer_array.get(i);
            let row = js_row_to_sql_row(row_val)?;
            rows.push(row);
        }

        Ok(rows)
    }

    fn query_one(&self, sql: &str, params: &[SqlParam]) -> Result<Option<SqlRow>, StoreError> {
        let js_params = params_to_js(params);
        let result = sql_query_one(self.db_id, sql, &js_params);

        if result.is_undefined() || result.is_null() {
            return Ok(None);
        }

        Ok(Some(js_row_to_sql_row(result)?))
    }
}

// CONVERSION HELPERS
// ================================================================================================

/// Convert `&[SqlParam]` to a JS Array for passing to JS functions.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn params_to_js(params: &[SqlParam]) -> JsValue {
    let arr = Array::new_with_length(params.len() as u32);
    for (i, p) in params.iter().enumerate() {
        let val = match p {
            SqlParam::Null => JsValue::NULL,
            SqlParam::Integer(v) => JsValue::from(*v as f64),
            SqlParam::Text(s) => JsValue::from_str(s),
            SqlParam::Blob(b) => {
                let uint8arr = Uint8Array::from(b.as_slice());
                uint8arr.into()
            },
        };
        arr.set(i as u32, val);
    }
    arr.into()
}

/// Convert a JS row value (Array of column values) to a `SqlRow`.
fn js_row_to_sql_row(row_val: JsValue) -> Result<SqlRow, StoreError> {
    let row_array: Array = row_val
        .dyn_into()
        .map_err(|_| StoreError::DatabaseError("row is not an array".to_string()))?;

    let mut values = Vec::with_capacity(row_array.length() as usize);
    for j in 0..row_array.length() {
        let cell = row_array.get(j);
        let value = js_value_to_sql_value(&cell);
        values.push(value);
    }

    Ok(SqlRow(values))
}

/// Convert a single JS value to a `SqlValue`.
#[allow(clippy::cast_possible_truncation)]
fn js_value_to_sql_value(val: &JsValue) -> SqlValue {
    if val.is_null() || val.is_undefined() {
        SqlValue::Null
    } else if let Some(n) = val.as_f64() {
        // JavaScript numbers are f64; cast to i64 for SQLite integer representation
        SqlValue::Integer(n as i64)
    } else if let Some(s) = val.as_string() {
        SqlValue::Text(s)
    } else if val.is_instance_of::<Uint8Array>() {
        // Buffer (Node.js) is a subclass of Uint8Array, so this catches both
        let arr: Uint8Array = val.clone().unchecked_into();
        SqlValue::Blob(arr.to_vec())
    } else {
        // Unknown type — treat as null to be safe
        SqlValue::Null
    }
}

/// Client version from Cargo.toml, used for DB version enforcement.
pub(crate) const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// SQLITE STORE (WASM)
// ================================================================================================

/// WASM variant of `SqliteStore` backed by a JavaScript `SQLite` adapter.
///
/// The database reference is stored in a JavaScript registry and looked up by
/// `database_id` when needed. This avoids storing `JsValue` references in Rust
/// which would prevent the struct from being Send + Sync.
pub struct SqliteStore {
    pub(crate) database_id: String,
}

impl SqliteStore {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Creates a new `SqliteStore` backed by a JS `SQLite` adapter.
    pub async fn new(database_name: String) -> Result<Self, StoreError> {
        let promise = open_database(database_name.as_str(), CLIENT_VERSION);
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|e| StoreError::DatabaseError(format!("Failed to open database: {e:?}")))?;
        Ok(SqliteStore { database_id: database_name })
    }

    /// Execute a closure with a [`SqlConnection`] for read-only queries.
    #[allow(clippy::unused_async)]
    pub(crate) async fn run<F, T>(&self, f: F) -> Result<T, StoreError>
    where
        F: FnOnce(&dyn SqlConnection) -> Result<T, StoreError>,
    {
        let conn = WasmConnection::new(&self.database_id);
        f(&conn)
    }

    /// Execute a closure within a SQL transaction via [`SqlConnection`].
    ///
    /// On WASM, better-sqlite3 is synchronous, so we use BEGIN/COMMIT/ROLLBACK SQL statements.
    #[allow(clippy::unused_async)]
    pub(crate) async fn run_in_tx<F, T>(&self, f: F) -> Result<T, StoreError>
    where
        F: FnOnce(&dyn SqlConnection) -> Result<T, StoreError>,
    {
        let conn = WasmConnection::new(&self.database_id);
        conn.execute("BEGIN TRANSACTION", &[])?;
        match f(&conn) {
            Ok(result) => {
                conn.execute("COMMIT", &[])?;
                Ok(result)
            },
            Err(e) => {
                let _ = conn.execute("ROLLBACK", &[]);
                Err(e)
            },
        }
    }
}

// APPLY TRANSACTION (WASM)
// ================================================================================================

/// WASM implementation of `apply_transaction`.
///
/// On WASM we don't have SMT forest, so we insert the transaction record and note
/// updates, but skip account delta application (account state is updated via
/// `apply_state_sync`).
pub(crate) fn apply_transaction_impl(
    conn: &dyn SqlConnection,
    tx_update: &TransactionStoreUpdate,
) -> Result<(), StoreError> {
    // Build transaction record
    let executed_transaction = tx_update.executed_transaction();
    let nullifiers: Vec<miden_client::Word> = executed_transaction
        .input_notes()
        .iter()
        .map(|x| x.nullifier().as_word())
        .collect();

    let details = miden_client::transaction::TransactionDetails {
        account_id: executed_transaction.account_id(),
        init_account_state: executed_transaction.initial_account().commitment(),
        final_account_state: executed_transaction.final_account().commitment(),
        input_note_nullifiers: nullifiers,
        output_notes: executed_transaction.output_notes().clone(),
        block_num: executed_transaction.block_header().block_num(),
        submission_height: tx_update.submission_height(),
        expiration_block_num: executed_transaction.expiration_block_num(),
        creation_timestamp: current_timestamp_u64(),
    };

    let transaction_record = TransactionRecord::new(
        executed_transaction.id(),
        details,
        executed_transaction.tx_args().tx_script().cloned(),
        miden_client::transaction::TransactionStatus::Pending,
    );

    transaction::upsert_transaction_record_shared(conn, &transaction_record)?;

    // Note updates
    note::apply_note_updates_shared(conn, tx_update.note_updates())?;

    // Note tags
    for tag_record in tx_update.new_tags() {
        sync::insert_note_tag_shared(conn, tag_record)?;
    }

    Ok(())
}
