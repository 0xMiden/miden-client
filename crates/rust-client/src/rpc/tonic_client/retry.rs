use core::time::Duration;

use tonic::Status;
use tracing::warn;

/// Maximum number of retry attempts for rate-limited requests.
const MAX_RETRIES: u32 = 5;

/// Fallback delay when no `retry-after` header is present.
const FALLBACK_RETRY_DELAY_MS: u64 = 500;

/// Minimum retry delay even when `retry-after: 0` is specified,
/// to avoid a tight retry loop.
const MIN_RETRY_DELAY_MS: u64 = 100;

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

/// If the status is retryable and we haven't exhausted attempts, sleeps and
/// returns `true` (caller should retry). Otherwise returns `false`.
pub(super) async fn should_retry(status: &Status, attempt: u32) -> bool {
    if attempt >= MAX_RETRIES || !is_retryable(status) {
        return false;
    }

    let delay = extract_retry_after(status)
        .unwrap_or(Duration::from_millis(FALLBACK_RETRY_DELAY_MS))
        .max(Duration::from_millis(MIN_RETRY_DELAY_MS));

    warn!(
        attempt = attempt + 1,
        delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
        "rate-limited by node, retrying after delay",
    );

    async_sleep(delay).await;
    true
}

#[cfg(not(target_arch = "wasm32"))]
async fn async_sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

/// On WASM, yield control back to the event loop once.
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
