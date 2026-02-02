//! Sync-related node RPC errors.

use core::fmt;

// NOTE SYNC ERROR
// ================================================================================================

/// Errors for the `SyncNotes` endpoint.
///
/// Error codes match `miden-node/crates/store/src/errors.rs::NoteSyncError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NoteSyncError {
    /// Internal server error
    Internal = 0,
    /// Invalid block range specified
    InvalidBlockRange = 1,
    /// Failed to deserialize data
    DeserializationFailed = 2,
}

impl From<u8> for NoteSyncError {
    fn from(code: u8) -> Self {
        match code {
            1 => Self::InvalidBlockRange,
            2 => Self::DeserializationFailed,
            _ => Self::Internal,
        }
    }
}

impl fmt::Display for NoteSyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::InvalidBlockRange => write!(f, "invalid block range"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
        }
    }
}

// SYNC NULLIFIERS ERROR
// ================================================================================================

/// Errors for the `SyncNullifiers` endpoint.
///
/// Error codes match `miden-node/crates/store/src/errors.rs::SyncNullifiersError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SyncNullifiersError {
    /// Internal server error
    Internal = 0,
    /// Invalid block range specified
    InvalidBlockRange = 1,
    /// Invalid prefix length
    InvalidPrefixLength = 2,
    /// Failed to deserialize data
    DeserializationFailed = 3,
}

impl From<u8> for SyncNullifiersError {
    fn from(code: u8) -> Self {
        match code {
            1 => Self::InvalidBlockRange,
            2 => Self::InvalidPrefixLength,
            3 => Self::DeserializationFailed,
            _ => Self::Internal,
        }
    }
}

impl fmt::Display for SyncNullifiersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::InvalidBlockRange => write!(f, "invalid block range"),
            Self::InvalidPrefixLength => write!(f, "invalid prefix length"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
        }
    }
}

// SYNC ACCOUNT VAULT ERROR
// ================================================================================================

/// Errors for the `SyncAccountVault` endpoint.
///
/// Error codes match `miden-node/crates/store/src/errors.rs::SyncAccountVaultError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SyncAccountVaultError {
    /// Internal server error
    Internal = 0,
    /// Invalid block range specified
    InvalidBlockRange = 1,
    /// Failed to deserialize data
    DeserializationFailed = 2,
    /// Account is not public
    AccountNotPublic = 3,
}

impl From<u8> for SyncAccountVaultError {
    fn from(code: u8) -> Self {
        match code {
            1 => Self::InvalidBlockRange,
            2 => Self::DeserializationFailed,
            3 => Self::AccountNotPublic,
            _ => Self::Internal,
        }
    }
}

impl fmt::Display for SyncAccountVaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::InvalidBlockRange => write!(f, "invalid block range"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
            Self::AccountNotPublic => write!(f, "account is not public"),
        }
    }
}

// SYNC ACCOUNT STORAGE MAPS ERROR
// ================================================================================================

/// Errors for the `SyncStorageMaps` endpoint.
///
/// Error codes match `miden-node/crates/store/src/errors.rs::SyncAccountStorageMapsError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SyncAccountStorageMapsError {
    /// Internal server error
    Internal = 0,
    /// Invalid block range specified
    InvalidBlockRange = 1,
    /// Failed to deserialize data
    DeserializationFailed = 2,
    /// Account was not found
    AccountNotFound = 3,
    /// Account is not public
    AccountNotPublic = 4,
}

impl From<u8> for SyncAccountStorageMapsError {
    fn from(code: u8) -> Self {
        match code {
            1 => Self::InvalidBlockRange,
            2 => Self::DeserializationFailed,
            3 => Self::AccountNotFound,
            4 => Self::AccountNotPublic,
            _ => Self::Internal,
        }
    }
}

impl fmt::Display for SyncAccountStorageMapsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::InvalidBlockRange => write!(f, "invalid block range"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
            Self::AccountNotFound => write!(f, "account not found"),
            Self::AccountNotPublic => write!(f, "account is not public"),
        }
    }
}

// SYNC TRANSACTIONS ERROR
// ================================================================================================

/// Errors for the `SyncTransactions` endpoint.
///
/// Error codes match `miden-node/crates/store/src/errors.rs::SyncTransactionsError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SyncTransactionsError {
    /// Internal server error
    Internal = 0,
    /// Invalid block range specified
    InvalidBlockRange = 1,
    /// Failed to deserialize data
    DeserializationFailed = 2,
    /// Account was not found
    AccountNotFound = 3,
    /// Witness error
    WitnessError = 4,
}

impl From<u8> for SyncTransactionsError {
    fn from(code: u8) -> Self {
        match code {
            1 => Self::InvalidBlockRange,
            2 => Self::DeserializationFailed,
            3 => Self::AccountNotFound,
            4 => Self::WitnessError,
            _ => Self::Internal,
        }
    }
}

impl fmt::Display for SyncTransactionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::InvalidBlockRange => write!(f, "invalid block range"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
            Self::AccountNotFound => write!(f, "account not found"),
            Self::WitnessError => write!(f, "witness error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_sync_error_codes() {
        assert_eq!(NoteSyncError::Internal as u8, 0);
        assert_eq!(NoteSyncError::InvalidBlockRange as u8, 1);
        assert_eq!(NoteSyncError::DeserializationFailed as u8, 2);
    }

    #[test]
    fn test_sync_nullifiers_error_codes() {
        assert_eq!(SyncNullifiersError::Internal as u8, 0);
        assert_eq!(SyncNullifiersError::InvalidBlockRange as u8, 1);
        assert_eq!(SyncNullifiersError::InvalidPrefixLength as u8, 2);
        assert_eq!(SyncNullifiersError::DeserializationFailed as u8, 3);
    }

    #[test]
    fn test_sync_account_vault_error_codes() {
        assert_eq!(SyncAccountVaultError::Internal as u8, 0);
        assert_eq!(SyncAccountVaultError::InvalidBlockRange as u8, 1);
        assert_eq!(SyncAccountVaultError::DeserializationFailed as u8, 2);
        assert_eq!(SyncAccountVaultError::AccountNotPublic as u8, 3);
    }

    #[test]
    fn test_sync_account_storage_maps_error_codes() {
        assert_eq!(SyncAccountStorageMapsError::Internal as u8, 0);
        assert_eq!(SyncAccountStorageMapsError::InvalidBlockRange as u8, 1);
        assert_eq!(SyncAccountStorageMapsError::DeserializationFailed as u8, 2);
        assert_eq!(SyncAccountStorageMapsError::AccountNotFound as u8, 3);
        assert_eq!(SyncAccountStorageMapsError::AccountNotPublic as u8, 4);
    }

    #[test]
    fn test_sync_transactions_error_codes() {
        assert_eq!(SyncTransactionsError::Internal as u8, 0);
        assert_eq!(SyncTransactionsError::InvalidBlockRange as u8, 1);
        assert_eq!(SyncTransactionsError::DeserializationFailed as u8, 2);
        assert_eq!(SyncTransactionsError::AccountNotFound as u8, 3);
        assert_eq!(SyncTransactionsError::WitnessError as u8, 4);
    }

    #[test]
    fn test_note_sync_error_from_code() {
        assert_eq!(NoteSyncError::from(0), NoteSyncError::Internal);
        assert_eq!(NoteSyncError::from(1), NoteSyncError::InvalidBlockRange);
        assert_eq!(NoteSyncError::from(2), NoteSyncError::DeserializationFailed);
        assert_eq!(NoteSyncError::from(99), NoteSyncError::Internal);
    }

    #[test]
    fn test_sync_nullifiers_error_from_code() {
        assert_eq!(SyncNullifiersError::from(0), SyncNullifiersError::Internal);
        assert_eq!(SyncNullifiersError::from(1), SyncNullifiersError::InvalidBlockRange);
        assert_eq!(SyncNullifiersError::from(2), SyncNullifiersError::InvalidPrefixLength);
        assert_eq!(SyncNullifiersError::from(3), SyncNullifiersError::DeserializationFailed);
        assert_eq!(SyncNullifiersError::from(99), SyncNullifiersError::Internal);
    }

    #[test]
    fn test_sync_account_vault_error_from_code() {
        assert_eq!(SyncAccountVaultError::from(0), SyncAccountVaultError::Internal);
        assert_eq!(SyncAccountVaultError::from(1), SyncAccountVaultError::InvalidBlockRange);
        assert_eq!(SyncAccountVaultError::from(2), SyncAccountVaultError::DeserializationFailed);
        assert_eq!(SyncAccountVaultError::from(3), SyncAccountVaultError::AccountNotPublic);
        assert_eq!(SyncAccountVaultError::from(99), SyncAccountVaultError::Internal);
    }

    #[test]
    fn test_sync_account_storage_maps_error_from_code() {
        assert_eq!(SyncAccountStorageMapsError::from(0), SyncAccountStorageMapsError::Internal);
        assert_eq!(
            SyncAccountStorageMapsError::from(1),
            SyncAccountStorageMapsError::InvalidBlockRange
        );
        assert_eq!(
            SyncAccountStorageMapsError::from(2),
            SyncAccountStorageMapsError::DeserializationFailed
        );
        assert_eq!(SyncAccountStorageMapsError::from(3), SyncAccountStorageMapsError::AccountNotFound);
        assert_eq!(SyncAccountStorageMapsError::from(4), SyncAccountStorageMapsError::AccountNotPublic);
        assert_eq!(SyncAccountStorageMapsError::from(99), SyncAccountStorageMapsError::Internal);
    }

    #[test]
    fn test_sync_transactions_error_from_code() {
        assert_eq!(SyncTransactionsError::from(0), SyncTransactionsError::Internal);
        assert_eq!(SyncTransactionsError::from(1), SyncTransactionsError::InvalidBlockRange);
        assert_eq!(SyncTransactionsError::from(2), SyncTransactionsError::DeserializationFailed);
        assert_eq!(SyncTransactionsError::from(3), SyncTransactionsError::AccountNotFound);
        assert_eq!(SyncTransactionsError::from(4), SyncTransactionsError::WitnessError);
        assert_eq!(SyncTransactionsError::from(99), SyncTransactionsError::Internal);
    }
}
