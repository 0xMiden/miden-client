// GET NOTES BY ID ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::GetNotesByIdError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GetNotesByIdError {
    /// Internal server error (code 0)
    #[error("internal server error")]
    Internal,
    /// Failed to deserialize data
    #[error("deserialization failed")]
    DeserializationFailed,
    /// Note was not found
    #[error("note not found")]
    NoteNotFound,
    /// Note is not public
    #[error("note is not public")]
    NoteNotPublic,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    #[error("unknown error (code {0})")]
    Unknown(u8),
}

impl From<u8> for GetNotesByIdError {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Internal,
            1 => Self::DeserializationFailed,
            2 => Self::NoteNotFound,
            3 => Self::NoteNotPublic,
            _ => Self::Unknown(code),
        }
    }
}

// GET NOTE SCRIPT BY ROOT ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::GetNoteScriptByRootError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GetNoteScriptByRootError {
    /// Internal server error (code 0)
    #[error("internal server error")]
    Internal,
    /// Failed to deserialize data
    #[error("deserialization failed")]
    DeserializationFailed,
    /// Script was not found
    #[error("script not found")]
    ScriptNotFound,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    #[error("unknown error (code {0})")]
    Unknown(u8),
}

impl From<u8> for GetNoteScriptByRootError {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Internal,
            1 => Self::DeserializationFailed,
            2 => Self::ScriptNotFound,
            _ => Self::Unknown(code),
        }
    }
}

// CHECK NULLIFIERS ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::CheckNullifiersError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CheckNullifiersError {
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

impl From<u8> for CheckNullifiersError {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Internal,
            1 => Self::DeserializationFailed,
            _ => Self::Unknown(code),
        }
    }
}
