//! Operation coordination for sync and transaction mutual exclusion.
//!
//! This module provides the [`OperationCoordinator`] which ensures:
//! - Only one sync or transaction runs at a time
//! - Transactions never overlap with sync operations

use tokio::sync::{Mutex, broadcast};

use crate::errors::ServiceError;

/// Coordinates sync and transaction operations to ensure mutual exclusion.
///
/// Features:
/// - **Sync exclusion**: Syncs are serialized and never overlap with transactions
/// - **Transaction exclusion**: Transactions are serialized and never overlap with syncs
pub struct OperationCoordinator {
    /// Guards all coordinated operations (syncs and transactions).
    operation_lock: Mutex<()>,
}

impl Default for OperationCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationCoordinator {
    /// Creates a new coordinator.
    pub fn new() -> Self {
        Self { operation_lock: Mutex::new(()) }
    }

    /// Executes a sync operation with mutual exclusion.
    ///
    /// Only one coordinated operation runs at a time; syncs are serialized
    /// and do not overlap with transactions.
    pub async fn with_sync<F, Fut, T>(&self, sync_fn: F) -> Result<T, ServiceError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, ServiceError>>,
    {
        let _guard = self.operation_lock.lock().await;
        sync_fn().await
    }

    /// Executes a transaction with proper coordination.
    ///
    /// This ensures:
    /// 1. No sync is running
    /// 2. No other transaction is running
    pub async fn with_transaction<F, Fut, T>(&self, tx_fn: F) -> Result<T, ServiceError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, ServiceError>>,
    {
        let _guard = self.operation_lock.lock().await;
        tx_fn().await
    }
}

/// A handle to control background sync operations.
pub struct BackgroundSyncHandle {
    /// Channel to signal shutdown.
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl BackgroundSyncHandle {
    /// Creates a new handle with the shutdown channel.
    pub(crate) fn new(shutdown_tx: broadcast::Sender<()>) -> Self {
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
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_sync_basic() {
        let coordinator = Arc::new(OperationCoordinator::new());
        let call_count = Arc::new(AtomicU32::new(0));

        let count = call_count.clone();
        let result = coordinator
            .with_sync(|| async {
                count.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok::<_, ServiceError>(42)
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_transaction_basic() {
        let coordinator = OperationCoordinator::new();

        let result = coordinator.with_transaction(|| async { Ok::<_, ServiceError>(42) }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_sequential_operations() {
        let coordinator = OperationCoordinator::new();
        let counter = std::sync::atomic::AtomicU32::new(0);

        // Run sync
        let _ = coordinator
            .with_sync(|| async {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ServiceError>(())
            })
            .await;

        // Run transaction
        let _ = coordinator
            .with_transaction(|| async {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ServiceError>(())
            })
            .await;

        // Both should have run
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}
