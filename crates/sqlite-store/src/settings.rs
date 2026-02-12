use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::store::StoreError;

use crate::sql_types::{SqlConnection, SqlParam};

/// Get a setting by name using [`SqlConnection`].
pub(crate) fn get_setting_shared(
    conn: &dyn SqlConnection,
    name: &str,
) -> Result<Option<Vec<u8>>, StoreError> {
    let row = conn.query_one(
        "SELECT value FROM settings WHERE name = ?",
        &[SqlParam::Text(name.to_string())],
    )?;
    match row {
        Some(row) => Ok(Some(row.get_blob(0)?.to_vec())),
        None => Ok(None),
    }
}

/// Set a setting by name using [`SqlConnection`].
pub(crate) fn set_setting_shared(
    conn: &dyn SqlConnection,
    name: &str,
    value: &[u8],
) -> Result<(), StoreError> {
    conn.execute(
        insert_sql!(settings { name, value } | REPLACE),
        &[SqlParam::Text(name.to_string()), SqlParam::Blob(value.to_vec())],
    )?;
    Ok(())
}

/// Remove a setting by name using [`SqlConnection`].
pub(crate) fn remove_setting_shared(
    conn: &dyn SqlConnection,
    name: &str,
) -> Result<(), StoreError> {
    conn.execute("DELETE FROM settings WHERE name = ?", &[SqlParam::Text(name.to_string())])?;
    Ok(())
}

/// List all setting keys using [`SqlConnection`].
pub(crate) fn list_setting_keys_shared(
    conn: &dyn SqlConnection,
) -> Result<Vec<String>, StoreError> {
    let rows = conn.query_all("SELECT name FROM settings", &[])?;
    rows.into_iter().map(|row| Ok(row.get_text(0)?.to_string())).collect()
}
