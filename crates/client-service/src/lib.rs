//! A service wrapper for the Miden client that provides operation coordination,
//! background synchronization, and an event system for reacting to on-chain
//! state changes.
//!
//! # Overview
//!
//! The [`ClientService`] wraps a [`Client`](miden_client::Client) to provide:
//!
//! - **Operation Coordination** - Mutual exclusion between sync and transaction operations
//! - **Background Sync** - Periodic automatic synchronization with the network
//! - **Event System** - Subscribe to state changes, register handlers, or await specific events
//! - **Async-Friendly Access** - Direct client access via [`MutexGuard`](tokio::sync::MutexGuard)
//!
//! # Example
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use std::time::Duration;
//! use miden_client_service::{ClientService, ServiceConfig, EventFilter};
//!
//! let service = Arc::new(ClientService::new(client, ServiceConfig::default()));
//! let _sync = service.start_background_sync();
//!
//! // Submit a transaction and wait for it to be committed
//! let tx_id = service.submit_transaction(account_id, tx_request).await?;
//! service.once(EventFilter::TransactionCommitted(tx_id), Some(Duration::from_secs(60))).await?;
//!
//! // Register a persistent handler (has access to the service)
//! service.on(EventFilter::AnyNoteReceived, |event, service| async move {
//!     let client = service.client().await;
//!     // query client state...
//! });
//!
//! // Or subscribe to the raw event stream
//! let mut events = service.subscribe();
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use miden_client::auth::TransactionAuthenticator;
use miden_client::sync::SyncSummary;
use miden_client::transaction::{TransactionId, TransactionRequest};
use miden_client::{Client, ClientError};
use miden_protocol::account::AccountId;
use tokio::sync::{Mutex, MutexGuard, broadcast};
use tracing::{debug, info, warn};

pub(crate) mod events;
mod handlers;

mod config;
pub use config::ServiceConfig;
pub use events::ClientEvent;
pub use handlers::{EventFilter, HandlerId};

#[cfg(test)]
pub(crate) mod test_utils;

/// A service wrapper for the Miden client that provides coordination, background
/// sync, and an event system.
///
/// The service is `Send + Sync` and designed to be used behind an `Arc`.
pub struct ClientService<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    client: Mutex<Client<AUTH>>,
    config: ServiceConfig,
    event_tx: broadcast::Sender<ClientEvent>,
    handlers: handlers::HandlerRegistry,
}

impl<AUTH> ClientService<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    /// Creates a new service wrapping the given client.
    pub fn new(client: Client<AUTH>, config: ServiceConfig) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            client: Mutex::new(client),
            config,
            event_tx,
            handlers: handlers::HandlerRegistry::new(),
        }
    }

    /// Creates a new service with default configuration.
    pub fn with_default_config(client: Client<AUTH>) -> Self {
        Self::new(client, ServiceConfig::default())
    }

    /// Returns a reference to the service configuration.
    pub fn config(&self) -> &ServiceConfig {
        &self.config
    }

    /// Returns a guard providing access to the underlying client.
    pub async fn client(&self) -> MutexGuard<'_, Client<AUTH>> {
        self.client.lock().await
    }

    /// Synchronizes the client state with the network.
    ///
    /// Acquires exclusive access, fetches the latest state, applies it to the
    /// local store, and emits [`ClientEvent`]s for each change observed.
    pub async fn sync_state(&self) -> Result<SyncSummary, ClientError> {
        debug!("Starting coordinated sync");
        let mut client = self.client.lock().await;

        let state_sync_update = client.get_sync_update().await?;
        let summary: SyncSummary = (&state_sync_update).into();
        client.apply_state_sync(state_sync_update).await?;

        info!(block_num = ?summary.block_num, "Sync completed");

        self.emit_events(events::events_from_sync(&summary));

        Ok(summary)
    }

    /// Submits a new transaction.
    pub async fn submit_transaction(
        &self,
        account_id: AccountId,
        transaction_request: TransactionRequest,
    ) -> Result<TransactionId, ClientError> {
        debug!(?account_id, "Starting coordinated transaction");
        let mut client = self.client.lock().await;
        let tx_id = client.submit_new_transaction(account_id, transaction_request).await?;
        info!(?tx_id, "Transaction submitted");
        Ok(tx_id)
    }

    /// Starts a background sync loop that periodically syncs with the network.
    ///
    /// Returns a handle that can be used to stop the background sync.
    /// If background sync is disabled in the config, returns an inactive handle.
    pub fn start_background_sync(self: &Arc<Self>) -> BackgroundSyncHandle {
        let Some(interval) = self.config.sync_interval else {
            let (tx, _) = broadcast::channel(1);
            return BackgroundSyncHandle::new(tx);
        };

        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        let service = Arc::clone(self);

        tokio::spawn(async move {
            info!(?interval, "Starting background sync");
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("Background sync shutting down");
                        break;
                    }
                    _ = tokio::time::sleep(interval) => {
                        match service.sync_state().await {
                            Ok(summary) => {
                                debug!(block_num = ?summary.block_num, "Background sync completed");
                            }
                            Err(e) => {
                                warn!(error = %e, "Background sync failed");
                            }
                        }
                    }
                }
            }
        });

        BackgroundSyncHandle::new(shutdown_tx)
    }

    // -------------------------------------------------------------------
    // Event system
    // -------------------------------------------------------------------

    /// Returns a receiver for the raw event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<ClientEvent> {
        self.event_tx.subscribe()
    }

    /// Registers a persistent event handler.
    ///
    /// The handler fires whenever an event matching `filter` is observed.
    /// It receives the [`ClientEvent`] and an `Arc<ClientService>` so it can
    /// query client state. Each invocation is spawned as a separate task.
    ///
    /// Returns a [`HandlerId`] that can be used with
    /// [`remove_handler`](Self::remove_handler).
    pub fn on<F, Fut>(self: &Arc<Self>, filter: EventFilter, handler: F) -> HandlerId
    where
        F: Fn(ClientEvent, Arc<ClientService<AUTH>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let service = Arc::clone(self);
        let stored = handlers::StoredHandler::new(move |event| {
            let service = Arc::clone(&service);
            handler(event, service)
        });
        self.handlers.register(filter, stored)
    }

    /// Removes a previously registered handler.
    pub fn remove_handler(&self, id: HandlerId) -> bool {
        self.handlers.unregister(id)
    }

    /// Waits for the first event matching `filter`.
    ///
    /// Returns the matched [`ClientEvent`]. If `timeout` is `Some`, returns
    /// an error if no matching event arrives within the duration.
    pub async fn once(
        &self,
        filter: EventFilter,
        timeout: Option<Duration>,
    ) -> Result<ClientEvent, EventTimeoutError> {
        let mut rx = self.event_tx.subscribe();

        let fut = async {
            loop {
                match rx.recv().await {
                    Ok(event) if filter.matches(&event) => return event,
                    Ok(_) => {},
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(skipped = n, "Event receiver lagged");
                    },
                    Err(broadcast::error::RecvError::Closed) => {
                        std::future::pending::<()>().await;
                        unreachable!()
                    },
                }
            }
        };

        match timeout {
            Some(duration) => {
                tokio::time::timeout(duration, fut).await.map_err(|_| EventTimeoutError)
            },
            None => Ok(fut.await),
        }
    }

    fn emit_events(&self, events: Vec<ClientEvent>) {
        self.handlers.dispatch(&events);
        for event in events {
            let _ = self.event_tx.send(event);
        }
    }
}

/// Error returned when a [`once`](ClientService::once) call times out.
#[derive(Debug, Clone)]
pub struct EventTimeoutError;

impl core::fmt::Display for EventTimeoutError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "timed out waiting for event")
    }
}

impl std::error::Error for EventTimeoutError {}

/// A handle to control background sync operations.
pub struct BackgroundSyncHandle {
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl BackgroundSyncHandle {
    fn new(shutdown_tx: broadcast::Sender<()>) -> Self {
        Self { shutdown_tx: Some(shutdown_tx) }
    }

    /// Signals the background sync to stop.
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    /// Returns true if the handle can still control the background sync.
    pub fn is_active(&self) -> bool {
        self.shutdown_tx.as_ref().is_some_and(|tx| tx.receiver_count() > 0)
    }
}

impl Drop for BackgroundSyncHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{empty_summary, test_note_id};

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn service_is_send_sync() {
        assert_send_sync::<ClientService<()>>();
    }

    #[tokio::test]
    async fn subscribe_receives_events() {
        let (event_tx, _) = broadcast::channel(256);
        let mut rx = event_tx.subscribe();

        let note_id = test_note_id();
        let _ = event_tx.send(ClientEvent::NoteCommitted { note_id });

        let event = rx.recv().await.unwrap();
        assert!(matches!(event, ClientEvent::NoteCommitted { note_id: id } if id == note_id));
    }

    #[tokio::test]
    async fn once_resolves_on_match() {
        let (event_tx, _) = broadcast::channel(256);
        let mut rx = event_tx.subscribe();
        let note_id = test_note_id();

        let tx = event_tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _ = tx.send(ClientEvent::NoteCommitted { note_id });
        });

        let filter = EventFilter::NoteCommitted(note_id);
        let event = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                match rx.recv().await {
                    Ok(event) if filter.matches(&event) => return event,
                    Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {},
                    Err(broadcast::error::RecvError::Closed) => unreachable!(),
                }
            }
        })
        .await
        .unwrap();

        assert_eq!(event.note_id().unwrap(), note_id);
    }

    #[tokio::test]
    async fn once_times_out() {
        let (_event_tx, mut rx) = broadcast::channel::<ClientEvent>(256);

        let result = tokio::time::timeout(Duration::from_millis(50), async {
            loop {
                match rx.recv().await {
                    Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {},
                    Err(broadcast::error::RecvError::Closed) => {
                        std::future::pending::<()>().await;
                        unreachable!()
                    },
                }
            }
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn handler_fires_for_matching_event() {
        let registry = handlers::HandlerRegistry::new();
        let note_id = test_note_id();

        let (notify_tx, notify_rx) = tokio::sync::oneshot::channel();
        let notify_tx = Arc::new(std::sync::Mutex::new(Some(notify_tx)));

        let handler = handlers::StoredHandler::new(move |_event| {
            let tx = notify_tx.clone();
            async move {
                if let Some(tx) = tx.lock().unwrap().take() {
                    let _ = tx.send(());
                }
            }
        });

        registry.register(EventFilter::NoteCommitted(note_id), handler);
        registry.dispatch(&[ClientEvent::NoteCommitted { note_id }]);

        tokio::time::timeout(Duration::from_secs(1), notify_rx)
            .await
            .expect("timed out")
            .expect("channel dropped");
    }

    #[tokio::test]
    async fn handler_skips_non_matching_event() {
        let registry = handlers::HandlerRegistry::new();
        let note_id = test_note_id();

        let (notify_tx, notify_rx) = tokio::sync::oneshot::channel();
        let notify_tx = Arc::new(std::sync::Mutex::new(Some(notify_tx)));

        let handler = handlers::StoredHandler::new(move |_event| {
            let tx = notify_tx.clone();
            async move {
                if let Some(tx) = tx.lock().unwrap().take() {
                    let _ = tx.send(());
                }
            }
        });

        registry.register(EventFilter::NoteCommitted(note_id), handler);
        registry.dispatch(&[ClientEvent::NoteReceived { note_id }]);

        let result = tokio::time::timeout(Duration::from_millis(50), notify_rx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn events_from_sync_integration() {
        let note_id = test_note_id();
        let mut summary = empty_summary();
        summary.new_public_notes = vec![note_id];

        let events = events::events_from_sync(&summary);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].note_id().unwrap(), note_id);
        assert!(events[1].summary().is_some());
    }

    #[test]
    fn client_event_accessors() {
        let note_id = test_note_id();
        let event = ClientEvent::NoteReceived { note_id };

        assert_eq!(event.note_id(), Some(note_id));
        assert_eq!(event.transaction_id(), None);
        assert_eq!(event.account_id(), None);
        assert!(event.summary().is_none());
    }
}
