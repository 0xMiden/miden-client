//! Sequential transaction submission queue.
//!
//! [`ClientService`](crate::ClientService) serializes mutations behind a single client mutex.
//! For most callers this is fine — you just `await` `submit_transaction` and let the mutex
//! queue you in FIFO order. When a caller wants **fire-and-forget** semantics — "take this
//! request, give me back a handle, process it whenever the client is free" — that's what the
//! transaction queue is for.
//!
//! A worker task owns an unbounded mpsc of pending submissions. Each enqueue returns an
//! [`EnqueuedTx`] future that resolves once the worker has submitted the transaction (or
//! failed). Dropping the [`TransactionQueueHandle`] closes the channel and the worker exits
//! after draining; in-flight submissions complete, not-yet-started submissions resolve with
//! [`EnqueueError::QueueShutDown`].
//!
//! ## Example
//!
//! ```rust,ignore
//! let queue = service.start_transaction_queue();
//!
//! // React to events by enqueueing a transaction — handler returns immediately.
//! let svc = Arc::clone(&service);
//! let q = queue.clone();
//! svc.on(EventFilter::AnyNoteCommitted, move |_event, service| {
//!     let q = q.clone();
//!     async move {
//!         let client = service.client().await;
//!         let req = build_some_request(&*client).await.unwrap();
//!         let account_id = client.account_id_to_act_on();
//!         drop(client);
//!
//!         let handle = q.enqueue(account_id, req);
//!         // Fire-and-forget: drop the handle, or tokio::spawn to log the result.
//!         tokio::spawn(async move {
//!             match handle.await {
//!                 Ok(tx_id) => tracing::info!(?tx_id, "tx submitted"),
//!                 Err(e) => tracing::warn!(error = %e, "tx failed"),
//!             }
//!         });
//!     }
//! });
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use miden_client::ClientError;
use miden_client::auth::TransactionAuthenticator;
use miden_client::transaction::{TransactionId, TransactionRequest};
use miden_protocol::account::AccountId;
use tokio::sync::{mpsc, oneshot};
use tracing::debug;

use crate::ClientService;

/// Internal envelope carrying a submission request to the worker.
struct QueuedTx {
    account_id: AccountId,
    request: TransactionRequest,
    result: oneshot::Sender<Result<TransactionId, ClientError>>,
}

/// Handle to a running transaction queue.
///
/// Cheap to clone — multiple producers can share one queue. The worker shuts down once the
/// last handle is dropped (and the mpsc channel closes).
#[derive(Clone)]
pub struct TransactionQueueHandle {
    sender: mpsc::UnboundedSender<QueuedTx>,
}

impl TransactionQueueHandle {
    /// Enqueues a transaction for sequential submission. Returns a future resolving to the
    /// submission result.
    ///
    /// Dropping the returned [`EnqueuedTx`] before it resolves is safe — the submission still
    /// runs, the result is just discarded. (Fire-and-forget mode.)
    pub fn enqueue(&self, account_id: AccountId, request: TransactionRequest) -> EnqueuedTx {
        let (result_tx, result_rx) = oneshot::channel();
        match self.sender.send(QueuedTx { account_id, request, result: result_tx }) {
            Ok(()) => EnqueuedTx { rx: Some(result_rx) },
            // Queue already shut down; the future will resolve synchronously to an error.
            Err(_) => EnqueuedTx { rx: None },
        }
    }

    /// Returns `true` if the worker is still running (the channel has not been closed).
    pub fn is_active(&self) -> bool {
        !self.sender.is_closed()
    }
}

/// Future returned by [`TransactionQueueHandle::enqueue`].
///
/// Resolves to the submitted [`TransactionId`] on success, or an [`EnqueueError`] on failure.
pub struct EnqueuedTx {
    rx: Option<oneshot::Receiver<Result<TransactionId, ClientError>>>,
}

impl Future for EnqueuedTx {
    type Output = Result<TransactionId, EnqueueError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Some(rx) = self.rx.as_mut() else {
            return Poll::Ready(Err(EnqueueError::QueueShutDown));
        };
        match Pin::new(rx).poll(cx) {
            Poll::Ready(Ok(Ok(tx_id))) => Poll::Ready(Ok(tx_id)),
            Poll::Ready(Ok(Err(e))) => Poll::Ready(Err(EnqueueError::Submission(e))),
            // Sender dropped without sending — worker exited before processing.
            Poll::Ready(Err(_)) => Poll::Ready(Err(EnqueueError::QueueShutDown)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Error returned by [`EnqueuedTx`].
#[derive(Debug)]
pub enum EnqueueError {
    /// The queue was shut down before the transaction was submitted. Either the
    /// [`TransactionQueueHandle`] was dropped, or the service was dropped.
    QueueShutDown,
    /// Submission reached the network layer and failed.
    Submission(ClientError),
}

impl core::fmt::Display for EnqueueError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::QueueShutDown => {
                write!(f, "transaction queue was shut down before submission")
            },
            Self::Submission(_) => write!(f, "transaction submission failed"),
        }
    }
}

impl std::error::Error for EnqueueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::QueueShutDown => None,
            Self::Submission(e) => Some(e),
        }
    }
}

impl From<ClientError> for EnqueueError {
    fn from(value: ClientError) -> Self {
        Self::Submission(value)
    }
}

impl<AUTH> ClientService<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    /// Starts a worker task that processes enqueued transactions sequentially.
    ///
    /// Transactions submitted via the returned handle are processed in FIFO order: each waits
    /// for the previous to complete. The worker calls
    /// [`submit_transaction`](Self::submit_transaction) internally, so queued transactions
    /// share the same mutex as [`sync_state`](Self::sync_state) and direct client access —
    /// they don't bypass coordination, they just free the caller from `await`ing synchronously.
    ///
    /// Returns a [`TransactionQueueHandle`] that can be cloned and shared. The worker shuts
    /// down when all handles are dropped.
    pub fn start_transaction_queue(self: &Arc<Self>) -> TransactionQueueHandle {
        let (sender, mut receiver) = mpsc::unbounded_channel::<QueuedTx>();
        let service = Arc::clone(self);

        tokio::spawn(async move {
            debug!("Transaction queue worker starting");
            while let Some(queued) = receiver.recv().await {
                let result = service.submit_transaction(queued.account_id, queued.request).await;
                // Result receiver may have been dropped (fire-and-forget caller) — ignore.
                let _ = queued.result.send(result);
            }
            debug!("Transaction queue worker exiting");
        });

        TransactionQueueHandle { sender }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn enqueue_on_dropped_handle_resolves_to_shutdown_error() {
        // Simulate a handle that has no live worker by closing the channel.
        let (sender, receiver) = mpsc::unbounded_channel::<QueuedTx>();
        drop(receiver);
        let handle = TransactionQueueHandle { sender };

        assert!(!handle.is_active());

        // We can't easily construct a TransactionRequest in a unit test (it needs a real
        // client + note), but we can verify the error path by constructing an EnqueuedTx
        // whose rx is `None` directly.
        let (dropped_tx, dropped_rx) = oneshot::channel::<Result<TransactionId, ClientError>>();
        drop(dropped_tx);
        let future = EnqueuedTx { rx: Some(dropped_rx) };
        let err = future.await.expect_err("should fail when sender drops");
        assert!(matches!(err, EnqueueError::QueueShutDown));
    }

    #[tokio::test]
    async fn enqueue_on_never_connected_handle_is_immediate_error() {
        let future = EnqueuedTx { rx: None };
        let err = future.await.expect_err("None rx should immediately error");
        assert!(matches!(err, EnqueueError::QueueShutDown));
    }
}
