use core::fmt;

// GET NOTES BY ID ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::GetNotesByIdError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetNotesByIdError {
    /// Internal server error (code 0)
    Internal,
    /// Failed to deserialize data
    DeserializationFailed,
    /// Note was not found
    NoteNotFound,
    /// Note is not public
    NoteNotPublic,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
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

impl fmt::Display for GetNotesByIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
            Self::NoteNotFound => write!(f, "note not found"),
            Self::NoteNotPublic => write!(f, "note is not public"),
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}

// GET NOTE SCRIPT BY ROOT ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::GetNoteScriptByRootError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetNoteScriptByRootError {
    /// Internal server error (code 0)
    Internal,
    /// Failed to deserialize data
    DeserializationFailed,
    /// Script was not found
    ScriptNotFound,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
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

impl fmt::Display for GetNoteScriptByRootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
            Self::ScriptNotFound => write!(f, "script not found"),
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}

// CHECK NULLIFIERS ERROR
// ================================================================================================

// Error codes match `miden-node/crates/store/src/errors.rs::CheckNullifiersError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckNullifiersError {
    /// Internal server error (code 0)
    Internal,
    /// Failed to deserialize data
    DeserializationFailed,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
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

impl fmt::Display for CheckNullifiersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}
