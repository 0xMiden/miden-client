use core::fmt;

// NOTE SYNC ERROR
// ================================================================================================

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
