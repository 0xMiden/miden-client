// GET BLOCK HEADER ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::GetBlockHeaderError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GetBlockHeaderError {
    /// Internal server error (code 0)
    #[error("internal server error")]
    Internal,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    #[error("unknown error (code {0})")]
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

// GET BLOCK BY NUMBER ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::GetBlockByNumberError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GetBlockByNumberError {
    /// Internal server error (code 0)
    #[error("internal server error")]
    Internal,
    /// Failed to deserialize data
    #[error("deserialization failed")]
    DeserializationFailed,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    #[error("unknown error (code {0})")]
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
