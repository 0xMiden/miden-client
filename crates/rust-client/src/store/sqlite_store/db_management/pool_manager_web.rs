use super::errors::SqliteStoreError;
use alloc::string::ToString;
use alloc::sync::Arc;
use async_lock::Mutex;
use rusqlite::{Connection, vtab::array};
use sqlite_wasm_rs::{self as ffi, sahpool_vfs::install as install_opfs_vfs};

// TODO(Maks) - this dummy pool implementation with interior mutability only for POC!!
// It blocks JS event loop with sync calls in the async context
// Necessary to implement web workers based pooling
// E.g. https://github.com/w3reality/wasm-mt
pub struct SqlitePool {
    connection: Arc<Mutex<Connection>>,
}

impl SqlitePool {
    pub async fn connect(path: &'static str) -> Result<Self, SqliteStoreError> {
        // TODO(Maks) extend with user provided path? (at least at comptime)
        install_opfs_vfs(None, true)
            .await
            .map_err(|e| SqliteStoreError::DatabaseError(e.to_string()))?;
        let mut db = core::ptr::null_mut();
        // TODO(Maks) justify unsafe invariants
        let ret = unsafe {
            ffi::sqlite3_open_v2(
                path.as_ptr().cast(),
                &mut db as *mut _,
                ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE,
                core::ptr::null(),
            )
        };

        // TODO(Maks) make an error
        assert_eq!(ffi::SQLITE_OK, ret);

        // TODO(Maks) justify unsafe invariants
        let connection = unsafe { Connection::from_handle_owned(db)? };
        // Feature used to support `IN` and `NOT IN` queries. We need to load
        // this module for every connection we create to the DB to support the
        // queries we want to run
        array::load_module(&connection)?;

        // Enable foreign key checks.
        connection.pragma_update(None, "foreign_keys", "ON")?;

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
