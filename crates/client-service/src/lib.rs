//! A service wrapper for the Miden client that provides operation coordination
//! and background synchronization.
//!
//! # Overview
//!
//! The [`ClientService`] wraps a [`Client`](miden_client::Client) to provide:
//!
//! - **Operation Coordination** - Mutual exclusion between sync and transaction operations
//! - **Background Sync** - Periodic automatic synchronization with the network
//! - **Async-Friendly Access** - Direct client access via [`MutexGuard`](tokio::sync::MutexGuard)
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
//! let service = Arc::new(ClientService::new(client, ServiceConfig::default()));
//!
//! // Start background sync
//! let sync_handle = service.start_background_sync();
//!
//! // Use convenience methods
//! let summary = service.sync_state().await?;
//! let tx_id = service.submit_transaction(account_id, tx_request).await?;
//!
//! // Or access the client directly for any operation
//! let mut client = service.client().await;
//! let accounts = client.get_account_headers().await?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use std::sync::Arc;

use miden_client::auth::TransactionAuthenticator;
use miden_client::sync::SyncSummary;
use miden_client::transaction::{TransactionId, TransactionRequest};
use miden_client::{Client, ClientError};
use miden_protocol::account::AccountId;
use tokio::sync::{Mutex, MutexGuard, broadcast};
use tracing::{debug, info, warn};

mod config;

pub use config::ServiceConfig;

/// A service wrapper for the Miden client that provides coordination and background sync.
///
/// `ClientService` adds the following on top of the base `Client`:
///
/// - **Operation serialization**: All operations are serialized through a single mutex, ensuring
///   sync and transaction operations never overlap.
///
/// - **Background sync**: Optional periodic synchronization that runs in the background.
///
/// The service is `Send + Sync` and can be safely shared across tasks. Access the
/// underlying client via [`client()`](Self::client) for any operation, or use the
/// convenience methods [`sync_state()`](Self::sync_state) and
/// [`submit_transaction()`](Self::submit_transaction).
pub struct ClientService<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    /// The underlying client, wrapped in a mutex for interior mutability.
    client: Mutex<Client<AUTH>>,
    /// Service configuration.
    config: ServiceConfig,
}

impl<AUTH> ClientService<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    /// Creates a new service wrapping the given client.
    pub fn new(client: Client<AUTH>, config: ServiceConfig) -> Self {
        Self { client: Mutex::new(client), config }
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
    ///
    /// The returned [`MutexGuard`] dereferences to `Client<AUTH>`, supporting both
    /// shared (`&`) and mutable (`&mut`) access. The lock is held until the guard
    /// is dropped.
    ///
    /// Use this for any client operation that isn't covered by the convenience methods.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut client = service.client().await;
    /// let accounts = client.get_account_headers().await?;
    /// ```
    pub async fn client(&self) -> MutexGuard<'_, Client<AUTH>> {
        self.client.lock().await
    }

    /// Synchronizes the client state with the network.
    ///
    /// This acquires exclusive access to the client, fetches the latest state
    /// from the network, and applies it to the local store.
    pub async fn sync_state(&self) -> Result<SyncSummary, ClientError> {
        debug!("Starting coordinated sync");
        let mut client = self.client.lock().await;

        let state_sync_update = client.get_sync_update().await?;
        let summary: SyncSummary = (&state_sync_update).into();
        client.apply_state_sync(state_sync_update).await?;

        info!(block_num = ?summary.block_num, "Sync completed");
        Ok(summary)
    }

    /// Submits a new transaction.
    ///
    /// This acquires exclusive access to the client and submits the transaction.
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
    /// The sync uses the interval configured in [`ServiceConfig::sync_interval`].
    ///
    /// If background sync is disabled in the config, returns a handle that does nothing.
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
}

/// A handle to control background sync operations.
pub struct BackgroundSyncHandle {
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl BackgroundSyncHandle {
    fn new(shutdown_tx: broadcast::Sender<()>) -> Self {
        Self { shutdown_tx: Some(shutdown_tx) }
    }

    /// Signals the background sync to stop.
    ///
    /// The sync will complete its current operation before stopping.
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

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn service_is_send_sync() {
        assert_send_sync::<ClientService<()>>();
    }
}
