use core::fmt;

// NOTE SYNC ERROR
// ================================================================================================

/// Error codes match `miden-node/crates/store/src/errors.rs::NoteSyncError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteSyncError {
    /// Internal server error (code 0)
    Internal,
    /// Invalid block range specified
    InvalidBlockRange,
    /// Failed to deserialize data
    DeserializationFailed,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    Unknown(u8),
}

impl From<u8> for NoteSyncError {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Internal,
            1 => Self::InvalidBlockRange,
            2 => Self::DeserializationFailed,
            _ => Self::Unknown(code),
        }
    }
}

impl fmt::Display for NoteSyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::InvalidBlockRange => write!(f, "invalid block range"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}

// SYNC NULLIFIERS ERROR
// ================================================================================================

/// Error codes match `miden-node/crates/store/src/errors.rs::SyncNullifiersError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncNullifiersError {
    /// Internal server error (code 0)
    Internal,
    /// Invalid block range specified
    InvalidBlockRange,
    /// Invalid prefix length
    InvalidPrefixLength,
    /// Failed to deserialize data
    DeserializationFailed,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    Unknown(u8),
}

impl From<u8> for SyncNullifiersError {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Internal,
            1 => Self::InvalidBlockRange,
            2 => Self::InvalidPrefixLength,
            3 => Self::DeserializationFailed,
            _ => Self::Unknown(code),
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
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}

// SYNC ACCOUNT VAULT ERROR
// ================================================================================================

/// Error codes match `miden-node/crates/store/src/errors.rs::SyncAccountVaultError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAccountVaultError {
    /// Internal server error (code 0)
    Internal,
    /// Invalid block range specified
    InvalidBlockRange,
    /// Failed to deserialize data
    DeserializationFailed,
    /// Account is not public
    AccountNotPublic,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    Unknown(u8),
}

impl From<u8> for SyncAccountVaultError {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Internal,
            1 => Self::InvalidBlockRange,
            2 => Self::DeserializationFailed,
            3 => Self::AccountNotPublic,
            _ => Self::Unknown(code),
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
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}

// SYNC ACCOUNT STORAGE MAPS ERROR
// ================================================================================================

/// Error codes match `miden-node/crates/store/src/errors.rs::SyncAccountStorageMapsError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAccountStorageMapsError {
    /// Internal server error (code 0)
    Internal,
    /// Invalid block range specified
    InvalidBlockRange,
    /// Failed to deserialize data
    DeserializationFailed,
    /// Account was not found
    AccountNotFound,
    /// Account is not public
    AccountNotPublic,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    Unknown(u8),
}

impl From<u8> for SyncAccountStorageMapsError {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Internal,
            1 => Self::InvalidBlockRange,
            2 => Self::DeserializationFailed,
            3 => Self::AccountNotFound,
            4 => Self::AccountNotPublic,
            _ => Self::Unknown(code),
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
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}

// SYNC TRANSACTIONS ERROR
// ================================================================================================

/// Error codes match `miden-node/crates/store/src/errors.rs::SyncTransactionsError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncTransactionsError {
    /// Internal server error (code 0)
    Internal,
    /// Invalid block range specified
    InvalidBlockRange,
    /// Failed to deserialize data
    DeserializationFailed,
    /// Account was not found
    AccountNotFound,
    /// Witness error
    WitnessError,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    Unknown(u8),
}

impl From<u8> for SyncTransactionsError {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Internal,
            1 => Self::InvalidBlockRange,
            2 => Self::DeserializationFailed,
            3 => Self::AccountNotFound,
            4 => Self::WitnessError,
            _ => Self::Unknown(code),
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
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}
