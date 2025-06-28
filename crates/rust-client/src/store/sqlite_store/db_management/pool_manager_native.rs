use alloc::string::ToString;
use std::path::PathBuf;

use core::ops::Deref;
use deadpool::{
    Runtime,
    managed::{Manager, Metrics, RecycleResult},
};
use rusqlite::{Connection, vtab::array};

use super::errors::SqliteStoreError;

deadpool::managed_reexports!(
    "miden-client-sqlite-store",
    SqlitePoolManager,
    deadpool::managed::Object<SqlitePoolManager>,
    rusqlite::Error,
    SqliteStoreError
);

const RUNTIME: Runtime = Runtime::Tokio1;

// POOL MANAGER
// ================================================================================================

/// `SQLite` connection pool manager
pub struct SqlitePoolManager {
    database_path: PathBuf,
}

/// `SQLite` connection pool manager
impl SqlitePoolManager {
    pub fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }

    fn new_connection(&self) -> rusqlite::Result<Connection> {
        let conn = Connection::open(&self.database_path)?;

        // Feature used to support `IN` and `NOT IN` queries. We need to load
        // this module for every connection we create to the DB to support the
        // queries we want to run
        array::load_module(&conn)?;

        // Enable foreign key checks.
        conn.pragma_update(None, "foreign_keys", "ON")?;

        Ok(conn)
    }
}

impl Manager for SqlitePoolManager {
    type Type = deadpool_sync::SyncWrapper<Connection>;
    type Error = rusqlite::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let conn = self.new_connection();
        deadpool_sync::SyncWrapper::new(RUNTIME, move || conn).await
    }

    async fn recycle(&self, _: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        Ok(())
    }
}

pub struct SqlitePool(Pool);

impl Deref for SqlitePool {
    type Target = Pool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SqlitePool {
    pub async fn connect(database_path: PathBuf) -> Result<SqlitePool, SqliteStoreError> {
        let sqlite_pool_manager = SqlitePoolManager::new(database_path);
        let pool = Pool::builder(sqlite_pool_manager)
            .build()
            .map_err(|e| SqliteStoreError::DatabaseError(e.to_string()))?;
        Ok(SqlitePool(pool))
    }
}
