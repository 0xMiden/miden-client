//! Event handler traits and the `EventBus` for dispatching events.

use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, error};

use crate::events::ServiceEvent;

/// A synchronous event handler that can filter or validate events.
///
/// Implementations should be lightweight and fast since they block
/// event processing. Use [`AsyncEventHandler`] for expensive operations
/// like I/O or network calls.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handles an event synchronously.
    ///
    /// Return `Ok(())` to continue processing, or `Err` to abort
    /// event processing. When a handler returns an error, remaining
    /// handlers are not called and the operation is aborted.
    async fn handle(&self, event: &ServiceEvent) -> Result<(), String>;
}

/// An asynchronous event handler for non-blocking operations.
///
/// Use this for logging, notifications, or other operations that
/// should not block the main event processing flow.
#[async_trait]
pub trait AsyncEventHandler: Send + Sync {
    /// Handles an event asynchronously.
    ///
    /// This method should not block. It is executed in a detached
    /// task and does not affect other handlers.
    async fn handle(&self, event: ServiceEvent);
}

/// Manages event handlers and dispatches events to them.
pub struct EventBus {
    /// Synchronous handlers that are called in order.
    sync_handlers: Vec<Arc<dyn EventHandler>>,
    /// Asynchronous handlers that are spawned concurrently.
    async_handlers: Vec<Arc<dyn AsyncEventHandler>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Creates a new empty event bus.
    pub fn new() -> Self {
        Self {
            sync_handlers: Vec::new(),
            async_handlers: Vec::new(),
        }
    }

    /// Registers a synchronous event handler.
    ///
    /// Handlers are called in the order they are registered.
    pub fn register_handler(&mut self, handler: Arc<dyn EventHandler>) {
        self.sync_handlers.push(handler);
    }

    /// Registers an asynchronous event handler.
    ///
    /// These handlers are spawned concurrently when an event is emitted.
    pub fn register_async_handler(&mut self, handler: Arc<dyn AsyncEventHandler>) {
        self.async_handlers.push(handler);
    }

    /// Emits an event to all registered handlers.
    ///
    /// Synchronous handlers are called first in order. If any handler
    /// returns an error, processing stops and the error is returned.
    ///
    /// Asynchronous handlers are notified in detached tasks and do not
    /// block event emission.
    pub async fn emit(&self, event: ServiceEvent) -> Result<(), String> {
        let event_type = event.event_type();
        debug!(event_type, "Emitting service event");

        // Call synchronous handlers in order
        for handler in &self.sync_handlers {
            if let Err(e) = handler.handle(&event).await {
                error!(event_type, error = %e, "Sync event handler failed");
                return Err(e);
            }
        }

        // Notify async handlers concurrently
        for handler in &self.async_handlers {
            let handler = handler.clone();
            let event = event.clone();
            tokio::spawn(async move {
                handler.handle(event).await;
            });
        }
        Ok(())
    }

    /// Returns the number of registered synchronous handlers.
    pub fn sync_handler_count(&self) -> usize {
        self.sync_handlers.len()
    }

    /// Returns the number of registered asynchronous handlers.
    pub fn async_handler_count(&self) -> usize {
        self.async_handlers.len()
    }
}

/// A simple logging handler that logs all events.
pub struct LoggingHandler {
    /// The log level to use (for filtering purposes).
    pub min_level: LogLevel,
}

/// Log level for the logging handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Log all events.
    Debug,
    /// Log info and above.
    Info,
    /// Log warnings and above.
    Warn,
}

impl Default for LoggingHandler {
    fn default() -> Self {
        Self { min_level: LogLevel::Info }
    }
}

#[async_trait]
impl AsyncEventHandler for LoggingHandler {
    async fn handle(&self, event: ServiceEvent) {
        match &event {
            ServiceEvent::NoteReceived { note_id, tag, metadata } => {
                tracing::info!(?note_id, ?tag, ?metadata, "Note received");
            },
            ServiceEvent::NoteCommitted { note_id, block_num, .. } => {
                tracing::info!(?note_id, ?block_num, "Note committed");
            },
            ServiceEvent::NoteConsumed { note_id, block_num, nullifier, .. } => {
                tracing::info!(?note_id, ?block_num, ?nullifier, "Note consumed");
            },
            ServiceEvent::TransactionCommitted { transaction_id, block_num } => {
                tracing::info!(?transaction_id, ?block_num, "Transaction committed");
            },
            ServiceEvent::TransactionDiscarded { transaction_id, cause } => {
                tracing::warn!(?transaction_id, ?cause, "Transaction discarded");
            },
            ServiceEvent::AccountUpdated { account_id, new_nonce } => {
                tracing::info!(?account_id, new_nonce, "Account updated");
            },
            ServiceEvent::AccountLocked { account_id } => {
                tracing::warn!(?account_id, "Account locked");
            },
            ServiceEvent::SyncCompleted { summary } => {
                if self.min_level <= LogLevel::Debug {
                    tracing::debug!(block_num = ?summary.block_num, "Sync completed");
                } else {
                    tracing::info!(block_num = ?summary.block_num, "Sync completed");
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use miden_protocol::block::BlockNumber;

    use super::*;

    struct TestHandler {
        should_fail: bool,
    }

    #[async_trait]
    impl EventHandler for TestHandler {
        async fn handle(&self, _event: &ServiceEvent) -> Result<(), String> {
            if self.should_fail {
                Err("test error".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_event_bus_basic() {
        let mut bus = EventBus::new();

        let handler = Arc::new(TestHandler { should_fail: false });
        bus.register_handler(handler);

        assert_eq!(bus.sync_handler_count(), 1);

        // Create a simple event
        let event = ServiceEvent::SyncCompleted {
            summary: miden_client::sync::SyncSummary::new_empty(BlockNumber::from(1u32)),
        };

        let result = bus.emit(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_bus_handler_error() {
        let mut bus = EventBus::new();

        let handler = Arc::new(TestHandler { should_fail: true });
        bus.register_handler(handler);

        let event = ServiceEvent::SyncCompleted {
            summary: miden_client::sync::SyncSummary::new_empty(BlockNumber::from(1u32)),
        };

        let result = bus.emit(event).await;
        assert!(result.is_err());
    }
}
