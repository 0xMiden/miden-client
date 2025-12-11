use std::time::Duration;

/// Configuration for the background client service.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Interval at which the client should attempt to sync with the chain.
    pub sync_interval: Duration,
    /// Whether the service should automatically sync on the configured interval.
    pub auto_sync: bool,
    /// Capacity for the internal command channel.
    pub command_buffer: usize,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            sync_interval: Duration::from_secs(5),
            auto_sync: true,
            command_buffer: 64,
        }
    }
}
