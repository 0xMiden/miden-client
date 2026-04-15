//! Event handler registration and dispatch.
//!
//! Each registered handler gets its own spawned worker task reading from a
//! bounded mpsc channel. `dispatch` pushes matching events onto the handler's
//! queue without blocking emission; if the queue is full the event is dropped
//! and a warning is logged. This caps the fanout at `num_handlers` tasks and
//! preserves per-handler event ordering.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use miden_client::transaction::TransactionId;
use miden_protocol::account::AccountId;
use miden_protocol::note::NoteId;
use tokio::sync::mpsc;
use tracing::warn;

use crate::events::ClientEvent;

/// Bounded capacity of each handler's event queue.
const HANDLER_QUEUE_CAPACITY: usize = 256;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub(crate) struct StoredHandler(Arc<dyn Fn(ClientEvent) -> BoxFuture + Send + Sync>);

impl StoredHandler {
    pub(crate) fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(ClientEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self(Arc::new(move |event| Box::pin(f(event))))
    }

    fn call(&self, event: ClientEvent) -> BoxFuture {
        (self.0)(event)
    }
}

/// Opaque identifier for a registered handler. Used with
/// [`ClientService::remove_handler`](crate::ClientService::remove_handler).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandlerId(u64);

/// Describes which events a handler or awaiter is interested in.
///
/// Used with [`ClientService::on`](crate::ClientService::on),
/// [`ClientService::once`](crate::ClientService::once), and
/// [`ClientService::expect`](crate::ClientService::expect).
#[derive(Debug, Clone)]
pub enum EventFilter {
    /// Match a specific note being received.
    NoteReceived(NoteId),
    /// Match any note received.
    AnyNoteReceived,
    /// Match a specific note being committed.
    NoteCommitted(NoteId),
    /// Match any note committed.
    AnyNoteCommitted,
    /// Match any note consumed.
    AnyNoteConsumed,
    /// Match a specific transaction committed.
    TransactionCommitted(TransactionId),
    /// Match any transaction committed.
    AnyTransactionCommitted,
    /// Match a specific account updated.
    AccountUpdated(AccountId),
    /// Match any account updated.
    AnyAccountUpdated,
    /// Match any sync completed.
    AnySyncCompleted,
}

impl EventFilter {
    pub(crate) fn matches(&self, event: &ClientEvent) -> bool {
        match (self, event) {
            (Self::NoteReceived(id), ClientEvent::NoteReceived { note }) => *id == note.id(),
            (Self::NoteCommitted(id), ClientEvent::NoteCommitted { note_id }) => id == note_id,
            (Self::AccountUpdated(id), ClientEvent::AccountUpdated { account_id }) => {
                id == account_id
            },
            (
                Self::TransactionCommitted(id),
                ClientEvent::TransactionCommitted { transaction_id },
            ) => id == transaction_id,
            (Self::AnyNoteReceived, ClientEvent::NoteReceived { .. })
            | (Self::AnyNoteCommitted, ClientEvent::NoteCommitted { .. })
            | (Self::AnyNoteConsumed, ClientEvent::NoteConsumed { .. })
            | (Self::AnyTransactionCommitted, ClientEvent::TransactionCommitted { .. })
            | (Self::AnyAccountUpdated, ClientEvent::AccountUpdated { .. })
            | (Self::AnySyncCompleted, ClientEvent::SyncCompleted { .. }) => true,
            _ => false,
        }
    }
}

struct HandlerEntry {
    id: HandlerId,
    filter: EventFilter,
    /// Bounded sender feeding the handler's worker task. Dropping this (via
    /// `unregister` or registry drop) closes the channel and terminates the
    /// worker.
    tx: mpsc::Sender<ClientEvent>,
}

pub(crate) struct HandlerRegistry {
    handlers: RwLock<Vec<HandlerEntry>>,
    next_id: AtomicU64,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(Vec::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Registers a handler and spawns its dedicated worker task.
    pub fn register(&self, filter: EventFilter, handler: StoredHandler) -> HandlerId {
        let id = HandlerId(self.next_id.fetch_add(1, Ordering::Relaxed));
        let (tx, mut rx) = mpsc::channel::<ClientEvent>(HANDLER_QUEUE_CAPACITY);

        tokio::spawn(async move {
            // The handler is invoked sequentially per event: preserves ordering
            // within a handler. The loop exits when all senders drop (which
            // happens on `unregister` or when the registry itself drops).
            while let Some(event) = rx.recv().await {
                handler.call(event).await;
            }
        });

        self.handlers.write().expect("handler registry poisoned").push(HandlerEntry {
            id,
            filter,
            tx,
        });
        id
    }

    /// Unregisters a handler by id. The worker task exits once the channel
    /// drains.
    pub fn unregister(&self, id: HandlerId) -> bool {
        let mut handlers = self.handlers.write().expect("handler registry poisoned");
        let len_before = handlers.len();
        handlers.retain(|entry| entry.id != id);
        handlers.len() < len_before
    }

    /// Dispatches events to all matching handlers.
    ///
    /// Non-blocking: uses `try_send` on each handler's bounded queue. If a
    /// handler's queue is full, the event is dropped and a warning is logged —
    /// emission is never stalled by a slow handler.
    pub fn dispatch(&self, events: &[ClientEvent]) {
        if events.is_empty() {
            return;
        }
        let handlers = self.handlers.read().expect("handler registry poisoned");

        for event in events {
            for entry in handlers.iter() {
                if !entry.filter.matches(event) {
                    continue;
                }
                if let Err(mpsc::error::TrySendError::Full(_)) = entry.tx.try_send(event.clone()) {
                    // Full queue: handler is slow. Drop the event rather than
                    // stall emission. `Closed` (worker died) is silently
                    // ignored; the entry will be cleaned up on unregister.
                    warn!(
                        handler_id = ?entry.id,
                        capacity = HANDLER_QUEUE_CAPACITY,
                        "handler queue full, dropping event",
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{empty_summary, test_note_arc, test_note_id};

    #[test]
    fn filter_matches_specific_note() {
        let note_id = test_note_id();
        let note = test_note_arc();

        assert!(
            EventFilter::NoteCommitted(note_id).matches(&ClientEvent::NoteCommitted { note_id })
        );
        assert!(!EventFilter::NoteCommitted(note_id).matches(&ClientEvent::NoteReceived { note }));
        assert!(
            !EventFilter::NoteCommitted(note_id).matches(&ClientEvent::NoteConsumed { note_id })
        );
    }

    #[test]
    fn filter_matches_any_variant() {
        let note_id = test_note_id();
        let note = test_note_arc();

        assert!(EventFilter::AnyNoteReceived.matches(&ClientEvent::NoteReceived { note }));
        assert!(!EventFilter::AnyNoteReceived.matches(&ClientEvent::NoteCommitted { note_id }));
        assert!(
            EventFilter::AnySyncCompleted
                .matches(&ClientEvent::SyncCompleted { summary: empty_summary() })
        );
    }

    #[tokio::test]
    async fn register_and_unregister() {
        let registry = HandlerRegistry::new();
        let handler = StoredHandler::new(|_| async {});
        let id = registry.register(EventFilter::NoteCommitted(test_note_id()), handler);

        assert!(registry.unregister(id));
        assert!(!registry.unregister(id));
    }
}
