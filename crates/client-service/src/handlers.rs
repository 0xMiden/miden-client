//! Event handler registration and dispatch.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use miden_client::transaction::TransactionId;
use miden_protocol::account::AccountId;
use miden_protocol::note::NoteId;

use crate::events::ClientEvent;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

#[derive(Clone)]
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
/// Used with [`ClientService::on`](crate::ClientService::on) and
/// [`ClientService::once`](crate::ClientService::once).
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
            (Self::NoteReceived(id), ClientEvent::NoteReceived { note_id })
            | (Self::NoteCommitted(id), ClientEvent::NoteCommitted { note_id }) => id == note_id,
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
    handler: StoredHandler,
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

    pub fn register(&self, filter: EventFilter, handler: StoredHandler) -> HandlerId {
        let id = HandlerId(self.next_id.fetch_add(1, Ordering::Relaxed));
        self.handlers.write().expect("handler registry poisoned").push(HandlerEntry {
            id,
            filter,
            handler,
        });
        id
    }

    pub fn unregister(&self, id: HandlerId) -> bool {
        let mut handlers = self.handlers.write().expect("handler registry poisoned");
        let len_before = handlers.len();
        handlers.retain(|entry| entry.id != id);
        handlers.len() < len_before
    }

    /// Dispatches events to all matching handlers. Each invocation is spawned
    /// as a separate tokio task with no ordering guarantees.
    pub fn dispatch(&self, events: &[ClientEvent]) {
        let handlers = self.handlers.read().expect("handler registry poisoned");

        for event in events {
            for entry in handlers.iter() {
                if entry.filter.matches(event) {
                    let handler = entry.handler.clone();
                    let event = event.clone();
                    tokio::spawn(async move { handler.call(event).await });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{empty_summary, test_note_id};

    #[test]
    fn filter_matches_specific_note() {
        let note_id = test_note_id();

        assert!(
            EventFilter::NoteCommitted(note_id).matches(&ClientEvent::NoteCommitted { note_id })
        );
        assert!(
            !EventFilter::NoteCommitted(note_id).matches(&ClientEvent::NoteReceived { note_id })
        );
        assert!(
            !EventFilter::NoteCommitted(note_id).matches(&ClientEvent::NoteConsumed { note_id })
        );
    }

    #[test]
    fn filter_matches_any_variant() {
        let note_id = test_note_id();

        assert!(EventFilter::AnyNoteReceived.matches(&ClientEvent::NoteReceived { note_id }));
        assert!(!EventFilter::AnyNoteReceived.matches(&ClientEvent::NoteCommitted { note_id }));
        assert!(
            EventFilter::AnySyncCompleted
                .matches(&ClientEvent::SyncCompleted { summary: empty_summary() })
        );
    }

    #[test]
    fn register_and_unregister() {
        let registry = HandlerRegistry::new();
        let handler = StoredHandler::new(|_| async {});
        let id = registry.register(EventFilter::NoteCommitted(test_note_id()), handler);

        assert!(registry.unregister(id));
        assert!(!registry.unregister(id));
    }
}
