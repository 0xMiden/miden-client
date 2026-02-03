use core::fmt;

/// Error codes match `miden-node/crates/block-producer/src/errors.rs::AddTransactionError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddTransactionError {
    /// Internal server error (code 0)
    Internal,
    /// One or more input notes have already been consumed
    InputNotesAlreadyConsumed,
    /// Unauthenticated notes were not found in the store
    UnauthenticatedNotesNotFound,
    /// One or more output notes already exist in the store
    OutputNotesAlreadyExist,
    /// Account's initial commitment doesn't match the current state
    IncorrectAccountInitialCommitment,
    /// Transaction proof verification failed
    InvalidTransactionProof,
    /// Failed to deserialize the transaction
    TransactionDeserializationFailed,
    /// Transaction has expired
    Expired,
    /// Block producer capacity exceeded
    CapacityExceeded,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    Unknown(u8),
}

impl From<u8> for AddTransactionError {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Internal,
            1 => Self::InputNotesAlreadyConsumed,
            2 => Self::UnauthenticatedNotesNotFound,
            3 => Self::OutputNotesAlreadyExist,
            4 => Self::IncorrectAccountInitialCommitment,
            5 => Self::InvalidTransactionProof,
            6 => Self::TransactionDeserializationFailed,
            7 => Self::Expired,
            8 => Self::CapacityExceeded,
            _ => Self::Unknown(code),
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
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}
