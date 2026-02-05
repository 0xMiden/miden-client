//! A service wrapper for the Miden client that provides operation coordination,
//! event emission, and background synchronization.
//!
//! # Overview
//!
//! The [`ClientService`] wraps a [`Client`](miden_client::Client) to provide:
//!
//! - **Operation Coordination** - Mutual exclusion between sync and transaction operations
//! - **Event System** - Ability to react to state changes (notes received, transactions committed,
//!   etc.)
//! - **Background Sync** - Periodic automatic synchronization with the network
//! - **Sync Serialization** - Sync requests are serialized and never overlap with transactions
//! - **Custom Sync Behavior** - Pluggable [`Syncer`] trait for customizing what data is synced
//!
//! # Example
//!
//! ```rust,ignore
//! use miden_client_service::{ClientService, ServiceConfig};
//! use miden_client::Client;
//!
//! // Create the underlying client
//! let client = Client::builder()
//!     .rpc(rpc_client)
//!     .store(store)
//!     .authenticator(keystore)
//!     .build()
//!     .await?;
//!
//! // Wrap it in a service
//! let service = ClientService::new(client, ServiceConfig::default());
//!
//! // Start background sync
//! let sync_handle = service.start_background_sync();
//!
//! // Operations are now coordinated
//! let summary = service.sync_state().await?;
//! let tx_id = service.submit_transaction(account_id, tx_request).await?;
//! ```
//!
//! # Custom Sync Behavior
//!
//! For advanced use cases, you can implement the [`Syncer`] trait to control exactly what
//! data is synchronized:
//!
//! ```rust,ignore
//! use miden_client_service::{ClientService, ServiceConfig, Syncer};
//! use miden_client::{Client, ClientError};
//! use miden_client::sync::StateSyncUpdate;
//! use async_trait::async_trait;
//!
//! struct CustomSyncer {
//!     // Custom sync parameters
//! }
//!
//! #[async_trait]
//! impl<AUTH> Syncer<AUTH> for CustomSyncer
//! where
//!     AUTH: miden_client::auth::TransactionAuthenticator + Send + Sync + 'static,
//! {
//!     async fn sync(&self, client: &Client<AUTH>) -> Result<StateSyncUpdate, ClientError> {
//!         // Custom sync logic - control which accounts, note tags, etc. to sync
//!         client.get_sync_update().await
//!     }
//! }
//!
//! // Use the custom syncer
//! let service = ClientService::with_syncer(client, config, CustomSyncer { /* ... */ });
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use std::collections::BTreeMap;
use std::sync::Arc;

use miden_client::Client;
use miden_client::auth::TransactionAuthenticator;
use miden_client::store::{InputNoteRecord, InputNoteState, NoteFilter, TransactionFilter};
use miden_client::sync::{StateSyncUpdate, SyncSummary};
use miden_client::transaction::{TransactionId, TransactionRequest, TransactionStatus};
use miden_protocol::account::AccountId;
use miden_protocol::note::NoteId;
use tokio::sync::{Mutex, RwLock, broadcast};
use tracing::{debug, info, warn};

mod config;
mod coordinator;
mod errors;
mod events;
mod handlers;
mod syncer;

pub use config::ServiceConfig;
pub use coordinator::{BackgroundSyncHandle, OperationCoordinator};
pub use errors::ServiceError;
pub use events::ServiceEvent;
pub use handlers::{AsyncEventHandler, EventBus, EventHandler, LogLevel, LoggingHandler};
pub use syncer::{DefaultSyncer, Syncer};

/// A service wrapper for the Miden client that provides coordination, events, and background sync.
///
/// `ClientService` adds the following features on top of the base `Client`:
///
/// - **Sync serialization**: Sync requests are serialized and never overlap with transactions,
///   ensuring consistency.
///
/// - **Transaction coordination**: Transactions are serialized and wait for any ongoing sync to
///   complete before executing.
///
/// - **Event emission**: State changes (notes received, transactions committed, etc.) are broadcast
///   to registered event handlers.
///
/// - **Background sync**: Optional periodic synchronization that runs in the background.
///
/// - **Custom sync behavior**: Pluggable [`Syncer`] trait allows customizing what data is synced.
///
/// The service is `Send + Sync` and can be safely shared across tasks.
pub struct ClientService<AUTH, S = DefaultSyncer>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
    S: Syncer<AUTH> + 'static,
{
    /// The underlying client, wrapped in a mutex for interior mutability.
    client: Mutex<Client<AUTH>>,
    /// Coordinates sync and transaction operations.
    coordinator: OperationCoordinator,
    /// Event bus for dispatching events to handlers.
    event_bus: RwLock<EventBus>,
    /// Service configuration.
    config: ServiceConfig,
    /// The syncer used for sync operations.
    syncer: S,
}

impl<AUTH> ClientService<AUTH, DefaultSyncer>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    /// Creates a new service wrapping the given client with the default syncer.
    pub fn new(client: Client<AUTH>, config: ServiceConfig) -> Self {
        Self::with_syncer(client, config, DefaultSyncer)
    }

    /// Creates a new service with default configuration and default syncer.
    pub fn with_default_config(client: Client<AUTH>) -> Self {
        Self::new(client, ServiceConfig::default())
    }
}

impl<AUTH, S> ClientService<AUTH, S>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
    S: Syncer<AUTH> + 'static,
{
    /// Creates a new service with a custom syncer.
    ///
    /// The syncer controls what data is synchronized with the network.
    /// Use [`DefaultSyncer`] for standard behavior, or implement [`Syncer`]
    /// for custom sync logic.
    pub fn with_syncer(client: Client<AUTH>, config: ServiceConfig, syncer: S) -> Self {
        Self {
            client: Mutex::new(client),
            coordinator: OperationCoordinator::new(),
            event_bus: RwLock::new(EventBus::new()),
            config,
            syncer,
        }
    }

    /// Returns a reference to the service configuration.
    pub fn config(&self) -> &ServiceConfig {
        &self.config
    }

    /// Registers a synchronous event handler.
    ///
    /// Handlers are called in order when events are emitted.
    pub async fn register_handler(&self, handler: Arc<dyn EventHandler>) {
        self.event_bus.write().await.register_handler(handler);
    }

    /// Registers an asynchronous event handler.
    ///
    /// These handlers are notified concurrently when events are emitted.
    pub async fn register_async_handler(&self, handler: Arc<dyn AsyncEventHandler>) {
        self.event_bus.write().await.register_async_handler(handler);
    }

    /// Synchronizes the client state with the network.
    ///
    /// This operation is coordinated:
    /// - Syncs are serialized and never overlap with transactions
    ///
    /// The sync behavior is determined by the configured [`Syncer`].
    /// Events are emitted for state changes discovered during sync.
    pub async fn sync_state(&self) -> Result<SyncSummary, ServiceError> {
        self.coordinator
            .with_sync(|| async {
                debug!("Starting coordinated sync");
                let emit_tx_events = self.config.emit_transaction_events;

                let (summary, transaction_events) = {
                    let client = self.client.lock().await;

                    // Get pending transactions before sync if we need to emit events
                    let pending_transactions = if emit_tx_events {
                        client.get_transactions(TransactionFilter::Uncommitted).await?
                    } else {
                        Vec::new()
                    };

                    // Use the syncer to get the sync update
                    let state_sync_update: StateSyncUpdate = self.syncer.sync(&*client).await?;
                    let summary: SyncSummary = (&state_sync_update).into();

                    // Release the immutable borrow and get a mutable one to apply
                    drop(client);
                    let mut client = self.client.lock().await;
                    client.apply_state_sync(state_sync_update).await?;

                    let transaction_events = if emit_tx_events && !pending_transactions.is_empty() {
                        let ids = pending_transactions.iter().map(|tx| tx.id).collect();
                        let updated_transactions =
                            client.get_transactions(TransactionFilter::Ids(ids)).await?;
                        Self::build_transaction_events(updated_transactions)
                    } else {
                        Vec::new()
                    };

                    (summary, transaction_events)
                };

                if self.config.emit_sync_events || self.config.emit_transaction_events {
                    self.emit_sync_events(&summary, transaction_events).await?;
                }

                info!(block_num = ?summary.block_num, "Sync completed");
                Ok(summary)
            })
            .await
    }

    /// Submits a new transaction.
    ///
    /// This operation is coordinated:
    /// - Waits for any ongoing sync to complete
    /// - Only one transaction runs at a time
    /// - No sync can start while a transaction is running
    ///
    /// Events are emitted when the transaction is committed or discarded.
    pub async fn submit_transaction(
        &self,
        account_id: AccountId,
        transaction_request: TransactionRequest,
    ) -> Result<TransactionId, ServiceError> {
        self.coordinator
            .with_transaction(|| async {
                debug!(?account_id, "Starting coordinated transaction");
                let tx_id = {
                    let mut client = self.client.lock().await;
                    client.submit_new_transaction(account_id, transaction_request).await?
                };

                info!(?tx_id, "Transaction submitted");
                Ok(tx_id)
            })
            .await
    }

    /// Starts a background sync loop that periodically syncs with the network.
    ///
    /// Returns a handle that can be used to stop the background sync.
    /// The sync will use the interval configured in [`ServiceConfig::sync_interval`].
    ///
    /// If background sync is disabled in the config, this returns a handle that does nothing.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let service = Arc::new(ClientService::new(client, config));
    /// let handle = service.start_background_sync();
    /// // Later: handle.stop();
    /// ```
    pub fn start_background_sync(self: &Arc<Self>) -> BackgroundSyncHandle {
        let Some(interval) = self.config.sync_interval else {
            // Return a dummy handle that does nothing
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

    /// Provides direct access to the underlying client for operations that don't need coordination.
    ///
    /// Use this for read-only operations like getting account info or listing notes.
    /// For sync and transaction operations, use the coordinated methods instead.
    pub async fn with_client<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Client<AUTH>) -> R,
    {
        let client = self.client.lock().await;
        f(&client)
    }

    /// Provides mutable access to the underlying client.
    ///
    /// **Warning**: This bypasses coordination. Only use for operations that don't
    /// conflict with sync or transactions.
    pub async fn with_client_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Client<AUTH>) -> R,
    {
        let mut client = self.client.lock().await;
        f(&mut client)
    }

    fn build_transaction_events(
        transactions: Vec<miden_client::transaction::TransactionRecord>,
    ) -> Vec<ServiceEvent> {
        let mut events = Vec::new();

        for transaction in transactions {
            match transaction.status {
                TransactionStatus::Committed { block_number, .. } => {
                    events.push(ServiceEvent::TransactionCommitted {
                        transaction_id: transaction.id,
                        block_num: block_number,
                    });
                },
                TransactionStatus::Discarded(cause) => {
                    events.push(ServiceEvent::TransactionDiscarded {
                        transaction_id: transaction.id,
                        cause,
                    });
                },
                TransactionStatus::Pending => {},
            }
        }

        events
    }

    /// Emits events based on the sync summary.
    async fn emit_sync_events(
        &self,
        summary: &SyncSummary,
        mut transaction_events: Vec<ServiceEvent>,
    ) -> Result<(), ServiceError> {
        let mut events = Vec::new();

        if self.config.emit_sync_events {
            let (note_records, account_nonces) = {
                let client = self.client.lock().await;

                let note_ids = Self::collect_note_ids(summary);
                let note_records = if note_ids.is_empty() {
                    Vec::new()
                } else {
                    client.get_input_notes(NoteFilter::List(note_ids)).await?
                };

                let mut account_nonces = BTreeMap::new();
                for account_id in &summary.updated_accounts {
                    let nonce = client.account_reader(*account_id).nonce().await?;
                    account_nonces.insert(*account_id, nonce.as_int());
                }

                (note_records, account_nonces)
            };

            let note_records = Self::index_notes(note_records);

            // Emit NoteReceived events for new public notes
            for note_id in &summary.new_public_notes {
                match note_records.get(note_id).and_then(|record| record.metadata().cloned()) {
                    Some(metadata) => {
                        let event = ServiceEvent::NoteReceived {
                            note_id: *note_id,
                            tag: metadata.tag(),
                            metadata,
                        };
                        events.push(event);
                    },
                    None => {
                        warn!(?note_id, "Skipping NoteReceived event; metadata not available");
                    },
                }
            }

            // Emit NoteCommitted events
            for note_id in &summary.committed_notes {
                let metadata =
                    note_records.get(note_id).and_then(|record| record.metadata()).cloned();
                let event = ServiceEvent::NoteCommitted {
                    note_id: *note_id,
                    block_num: summary.block_num,
                    metadata,
                };
                events.push(event);
            }

            // Emit NoteConsumed events
            for note_id in &summary.consumed_notes {
                let Some(record) = note_records.get(note_id) else {
                    warn!(?note_id, "Skipping NoteConsumed event; note not found");
                    continue;
                };

                let block_num = Self::consumed_block_num(record, summary.block_num);
                let event = ServiceEvent::NoteConsumed {
                    note_id: *note_id,
                    nullifier: record.nullifier(),
                    block_num,
                    metadata: record.metadata().cloned(),
                };
                events.push(event);
            }

            // Emit AccountUpdated events
            for account_id in &summary.updated_accounts {
                if let Some(new_nonce) = account_nonces.get(account_id) {
                    events.push(ServiceEvent::AccountUpdated {
                        account_id: *account_id,
                        new_nonce: *new_nonce,
                    });
                } else {
                    warn!(?account_id, "Skipping AccountUpdated event; nonce not available");
                }
            }

            // Emit AccountLocked events
            for account_id in &summary.locked_accounts {
                events.push(ServiceEvent::AccountLocked { account_id: *account_id });
            }
        }

        if self.config.emit_transaction_events {
            events.append(&mut transaction_events);
        }

        if self.config.emit_sync_events {
            events.push(ServiceEvent::SyncCompleted { summary: summary.clone() });
        }

        let event_bus = self.event_bus.read().await;
        for event in events {
            event_bus.emit(event).await.map_err(ServiceError::HandlerError)?;
        }

        Ok(())
    }

    fn collect_note_ids(summary: &SyncSummary) -> Vec<NoteId> {
        let mut note_ids = Vec::new();
        note_ids.extend_from_slice(&summary.new_public_notes);
        note_ids.extend_from_slice(&summary.committed_notes);
        note_ids.extend_from_slice(&summary.consumed_notes);
        note_ids
    }

    fn index_notes(note_records: Vec<InputNoteRecord>) -> BTreeMap<NoteId, InputNoteRecord> {
        let mut indexed = BTreeMap::new();
        for record in note_records {
            indexed.insert(record.id(), record);
        }
        indexed
    }

    fn consumed_block_num(
        record: &InputNoteRecord,
        fallback: miden_protocol::block::BlockNumber,
    ) -> miden_protocol::block::BlockNumber {
        match record.state() {
            InputNoteState::ConsumedExternal(state) => state.nullifier_block_height,
            InputNoteState::ConsumedAuthenticatedLocal(state) => state.nullifier_block_height,
            InputNoteState::ConsumedUnauthenticatedLocal(state) => state.nullifier_block_height,
            _ => fallback,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn service_is_send_sync() {
        assert_send_sync::<ClientService<()>>();
    }
}
