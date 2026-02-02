//! Block-related node RPC errors.

use core::fmt;

// GET BLOCK HEADER ERROR
// ================================================================================================

/// Errors for the `GetBlockHeaderByNumber` endpoint.
///
/// Error codes match `miden-node/crates/store/src/errors.rs::GetBlockHeaderError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GetBlockHeaderError {
    /// Internal server error
    Internal = 0,
}

impl From<u8> for GetBlockHeaderError {
    fn from(_code: u8) -> Self {
        // This error type only has Internal, all codes map to it
        Self::Internal
    }
}

impl fmt::Display for GetBlockHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
        }
    }
}

// GET BLOCK BY NUMBER ERROR
// ================================================================================================

/// Errors for the `GetBlockByNumber` endpoint.
///
/// Error codes match `miden-node/crates/store/src/errors.rs::GetBlockByNumberError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GetBlockByNumberError {
    /// Internal server error
    Internal = 0,
    /// Failed to deserialize data
    DeserializationFailed = 1,
}

impl From<u8> for GetBlockByNumberError {
    fn from(code: u8) -> Self {
        match code {
            1 => Self::DeserializationFailed,
            _ => Self::Internal,
        }
    }
}

impl fmt::Display for GetBlockByNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::DeserializationFailed => write!(f, "deserialization failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_block_header_error_codes() {
        assert_eq!(GetBlockHeaderError::Internal as u8, 0);
    }

    #[test]
    fn test_get_block_by_number_error_codes() {
        assert_eq!(GetBlockByNumberError::Internal as u8, 0);
        assert_eq!(GetBlockByNumberError::DeserializationFailed as u8, 1);
    }

    #[test]
    fn test_get_block_header_error_from_code() {
        assert_eq!(GetBlockHeaderError::from(0), GetBlockHeaderError::Internal);
        assert_eq!(GetBlockHeaderError::from(1), GetBlockHeaderError::Internal);
        assert_eq!(GetBlockHeaderError::from(99), GetBlockHeaderError::Internal);
    }

    #[test]
    fn test_get_block_by_number_error_from_code() {
        assert_eq!(GetBlockByNumberError::from(0), GetBlockByNumberError::Internal);
        assert_eq!(GetBlockByNumberError::from(1), GetBlockByNumberError::DeserializationFailed);
        assert_eq!(GetBlockByNumberError::from(99), GetBlockByNumberError::Internal);
    }
}
