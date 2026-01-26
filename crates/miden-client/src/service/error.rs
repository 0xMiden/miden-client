use anyhow::Error as AnyhowError;
use miden_client_core::ClientError;
use thiserror::Error;

/// Error type produced by service operations.
#[derive(Debug, Error)]
pub enum ClientServiceError {
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error(transparent)]
    Handler(#[from] HandlerError),
    #[error("service is shutting down")]
    ShuttingDown,
}

/// Error returned by sync handlers.
#[derive(Debug, Error)]
pub enum HandlerError {
    #[error(transparent)]
    Other(#[from] AnyhowError),
}

impl HandlerError {
    #[must_use]
    pub fn new(err: impl Into<anyhow::Error>) -> Self {
        Self::Other(err.into())
    }
}
