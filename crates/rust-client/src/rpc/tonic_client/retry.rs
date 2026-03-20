use core::time::Duration;

use tonic::Status;
use tracing::warn;

// CONSTS
// ================================================================================================

// TODO: Make these configurable at the tonic client level

/// Maximum number of retry attempts for rate-limited requests.
const MAX_RETRIES: u32 = 5;

/// Fallback delay when no `retry-after` header is present.
const FALLBACK_RETRY_DELAY_MS: u64 = 250;

// RETRY STATE
// ================================================================================================

/// Tracks retry attempts for a single RPC call and applies the node-provided cooldown policy.
///
/// The state is intentionally tiny: it only counts how many retries have already been attempted.
/// Delay selection is derived from the current gRPC [`Status`], preferring the `retry-after`
/// response metadata when present and falling back to [`FALLBACK_RETRY_DELAY_MS`] otherwise.
pub(super) struct RetryState {
    attempt: u32,
}

impl RetryState {
    /// Creates a new retry state for a fresh RPC call.
    pub(super) const fn new() -> Self {
        Self { attempt: 0 }
    }

    /// Applies retry policy for the provided status.
    ///
    /// Returns `true` after waiting the requested cooldown when the error is retryable and the
    /// attempt limit has not been reached. Returns `false` for non-retryable statuses or once the
    /// retry budget is exhausted.
    pub(super) async fn should_retry(&mut self, status: &Status) -> bool {
        if self.attempt >= MAX_RETRIES || !is_retryable(status) {
            return false;
        }

        let delay =
            extract_retry_after(status).unwrap_or(Duration::from_millis(FALLBACK_RETRY_DELAY_MS));

        warn!(
            attempt = self.attempt + 1,
            delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
            "rate-limited by node, retrying after delay",
        );

        async_sleep(delay).await;
        self.attempt += 1;
        true
    }
}

// HELPERS
// ================================================================================================

fn is_retryable(status: &Status) -> bool {
    matches!(status.code(), tonic::Code::ResourceExhausted | tonic::Code::Unavailable)
}

fn extract_retry_after(status: &Status) -> Option<Duration> {
    status
        .metadata()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
}

#[cfg(not(target_arch = "wasm32"))]
async fn async_sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

/// On WASM, sleep using browser timers so retry delays are honored.
#[cfg(target_arch = "wasm32")]
async fn async_sleep(duration: Duration) {
    gloo_timers::future::sleep(duration).await;
}
