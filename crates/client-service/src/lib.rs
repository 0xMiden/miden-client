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

use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use miden_client::auth::TransactionAuthenticator;
use miden_client::store::{InputNoteRecord, NoteFilter};
use miden_client::sync::{SyncSummary, SyncTarget};
use miden_client::transaction::{TransactionId, TransactionRequest};
use miden_client::{Client, ClientError};
use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::NoteId;
use tokio::sync::{Mutex, MutexGuard, broadcast};
use tracing::{debug, info, warn};

mod awaiters;
pub(crate) mod events;
mod handlers;
mod tx_queue;

mod config;
pub use awaiters::{AwaiterCancelled, AwaiterFuture};
pub use config::ServiceConfig;
pub use events::ClientEvent;
pub use handlers::{EventFilter, HandlerId};
/// Re-exported so consumers implementing a custom note filter for
/// [`ClientService::with_note_screener`] don't have to reach into `miden_client`.
///
/// **This is a filter, not a listener.** Implementing [`OnNoteReceived`] changes what the
/// client persists during sync. To observe notes without changing persistence, use
/// [`ClientService::subscribe`] or [`ClientService::on`].
pub use miden_client::sync::{NoteUpdateAction, OnNoteReceived};
pub use tx_queue::{EnqueueError, EnqueuedTx, TransactionQueueHandle};

/// Capacity of the broadcast channel backing [`ClientService::subscribe`].
///
/// Subscribers that fall more than this many events behind will receive
/// `RecvError::Lagged` and permanently drop those events. A catch-up sync can
/// emit hundreds or thousands of events in a single burst, so this is sized
/// generously by default.
const EVENT_BROADCAST_CAPACITY: usize = 4096;

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
    awaiters: awaiters::AwaiterRegistry,
    /// Optional custom note filter. When `None`, sync uses the default
    /// [`NoteScreener`](miden_client::note::NoteScreener) constructed from the client's store and
    /// RPC API.
    note_screener: Option<Arc<dyn OnNoteReceived>>,
}

impl<AUTH> ClientService<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    /// Creates a new service wrapping the given client.
    pub fn new(client: Client<AUTH>, config: ServiceConfig) -> Self {
        Self::build(client, config, None)
    }

    /// Creates a new service with a caller-supplied note filter.
    ///
    /// The screener controls **what gets persisted to the store** during sync — Commit, Insert,
    /// or Discard per incoming note. Events emitted by the service mirror persistence: a more
    /// aggressive `Discard` policy produces fewer events, a more aggressive `Insert` policy
    /// produces more.
    ///
    /// `OnNoteReceived` is a **filter**, not a listener. It is invoked *during* sync to decide
    /// what the client keeps. If all you need is to observe notes (for analytics, UI, etc.),
    /// use [`subscribe`](Self::subscribe) or [`on`](Self::on) instead — observers run after
    /// persistence and do not affect store state.
    ///
    /// When `None` is passed (or [`new`](Self::new) is used), the default
    /// [`NoteScreener`](miden_client::note::NoteScreener) is used: keep tracked notes, keep
    /// public notes whose tag is tracked, keep public notes consumable by a tracked account,
    /// discard everything else.
    pub fn with_note_screener(
        client: Client<AUTH>,
        config: ServiceConfig,
        screener: Arc<dyn OnNoteReceived>,
    ) -> Self {
        Self::build(client, config, Some(screener))
    }

    fn build(
        client: Client<AUTH>,
        config: ServiceConfig,
        note_screener: Option<Arc<dyn OnNoteReceived>>,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(EVENT_BROADCAST_CAPACITY);
        Self {
            client: Mutex::new(client),
            config,
            event_tx,
            handlers: handlers::HandlerRegistry::new(),
            awaiters: awaiters::AwaiterRegistry::new(),
            note_screener,
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
    ///
    /// When [`ServiceConfig::sync_chunk_blocks`] is set, advances at most that
    /// many blocks per call; otherwise syncs to the current chain tip. When a
    /// custom note filter was supplied via [`with_note_screener`](Self::with_note_screener),
    /// it is used instead of the default screener.
    pub async fn sync_state(&self) -> Result<SyncSummary, ClientError> {
        debug!("Starting coordinated sync");
        let mut client = self.client.lock().await;

        let screener: Arc<dyn OnNoteReceived> = match &self.note_screener {
            Some(s) => Arc::clone(s),
            None => Arc::new(client.note_screener()),
        };
        let upper_bound = match self.config.sync_chunk_blocks {
            Some(chunk) => {
                let current = client.get_sync_height().await?;
                SyncTarget::BlockNumber(BlockNumber::from(
                    current.as_u32().saturating_add(chunk.get()),
                ))
            },
            None => SyncTarget::CommittedChainTip,
        };

        let state_sync_update = client.get_sync_update_with_screener(screener, upper_bound).await?;
        let summary: SyncSummary = (&state_sync_update).into();
        client.apply_state_sync(state_sync_update).await?;

        info!(block_num = ?summary.block_num, "Sync completed");

        // Enrich the event stream: fetch full records for notes we just inserted so
        // subscribers get the body inline without doing their own store roundtrip.
        // One batched query per sync, skipped entirely if no new public notes arrived.
        let new_note_records = fetch_new_note_records(&client, &summary.new_public_notes).await?;
        drop(client);

        self.emit_events(events::events_from_sync(&summary, &new_note_records));

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
    /// If [`ServiceConfig::sync_on_start`] is `true` (the default), an
    /// immediate sync is run before the first interval tick.
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
        let sync_on_start = self.config.sync_on_start;

        tokio::spawn(async move {
            info!(?interval, sync_on_start, "Starting background sync");

            if sync_on_start {
                match service.sync_state().await {
                    Ok(summary) => {
                        debug!(block_num = ?summary.block_num, "Initial sync completed");
                    },
                    Err(e) => {
                        warn!(error = %e, "Initial sync failed");
                    },
                }
            }

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

    /// Pre-registers an awaiter for the next event matching `filter`.
    ///
    /// Returns a future that resolves with the first matching [`ClientEvent`].
    /// The awaiter is registered synchronously before this method returns, so
    /// events emitted after this call cannot be missed — including events
    /// triggered by actions taken while the future is being awaited.
    ///
    /// Prefer this over [`once`](Self::once) when awaiting an event caused by
    /// an action you're about to perform:
    ///
    /// ```rust,ignore
    /// let awaiter = service.expect(EventFilter::TransactionCommitted(tx_id));
    /// service.submit_transaction(account_id, request).await?;
    /// let event = awaiter.await?;  // cannot miss, even if sync bursts events
    /// ```
    ///
    /// Unlike [`subscribe`](Self::subscribe)-based awaiting, matching is
    /// guaranteed: awaiters are fulfilled inline during event emission and are
    /// not subject to broadcast channel lag.
    pub fn expect(&self, filter: EventFilter) -> AwaiterFuture {
        self.awaiters.register(filter)
    }

    /// Waits for the first event matching `filter`, with an optional timeout.
    ///
    /// Convenience wrapper over [`expect`](Self::expect) plus
    /// [`tokio::time::timeout`]. Returns [`EventTimeoutError`] if the timeout
    /// fires before a matching event arrives, or if the service is dropped.
    pub async fn once(
        &self,
        filter: EventFilter,
        timeout: Option<Duration>,
    ) -> Result<ClientEvent, EventTimeoutError> {
        let awaiter = self.expect(filter);
        match timeout {
            Some(duration) => match tokio::time::timeout(duration, awaiter).await {
                Ok(Ok(event)) => Ok(event),
                Ok(Err(_)) | Err(_) => Err(EventTimeoutError),
            },
            None => awaiter.await.map_err(|_| EventTimeoutError),
        }
    }

    fn emit_events(&self, events: Vec<ClientEvent>) {
        // Fulfill awaiters first: they are lossless and callers depend on them.
        self.awaiters.fulfill(&events);
        // Hand off to per-handler workers (bounded queues, backpressure on the
        // handler side — never blocks emission).
        self.handlers.dispatch(&events);
        // Best-effort broadcast for raw subscribers.
        for event in events {
            let _ = self.event_tx.send(event);
        }
    }
}

/// Bulk-fetches input-note records for the given IDs and wraps each in an `Arc` for cheap
/// broadcasting. A single store query; returns an empty map if `note_ids` is empty.
async fn fetch_new_note_records<AUTH>(
    client: &Client<AUTH>,
    note_ids: &[NoteId],
) -> Result<BTreeMap<NoteId, Arc<InputNoteRecord>>, ClientError>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    if note_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    Ok(client
        .get_input_notes(NoteFilter::List(note_ids.to_vec()))
        .await?
        .into_iter()
        .map(|record| (record.id(), Arc::new(record)))
        .collect())
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
    use crate::test_utils::{empty_summary, test_note_arc, test_note_id};

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
        registry.dispatch(&[ClientEvent::NoteReceived { note: test_note_arc() }]);

        let result = tokio::time::timeout(Duration::from_millis(50), notify_rx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn events_from_sync_integration() {
        let note_id = test_note_id();
        let note = test_note_arc();
        let mut summary = empty_summary();
        summary.new_public_notes = vec![note_id];

        let mut records = BTreeMap::new();
        records.insert(note_id, Arc::clone(&note));

        let events = events::events_from_sync(&summary, &records);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].note_id().unwrap(), note_id);
        assert!(events[1].summary().is_some());
    }

    #[test]
    fn client_event_accessors() {
        let note_id = test_note_id();
        let event = ClientEvent::NoteReceived { note: test_note_arc() };

        assert_eq!(event.note_id(), Some(note_id));
        assert_eq!(event.transaction_id(), None);
        assert_eq!(event.account_id(), None);
        assert!(event.summary().is_none());
    }
}
