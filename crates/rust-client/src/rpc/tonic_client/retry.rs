use core::time::Duration;

use tonic::Status;
use tracing::warn;

use super::RpcEndpoint;
use crate::rpc::RpcError;

/// Maximum number of retry attempts for rate-limited requests.
const MAX_RETRIES: u32 = 5;

/// Default retry delay when no `retry-after` header is present.
const DEFAULT_RETRY_DELAY_MS: u64 = 500;

/// Maximum retry delay to cap exponential backoff.
const MAX_RETRY_DELAY_MS: u64 = 10_000;

/// Minimum retry delay even when `retry-after: 0` is specified,
/// to avoid a tight retry loop.
const MIN_RETRY_DELAY_MS: u64 = 100;

/// Returns whether a tonic Status represents a transiently retryable error.
fn is_retryable(status: &Status) -> bool {
    matches!(status.code(), tonic::Code::ResourceExhausted | tonic::Code::Unavailable)
}

/// Extracts the retry-after delay from a tonic Status's metadata.
///
/// The gRPC response may include a `retry-after` header with the number of
/// seconds to wait before retrying.
fn extract_retry_after(status: &Status) -> Option<Duration> {
    status
        .metadata()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
}

/// Computes the retry delay for the given attempt number.
///
/// Uses the `retry-after` header if present, otherwise falls back to
/// exponential backoff starting at [`DEFAULT_RETRY_DELAY_MS`].
fn compute_retry_delay(status: &Status, attempt: u32) -> Duration {
    let delay = extract_retry_after(status)
        .unwrap_or_else(|| Duration::from_millis(DEFAULT_RETRY_DELAY_MS * 2u64.pow(attempt)));

    delay
        .max(Duration::from_millis(MIN_RETRY_DELAY_MS))
        .min(Duration::from_millis(MAX_RETRY_DELAY_MS))
}

#[cfg(not(target_arch = "wasm32"))]
async fn async_sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

/// On WASM, yield control back to the event loop once. A proper timer-based
/// sleep would require an extra dependency; yielding is sufficient because
/// WASM is single-threaded and rate-limiting is rare in that context.
#[cfg(target_arch = "wasm32")]
async fn async_sleep(_duration: Duration) {
    let mut yielded = false;
    futures::future::poll_fn(|cx| {
        if yielded {
            core::task::Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        }
    })
    .await;
}

/// Tracks retry state for a single RPC call sequence.
pub(super) struct RetryState {
    endpoint: RpcEndpoint,
    attempt: u32,
}

impl RetryState {
    pub fn new(endpoint: RpcEndpoint) -> Self {
        Self { endpoint, attempt: 0 }
    }

    /// Checks if the given status is retryable and, if so, waits before the
    /// next attempt. Returns `Ok(())` if the caller should retry, or
    /// `Err(RpcError)` if retries are exhausted or the error isn't retryable.
    pub async fn maybe_retry(
        &mut self,
        status: Status,
        grpc_client: &super::GrpcClient,
    ) -> Result<(), RpcError> {
        if self.attempt >= MAX_RETRIES || !is_retryable(&status) {
            return Err(grpc_client.rpc_error_from_status(self.endpoint, status));
        }

        let delay = compute_retry_delay(&status, self.attempt);
        self.attempt += 1;

        warn!(
            endpoint = %self.endpoint,
            attempt = self.attempt,
            delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
            "rate-limited by node, retrying after delay",
        );

        async_sleep(delay).await;
        Ok(())
    }
}
