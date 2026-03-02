//! Configuration options for the client service.

use std::time::Duration;

/// Configuration options for [`ClientService`](crate::ClientService).
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Interval between automatic background sync operations.
    ///
    /// Set to `None` to disable automatic background sync.
    /// Default: 30 seconds.
    pub sync_interval: Option<Duration>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            sync_interval: Some(Duration::from_secs(30)),
        }
    }
}

impl ServiceConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the sync interval.
    #[must_use]
    pub fn with_sync_interval(mut self, interval: Option<Duration>) -> Self {
        self.sync_interval = interval;
        self
    }

    /// Disables automatic background sync.
    #[must_use]
    pub fn without_background_sync(mut self) -> Self {
        self.sync_interval = None;
        self
    }
}
