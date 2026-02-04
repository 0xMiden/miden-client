// Error codes match `miden-node/crates/block-producer/src/errors.rs::AddTransactionError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AddTransactionError {
    /// Internal server error (code 0)
    #[error("internal server error")]
    Internal,
    /// One or more input notes have already been consumed
    #[error("input notes already consumed")]
    InputNotesAlreadyConsumed,
    /// Unauthenticated notes were not found in the store
    #[error("unauthenticated notes not found")]
    UnauthenticatedNotesNotFound,
    /// One or more output notes already exist in the store
    #[error("output notes already exist")]
    OutputNotesAlreadyExist,
    /// Account's initial commitment doesn't match the current state
    #[error("incorrect account initial commitment")]
    IncorrectAccountInitialCommitment,
    /// Transaction proof verification failed
    #[error("invalid transaction proof")]
    InvalidTransactionProof,
    /// Failed to deserialize the transaction
    #[error("failed to deserialize transaction")]
    TransactionDeserializationFailed,
    /// Transaction has expired
    #[error("transaction expired")]
    Expired,
    /// Block producer capacity exceeded
    #[error("block producer capacity exceeded")]
    CapacityExceeded,
    /// Error code not recognized by this client version. This can happen if the node
    /// is newer than the client and has added new error variants.
    #[error("unknown error (code {0})")]
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
