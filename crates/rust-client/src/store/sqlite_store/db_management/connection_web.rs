use alloc::string::ToString;
use rusqlite::{Connection, vtab::array};
use sqlite_wasm_rs::{self as ffi, sahpool_vfs::install as install_opfs_vfs};

use super::errors::SqliteStoreError;

pub async fn connect() -> Result<Connection, SqliteStoreError> {
    // TODO(Maks) extend with user provided path?
    install_opfs_vfs(None, true)
        .await
        .map_err(|e| SqliteStoreError::DatabaseError(e.to_string()))?;
    let mut db = core::ptr::null_mut();
    let ret = unsafe {
        ffi::sqlite3_open_v2(
            c"store.sqlite3".as_ptr().cast(),
            &mut db as *mut _,
            ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE,
            core::ptr::null(),
        )
    };

    assert_eq!(ffi::SQLITE_OK, ret);

    let conn = unsafe { Connection::from_handle_owned(db)? };
    // Feature used to support `IN` and `NOT IN` queries. We need to load
    // this module for every connection we create to the DB to support the
    // queries we want to run
    array::load_module(&conn)?;

    // Enable foreign key checks.
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}
