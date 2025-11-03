use std::time::Duration;

/// Configuration options for the [`ClientRuntime`](crate::service::ClientRuntime).
#[derive(Debug, Clone)]
pub struct ClientServiceConfig {
    /// Interval at which the background sync loop will poll the chain.
    /// If `None`, the service will only sync when explicitly triggered.
    pub sync_interval: Option<Duration>,
    /// Whether an initial sync should be performed before the background task starts looping.
    pub initial_sync: bool,
}

impl Default for ClientServiceConfig {
    fn default() -> Self {
        Self {
            sync_interval: Some(Duration::from_secs(5)),
            initial_sync: true,
        }
    }
}
