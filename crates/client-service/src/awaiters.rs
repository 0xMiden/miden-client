//! Pre-registered event awaiters.
//!
//! Unlike the broadcast channel used by [`subscribe`](crate::ClientService::subscribe),
//! awaiters are fulfilled synchronously during event emission and cannot be
//! lost to lag. This makes them suitable for RPC-style "wait for a specific
//! event" patterns where missing the event would be a correctness bug.

use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll};

use tokio::sync::oneshot;

use crate::events::ClientEvent;
use crate::handlers::EventFilter;

/// Registry of pending awaiters. Consulted in-line by `emit_events` before
/// the broadcast send so awaited events cannot be dropped by a slow broadcast
/// consumer.
pub(crate) struct AwaiterRegistry {
    inner: Mutex<Vec<Awaiter>>,
}

struct Awaiter {
    filter: EventFilter,
    tx: oneshot::Sender<ClientEvent>,
}

impl AwaiterRegistry {
    pub(crate) fn new() -> Self {
        Self { inner: Mutex::new(Vec::new()) }
    }

    /// Registers an awaiter for the next event matching `filter`. Returns a
    /// future that resolves to the matching event, or [`AwaiterCancelled`] if
    /// the service is dropped before a match arrives.
    pub(crate) fn register(&self, filter: EventFilter) -> AwaiterFuture {
        let (tx, rx) = oneshot::channel();
        self.inner
            .lock()
            .expect("awaiter registry poisoned")
            .push(Awaiter { filter, tx });
        AwaiterFuture { rx }
    }

    /// Fulfills any awaiters matching the given events. Consumed awaiters are
    /// removed; awaiters whose receivers have been dropped are pruned.
    pub(crate) fn fulfill(&self, events: &[ClientEvent]) {
        if events.is_empty() {
            return;
        }
        let mut inner = self.inner.lock().expect("awaiter registry poisoned");
        let old = std::mem::take(&mut *inner);
        for awaiter in old {
            if awaiter.tx.is_closed() {
                continue; // receiver dropped — prune
            }
            match events.iter().find(|e| awaiter.filter.matches(e)) {
                Some(event) => {
                    // Fulfilled — consume the awaiter.
                    let _ = awaiter.tx.send(event.clone());
                },
                None => {
                    // No match this round — keep waiting.
                    inner.push(awaiter);
                },
            }
        }
    }
}

/// Future returned by [`ClientService::expect`](crate::ClientService::expect)
/// and used internally by [`ClientService::once`](crate::ClientService::once).
///
/// Resolves with the first matching [`ClientEvent`]. Dropping the future
/// cancels the awaiter; the registry prunes stale entries on the next
/// emission.
pub struct AwaiterFuture {
    rx: oneshot::Receiver<ClientEvent>,
}

impl Future for AwaiterFuture {
    type Output = Result<ClientEvent, AwaiterCancelled>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map_err(|_| AwaiterCancelled)
    }
}

/// Error returned when an awaiter is cancelled because the service was
/// dropped before a matching event arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AwaiterCancelled;

impl core::fmt::Display for AwaiterCancelled {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "awaiter cancelled: service was dropped before event arrived")
    }
}

impl std::error::Error for AwaiterCancelled {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{test_note_arc, test_note_id};

    #[tokio::test]
    async fn fulfills_matching_event() {
        let registry = AwaiterRegistry::new();
        let note_id = test_note_id();
        let awaiter = registry.register(EventFilter::NoteCommitted(note_id));

        registry.fulfill(&[ClientEvent::NoteCommitted { note_id }]);

        let event = awaiter.await.unwrap();
        assert_eq!(event.note_id(), Some(note_id));
    }

    #[tokio::test]
    async fn ignores_non_matching_events() {
        let registry = AwaiterRegistry::new();
        let note_id = test_note_id();
        let awaiter = registry.register(EventFilter::NoteCommitted(note_id));

        registry.fulfill(&[ClientEvent::NoteReceived { note: test_note_arc() }]);

        let result = tokio::time::timeout(std::time::Duration::from_millis(50), awaiter).await;
        assert!(result.is_err(), "awaiter should not have resolved");
    }

    #[tokio::test]
    async fn dropped_awaiter_is_pruned() {
        let registry = AwaiterRegistry::new();
        let note_id = test_note_id();
        {
            let _awaiter = registry.register(EventFilter::NoteCommitted(note_id));
            // Awaiter dropped at end of block.
        }
        // Pruning happens opportunistically during fulfill.
        registry.fulfill(&[ClientEvent::NoteCommitted { note_id }]);
        assert!(registry.inner.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn only_first_match_wins() {
        let registry = AwaiterRegistry::new();
        let note_id = test_note_id();
        let awaiter = registry.register(EventFilter::AnyNoteCommitted);

        registry.fulfill(&[
            ClientEvent::NoteCommitted { note_id },
            ClientEvent::NoteCommitted { note_id },
        ]);

        let event = awaiter.await.unwrap();
        assert!(matches!(event, ClientEvent::NoteCommitted { .. }));
        // Second call should find no awaiter.
        assert!(registry.inner.lock().unwrap().is_empty());
    }
}
