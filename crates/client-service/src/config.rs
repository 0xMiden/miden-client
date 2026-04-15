//! Configuration options for the client service.

use core::num::NonZeroU32;
use std::time::Duration;

/// Default interval between automatic background sync operations.
pub const DEFAULT_SYNC_INTERVAL: Duration = Duration::from_secs(5);

/// Configuration options for [`ClientService`](crate::ClientService).
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Interval between automatic background sync operations.
    ///
    /// Set to `None` to disable automatic background sync.
    /// Default: [`DEFAULT_SYNC_INTERVAL`].
    pub sync_interval: Option<Duration>,

    /// Whether `start_background_sync` should trigger an immediate sync before
    /// waiting for the first interval tick.
    ///
    /// Default: `true`.
    pub sync_on_start: bool,

    /// If set, each sync advances at most this many blocks past the current
    /// sync height. When `None`, every sync runs to the current chain tip.
    ///
    /// Useful for catch-up scenarios: a client that has been offline for
    /// thousands of blocks would otherwise emit a massive burst of events
    /// (one per note/transaction/account across the whole range) and hold the
    /// client mutex for the full duration of a single huge sync. Chunking
    /// breaks that into bounded steps.
    ///
    /// Default: `None`.
    pub sync_chunk_blocks: Option<NonZeroU32>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            sync_interval: Some(DEFAULT_SYNC_INTERVAL),
            sync_on_start: true,
            sync_chunk_blocks: None,
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

    /// Controls whether background sync triggers an immediate sync on start.
    #[must_use]
    pub fn with_sync_on_start(mut self, sync_on_start: bool) -> Self {
        self.sync_on_start = sync_on_start;
        self
    }

    /// Configures chunked sync. When set, each sync advances at most `n` blocks
    /// past the current sync height; `None` disables chunking.
    #[must_use]
    pub fn with_sync_chunk_blocks(mut self, n: Option<NonZeroU32>) -> Self {
        self.sync_chunk_blocks = n;
        self
    }
}
