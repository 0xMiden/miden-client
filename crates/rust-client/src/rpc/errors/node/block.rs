use core::fmt;

// GET BLOCK HEADER ERROR
// ================================================================================================

/// Error codes match `miden-node/crates/store/src/errors.rs::GetBlockHeaderError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetBlockHeaderError {
    /// Internal server error (code 0)
    Internal,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    Unknown(u8),
}

impl From<u8> for GetBlockHeaderError {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Internal,
            _ => Self::Unknown(code),
        }
    }
}

impl fmt::Display for GetBlockHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}

// GET BLOCK BY NUMBER ERROR
// ================================================================================================

/// Error codes match `miden-node/crates/store/src/errors.rs::GetBlockByNumberError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetBlockByNumberError {
    /// Internal server error (code 0)
    Internal,
    /// Failed to deserialize data
    DeserializationFailed,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    Unknown(u8),
}

impl From<u8> for GetBlockByNumberError {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Internal,
            1 => Self::DeserializationFailed,
            _ => Self::Unknown(code),
        }
    }
}

impl fmt::Display for GetBlockByNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}
