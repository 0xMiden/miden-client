//! Transaction-related node RPC errors.

use core::fmt;

/// Errors for the `SubmitProvenTransaction` endpoint.
///
/// Error codes match `miden-node/crates/block-producer/src/errors.rs::AddTransactionError`.
/// The node's `#[derive(GrpcError)]` macro assigns codes based on variant order,
/// skipping `#[grpc(internal)]` variants which all map to code 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AddTransactionError {
    /// Internal server error
    Internal = 0,
    /// One or more input notes have already been consumed
    InputNotesAlreadyConsumed = 1,
    /// Unauthenticated notes were not found in the store
    UnauthenticatedNotesNotFound = 2,
    /// One or more output notes already exist in the store
    OutputNotesAlreadyExist = 3,
    /// Account's initial commitment doesn't match the current state
    IncorrectAccountInitialCommitment = 4,
    /// Transaction proof verification failed
    InvalidTransactionProof = 5,
    /// Failed to deserialize the transaction
    TransactionDeserializationFailed = 6,
    /// Transaction has expired
    Expired = 7,
    /// Block producer capacity exceeded
    CapacityExceeded = 8,
}

impl From<u8> for AddTransactionError {
    fn from(code: u8) -> Self {
        match code {
            1 => Self::InputNotesAlreadyConsumed,
            2 => Self::UnauthenticatedNotesNotFound,
            3 => Self::OutputNotesAlreadyExist,
            4 => Self::IncorrectAccountInitialCommitment,
            5 => Self::InvalidTransactionProof,
            6 => Self::TransactionDeserializationFailed,
            7 => Self::Expired,
            8 => Self::CapacityExceeded,
            _ => Self::Internal,
        }
    }
}

impl fmt::Display for AddTransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal => write!(f, "internal server error"),
            Self::InputNotesAlreadyConsumed => write!(f, "input notes already consumed"),
            Self::UnauthenticatedNotesNotFound => write!(f, "unauthenticated notes not found"),
            Self::OutputNotesAlreadyExist => write!(f, "output notes already exist"),
            Self::IncorrectAccountInitialCommitment => {
                write!(f, "incorrect account initial commitment")
            },
            Self::InvalidTransactionProof => write!(f, "invalid transaction proof"),
            Self::TransactionDeserializationFailed => {
                write!(f, "failed to deserialize transaction")
            },
            Self::Expired => write!(f, "transaction expired"),
            Self::CapacityExceeded => write!(f, "block producer capacity exceeded"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_transaction_error_codes() {
        // Verify codes match node's macro-generated values
        assert_eq!(AddTransactionError::Internal as u8, 0);
        assert_eq!(AddTransactionError::InputNotesAlreadyConsumed as u8, 1);
        assert_eq!(AddTransactionError::UnauthenticatedNotesNotFound as u8, 2);
        assert_eq!(AddTransactionError::OutputNotesAlreadyExist as u8, 3);
        assert_eq!(AddTransactionError::IncorrectAccountInitialCommitment as u8, 4);
        assert_eq!(AddTransactionError::InvalidTransactionProof as u8, 5);
        assert_eq!(AddTransactionError::TransactionDeserializationFailed as u8, 6);
        assert_eq!(AddTransactionError::Expired as u8, 7);
        assert_eq!(AddTransactionError::CapacityExceeded as u8, 8);
    }

    #[test]
    fn test_add_transaction_error_from_code() {
        assert_eq!(AddTransactionError::from(0), AddTransactionError::Internal);
        assert_eq!(AddTransactionError::from(1), AddTransactionError::InputNotesAlreadyConsumed);
        assert_eq!(AddTransactionError::from(2), AddTransactionError::UnauthenticatedNotesNotFound);
        assert_eq!(AddTransactionError::from(3), AddTransactionError::OutputNotesAlreadyExist);
        assert_eq!(
            AddTransactionError::from(4),
            AddTransactionError::IncorrectAccountInitialCommitment
        );
        assert_eq!(AddTransactionError::from(5), AddTransactionError::InvalidTransactionProof);
        assert_eq!(
            AddTransactionError::from(6),
            AddTransactionError::TransactionDeserializationFailed
        );
        assert_eq!(AddTransactionError::from(7), AddTransactionError::Expired);
        assert_eq!(AddTransactionError::from(8), AddTransactionError::CapacityExceeded);
        // Unknown codes map to Internal
        assert_eq!(AddTransactionError::from(99), AddTransactionError::Internal);
        assert_eq!(AddTransactionError::from(255), AddTransactionError::Internal);
    }
}
