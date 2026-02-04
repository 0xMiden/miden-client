// NOTE SYNC ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::NoteSyncError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NoteSyncError {
    /// Internal server error (code 0)
    #[error("internal server error")]
    Internal,
    /// Invalid block range specified
    #[error("invalid block range")]
    InvalidBlockRange,
    /// Failed to deserialize data
    #[error("deserialization failed")]
    DeserializationFailed,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    #[error("unknown error (code {0})")]
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

// SYNC NULLIFIERS ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::SyncNullifiersError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SyncNullifiersError {
    /// Internal server error (code 0)
    #[error("internal server error")]
    Internal,
    /// Invalid block range specified
    #[error("invalid block range")]
    InvalidBlockRange,
    /// Invalid prefix length
    #[error("invalid prefix length")]
    InvalidPrefixLength,
    /// Failed to deserialize data
    #[error("deserialization failed")]
    DeserializationFailed,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    #[error("unknown error (code {0})")]
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

// SYNC ACCOUNT VAULT ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::SyncAccountVaultError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SyncAccountVaultError {
    /// Internal server error (code 0)
    #[error("internal server error")]
    Internal,
    /// Invalid block range specified
    #[error("invalid block range")]
    InvalidBlockRange,
    /// Failed to deserialize data
    #[error("deserialization failed")]
    DeserializationFailed,
    /// Account is not public
    #[error("account is not public")]
    AccountNotPublic,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    #[error("unknown error (code {0})")]
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

// SYNC ACCOUNT STORAGE MAPS ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::SyncAccountStorageMapsError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SyncAccountStorageMapsError {
    /// Internal server error (code 0)
    #[error("internal server error")]
    Internal,
    /// Invalid block range specified
    #[error("invalid block range")]
    InvalidBlockRange,
    /// Failed to deserialize data
    #[error("deserialization failed")]
    DeserializationFailed,
    /// Account was not found
    #[error("account not found")]
    AccountNotFound,
    /// Account is not public
    #[error("account is not public")]
    AccountNotPublic,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    #[error("unknown error (code {0})")]
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

// SYNC TRANSACTIONS ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::SyncTransactionsError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SyncTransactionsError {
    /// Internal server error (code 0)
    #[error("internal server error")]
    Internal,
    /// Invalid block range specified
    #[error("invalid block range")]
    InvalidBlockRange,
    /// Failed to deserialize data
    #[error("deserialization failed")]
    DeserializationFailed,
    /// Account was not found
    #[error("account not found")]
    AccountNotFound,
    /// Witness error
    #[error("witness error")]
    WitnessError,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    #[error("unknown error (code {0})")]
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
