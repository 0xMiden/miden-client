use std::time::Duration;

use miden_client_core::ClientError;
use tokio::task::JoinError;

/// Configuration options for the [`ClientService`](crate::service::handle::ClientService).
#[derive(Clone, Debug)]
pub struct ClientServiceConfig {
    /// Period for the background sync loop. Set to `None` to disable automatic sync.
    pub sync_interval: Option<Duration>,
    /// Whether to perform an initial sync as soon as the service starts.
    pub initial_sync: bool,
    /// Capacity of the command channel feeding the coordinator.
    pub command_buffer: usize,
    /// Capacity of the proven-transaction channel between prover workers and the coordinator.
    pub proof_buffer: usize,
    /// Maximum number of transaction proving tasks that may run in parallel.
    pub max_parallel_proofs: usize,
}

impl Default for ClientServiceConfig {
    fn default() -> Self {
        Self {
            sync_interval: Some(Duration::from_secs(15)),
            initial_sync: true,
            command_buffer: 64,
            proof_buffer: 64,
            max_parallel_proofs: 2,
        }
    }
}

/// Errors returned by [`ClientServiceHandle`](crate::service::handle::ClientServiceHandle) APIs.
#[derive(thiserror::Error, Debug)]
pub enum ClientServiceError {
    #[error("client service is not running")]
    ServiceClosed,
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error(transparent)]
    Join(#[from] JoinError),
}
