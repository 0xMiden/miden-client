use std::string::{String, ToString};

use miden_client::errors::ErrorCode;
use rusqlite::Error as RusqliteError;
use rusqlite_migration::Error as MigrationError;
use thiserror::Error;

// ERRORS
// ================================================================================================

/// Errors generated from the `SQLite` store.
#[derive(Debug, Error)]
pub enum SqliteStoreError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Migration error: {0}")]
    MigrationError(String),
    #[error("No migrations table in the database")]
    MissingMigrationsTable,
    #[error("Migration hashes mismatch")]
    MigrationHashMismatch,
}

impl ErrorCode for SqliteStoreError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::DatabaseError(_) => "MIDEN-SQ-001",
            Self::MigrationError(_) => "MIDEN-SQ-002",
            Self::MissingMigrationsTable => "MIDEN-SQ-003",
            Self::MigrationHashMismatch => "MIDEN-SQ-004",
        }
    }
}

impl From<RusqliteError> for SqliteStoreError {
    fn from(err: RusqliteError) -> Self {
        SqliteStoreError::DatabaseError(err.to_string())
    }
}

impl From<MigrationError> for SqliteStoreError {
    fn from(err: MigrationError) -> Self {
        SqliteStoreError::MigrationError(err.to_string())
    }
}
