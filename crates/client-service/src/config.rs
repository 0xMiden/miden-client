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

    /// Whether to emit events during sync operations.
    ///
    /// Default: true.
    pub emit_sync_events: bool,

    /// Whether to emit events during transaction operations.
    ///
    /// Default: true.
    pub emit_transaction_events: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            sync_interval: Some(Duration::from_secs(30)),
            emit_sync_events: true,
            emit_transaction_events: true,
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

    /// Sets whether to emit sync events.
    #[must_use]
    pub fn with_sync_events(mut self, emit: bool) -> Self {
        self.emit_sync_events = emit;
        self
    }

    /// Sets whether to emit transaction events.
    #[must_use]
    pub fn with_transaction_events(mut self, emit: bool) -> Self {
        self.emit_transaction_events = emit;
        self
    }
}
