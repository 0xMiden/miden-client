//! Service-specific error types.

use miden_client::ClientError;
use thiserror::Error;

/// Errors that can occur during service operations.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// An error occurred in the underlying client.
    #[error("client error: {0}")]
    ClientError(#[from] ClientError),

    /// The service has already been shut down.
    #[error("service has been shut down")]
    ServiceShutdown,

    /// A sync operation is already in progress and couldn't be joined.
    #[error("sync operation failed to complete")]
    SyncFailed,

    /// Failed to acquire a coordination lock.
    #[error("failed to acquire coordination lock")]
    LockAcquisitionFailed,

    /// An event handler returned an error.
    #[error("event handler error: {0}")]
    HandlerError(String),

    /// Background sync task failed.
    #[error("background sync task failed: {0}")]
    BackgroundSyncFailed(String),
}
