//! Note-related node RPC errors.

use core::fmt;

// GET NOTES BY ID ERROR
// ================================================================================================

/// Errors for the `GetNotesById` endpoint.
///
/// Error codes match `miden-node/crates/store/src/errors.rs::GetNotesByIdError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GetNotesByIdError {
    /// Internal server error
    Internal = 0,
    /// Failed to deserialize data
    DeserializationFailed = 1,
    /// Note was not found
    NoteNotFound = 2,
    /// Note is not public
    NoteNotPublic = 3,
}

impl From<u8> for GetNotesByIdError {
    fn from(code: u8) -> Self {
        match code {
            1 => Self::DeserializationFailed,
            2 => Self::NoteNotFound,
            3 => Self::NoteNotPublic,
            _ => Self::Internal,
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
        }
    }
}

// GET NOTE SCRIPT BY ROOT ERROR
// ================================================================================================

/// Errors for the `GetNoteScriptByRoot` endpoint.
///
/// Error codes match `miden-node/crates/store/src/errors.rs::GetNoteScriptByRootError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GetNoteScriptByRootError {
    /// Internal server error
    Internal = 0,
    /// Failed to deserialize data
    DeserializationFailed = 1,
    /// Script was not found
    ScriptNotFound = 2,
}

impl From<u8> for GetNoteScriptByRootError {
    fn from(code: u8) -> Self {
        match code {
            1 => Self::DeserializationFailed,
            2 => Self::ScriptNotFound,
            _ => Self::Internal,
        }
    }
}

impl fmt::Display for GetNoteScriptByRootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
            Self::ScriptNotFound => write!(f, "script not found"),
        }
    }
}

// CHECK NULLIFIERS ERROR
// ================================================================================================

/// Errors for the `CheckNullifiers` endpoint.
///
/// Error codes match `miden-node/crates/store/src/errors.rs::CheckNullifiersError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CheckNullifiersError {
    /// Internal server error
    Internal = 0,
    /// Failed to deserialize data
    DeserializationFailed = 1,
}

impl From<u8> for CheckNullifiersError {
    fn from(code: u8) -> Self {
        match code {
            1 => Self::DeserializationFailed,
            _ => Self::Internal,
        }
    }
}

impl fmt::Display for CheckNullifiersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
        }
    }
}
