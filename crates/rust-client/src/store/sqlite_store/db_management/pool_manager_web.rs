use super::errors::SqliteStoreError;
use alloc::string::ToString;
use alloc::sync::Arc;
use async_lock::Mutex;
use rusqlite::{Connection, vtab::array};
use sqlite_wasm_rs::{self as ffi, sahpool_vfs::install as install_opfs_vfs};
use core::ffi::CStr;

// TODO(Maks) - this dummy pool implementation with interior mutability only for POC!!
// It blocks JS event loop with sync calls in the async context
// Necessary to implement web workers based pooling
// E.g. https://github.com/w3reality/wasm-mt, https://github.com/paberr/wasmworker, https://rustwasm.github.io/wasm-bindgen/examples/wasm-in-web-worker.html
// see also crates/web-client/js/index.js
pub struct SqlitePool {
    connection: Arc<Mutex<Connection>>,
}

impl SqlitePool {
    pub async fn connect(path: &'static str) -> Result<Self, SqliteStoreError> {
        let cstr = CStr::from_bytes_with_nul(path.as_bytes())
            .map_err(|_| SqliteStoreError::ConfigurationError(format!("sqlite db name should be null terminated: {path}")))?;

        // This implementation of VFS is only available in a dedicated worker
        install_opfs_vfs(None, true)
            .await
            .map_err(|e| SqliteStoreError::DatabaseError(e.to_string()))?;

        let mut db = core::ptr::null_mut();
        // TODO(Maks) justify unsafe invariants
        let ret = unsafe {
            ffi::sqlite3_open_v2(
                cstr.as_ptr().cast(),
                &mut db as *mut _,
                ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE,
                core::ptr::null(),
            )
        };

        if ret != ffi::SQLITE_OK {
            return Err(SqliteStoreError::DatabaseError(format!("error opening sqlite db: {ret}")));
        }

        // TODO(Maks) justify unsafe invariants
        let connection = unsafe { Connection::from_handle_owned(db)? };
        // Feature used to support `IN` and `NOT IN` queries. We need to load
        // this module for every connection we create to the DB to support the
        // queries we want to run
        array::load_module(&connection).map_err(|e| {
            SqliteStoreError::DatabaseError(format!("error loading array module: {e:#?}"))
        })?;

        // Enable foreign key checks.
        connection.pragma_update(None, "foreign_keys", "ON").map_err(|e| {
            SqliteStoreError::DatabaseError(format!("error enabling foreign keys: {e:#?}"))
        })?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub async fn interact<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&mut Connection) -> Result<R, E> + Send + 'static,
        R: Send + 'static,
        E: core::error::Error,
    {
        let mut conn = self.connection.lock().await;
        f(&mut conn)
    }
}
