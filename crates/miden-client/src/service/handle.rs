use std::marker::PhantomData;
use std::sync::Arc;

use miden_client_core::account::AccountId;
use miden_client_core::store::Store;
use miden_client_core::sync::SyncSummary;
use miden_client_core::transaction::{
    TransactionAuthenticator, TransactionRequest, TransactionResult,
};
use miden_client_core::{Client, ClientError};
use tokio::sync::{Semaphore, mpsc, oneshot};
use tokio::task::{self, JoinHandle};

use super::config::{ClientServiceConfig, ClientServiceError};
use super::runtime::{Command, TransactionCommand, run_service};

/// High-level controller that wires background sync and transaction handling around a [`Client`].
pub struct ClientService<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    handle: ClientServiceHandle<AUTH>,
    join_handle: JoinHandle<Result<(), ClientError>>,
}

impl<AUTH> ClientService<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    /// Starts the service using the provided client and configuration.
    pub fn start(client: Client<AUTH>, config: ClientServiceConfig) -> Self {
        let store = client.store();
        let (command_tx, command_rx) = mpsc::channel(config.command_buffer);
        let (proven_tx, proven_rx) = mpsc::channel(config.proof_buffer);
        let proof_limiter = Arc::new(Semaphore::new(config.max_parallel_proofs.max(1)));

        let join_handle = task::spawn_local(run_service(
            client,
            config.clone(),
            command_rx,
            proven_rx,
            proof_limiter,
            proven_tx.clone(),
        ));

        let handle = ClientServiceHandle {
            store,
            command_tx: command_tx.clone(),
            transaction: TransactionServiceHandle {
                command_tx: command_tx.clone(),
                _auth: PhantomData,
            },
            _auth: PhantomData,
        };

        Self { handle, join_handle }
    }

    /// Returns a handle that can be used to interact with the running service.
    pub fn handle(&self) -> ClientServiceHandle<AUTH> {
        self.handle.clone()
    }

    /// Stops the service and awaits the completion of background tasks.
    pub async fn shutdown(self) -> Result<(), ClientServiceError> {
        self.handle.shutdown().await?;
        self.join_handle
            .await
            .map_err(ClientServiceError::Join)?
            .map_err(ClientServiceError::from)
    }
}

/// Lightweight handle for interacting with a running [`ClientService`].
pub struct ClientServiceHandle<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    pub(crate) store: Arc<dyn Store>,
    pub(crate) command_tx: mpsc::Sender<Command>,
    pub(crate) transaction: TransactionServiceHandle<AUTH>,
    pub(crate) _auth: PhantomData<AUTH>,
}

impl<AUTH> Clone for ClientServiceHandle<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
            command_tx: self.command_tx.clone(),
            transaction: self.transaction.clone(),
            _auth: PhantomData,
        }
    }
}

impl<AUTH> ClientServiceHandle<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    /// Exposes the underlying store so callers can issue read operations directly.
    pub fn store(&self) -> Arc<dyn Store> {
        Arc::clone(&self.store)
    }

    /// Returns a handle that can be used to submit transactions through the service.
    pub fn transaction_service(&self) -> TransactionServiceHandle<AUTH> {
        self.transaction.clone()
    }

    /// Triggers a sync immediately and returns the resulting [`SyncSummary`].
    pub async fn sync_now(&self) -> Result<SyncSummary, ClientServiceError> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(Command::SyncNow { respond_to: tx })
            .await
            .map_err(|_| ClientServiceError::ServiceClosed)?;
        let result = rx.await.map_err(|_| ClientServiceError::ServiceClosed)?;
        result.map_err(ClientServiceError::from)
    }

    pub async fn shutdown(self) -> Result<(), ClientServiceError> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(Command::Shutdown { respond_to: tx })
            .await
            .map_err(|_| ClientServiceError::ServiceClosed)?;
        rx.await.map_err(|_| ClientServiceError::ServiceClosed)?;
        Ok(())
    }
}

/// Handle used to enqueue transactions into the service pipeline.
pub struct TransactionServiceHandle<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    pub(crate) command_tx: mpsc::Sender<Command>,
    pub(crate) _auth: PhantomData<AUTH>,
}

impl<AUTH> Clone for TransactionServiceHandle<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            command_tx: self.command_tx.clone(),
            _auth: PhantomData,
        }
    }
}

impl<AUTH> TransactionServiceHandle<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    /// Requests execution/proving/submission of a transaction for the specified account.
    pub async fn submit_transaction(
        &self,
        account_id: AccountId,
        request: TransactionRequest,
    ) -> Result<TransactionJob, ClientServiceError> {
        let (execution_tx, execution_rx) = oneshot::channel();
        let (completion_tx, completion_rx) = oneshot::channel();

        let command = Command::Transaction(TransactionCommand {
            account_id,
            request,
            execution: execution_tx,
            completion: completion_tx,
        });

        self.command_tx
            .send(command)
            .await
            .map_err(|_| ClientServiceError::ServiceClosed)?;

        Ok(TransactionJob {
            execution: execution_rx,
            completion: completion_rx,
        })
    }
}

/// Represents the asynchronous stages of a transaction submitted through the service.
pub struct TransactionJob {
    /// Resolves when execution finishes (or fails) with the produced [`TransactionResult`].
    pub execution: oneshot::Receiver<Result<TransactionResult, ClientError>>,
    /// Resolves when the transaction has been submitted and applied locally.
    pub completion: oneshot::Receiver<Result<(), ClientError>>,
}
