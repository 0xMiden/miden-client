use core::future::Future;
use core::pin::Pin;
use std::sync::Arc;

use miden_client_core::Client;
use miden_client_core::account::AccountId;
use miden_client_core::sync::SyncSummary;
use miden_client_core::transaction::{TransactionAuthenticator, TransactionId, TransactionRequest};
use tokio::sync::{mpsc, oneshot};
use tokio::task::{JoinHandle, spawn_local};
use tokio::time;
use tracing::{error, warn};

use super::{ServiceConfig, ServiceError};

// HANDLER TYPES
// ================================================================================================

/// Trait for handles that run after every sync and blocks the sync loop until it completes.
///
/// Handlers stay registered until the service stops, so they can maintain internal state. They
/// also receive a mutable reference to the client, allowing serialized, async follow-up work
/// (e.g. enqueueing another transaction or updating tracked tags).
pub trait BlockingHandler<AUTH>: Send {
    fn call<'a>(
        &'a mut self,
        client: &'a mut Client<AUTH>,
        summary: &'a SyncSummary,
    ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>>;
}

impl<AUTH, F> BlockingHandler<AUTH> for F
where
    F: for<'a> FnMut(
            &'a mut Client<AUTH>,
            &'a SyncSummary,
        )
            -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>>
        + Send,
{
    fn call<'a>(
        &'a mut self,
        client: &'a mut Client<AUTH>,
        summary: &'a SyncSummary,
    ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>> {
        (self)(client, summary)
    }
}

pub type BlockingHandlerBox<AUTH> = Box<dyn BlockingHandler<AUTH> + Send>;
type ClientJob<AUTH> = Box<
    dyn for<'a> FnMut(
            &'a mut Client<AUTH>,
        )
            -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>>
        + Send
        + 'static,
>;

/// Handler that runs after a sync without blocking the sync loop.
///
/// Async handlers receive a fresh [`SyncSummary`] and a cheap [`ClientHandle`] clone. Use this to
/// kick off side work without stalling the main sync cycle
pub type AsyncHandler<AUTH> = Arc<
    dyn Fn(SyncSummary, ClientHandle<AUTH>) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync
        + 'static,
>;

// CLIENT HANDLE
// ================================================================================================

/// Handle for interacting with a running client service.
pub struct ClientHandle<AUTH> {
    command_tx: mpsc::Sender<Command<AUTH>>,
}

impl<AUTH> Clone for ClientHandle<AUTH> {
    fn clone(&self) -> Self {
        Self { command_tx: self.command_tx.clone() }
    }
}

impl<AUTH> ClientHandle<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    pub(crate) fn new(command_tx: mpsc::Sender<Command<AUTH>>) -> Self {
        Self { command_tx }
    }

    /// Enqueues a transaction request to run sequentially.
    pub async fn submit_transaction(
        &self,
        account_id: AccountId,
        tx_request: TransactionRequest,
    ) -> Result<TransactionId, ServiceError> {
        // TODO: this could be abstracted away into a transaction queue where all steps can be done
        // in parallel, except the submitting + applying step, which should happen in order.
        let (resp_tx, resp_rx) = oneshot::channel();
        self.command_tx
            .send(Command::SubmitTransaction { account_id, tx_request, resp_tx })
            .await
            .map_err(|_| ServiceError::ChannelClosed)?;

        resp_rx
            .await
            .map_err(|_| ServiceError::ChannelClosed)?
            .map_err(ServiceError::from)
    }

    /// Adds a new blocking handler.
    ///
    /// Blocking handlers run after every sync and stay registered until the service shuts
    /// down. Use this for stateful observers that must finish before the next sync cycle
    /// proceeds.
    pub async fn add_blocking_handler(
        &self,
        handler: BlockingHandlerBox<AUTH>,
    ) -> Result<(), ServiceError> {
        self.command_tx
            .send(Command::AddBlockingHandler(handler))
            .await
            .map_err(|_| ServiceError::ChannelClosed)
    }

    /// Adds a new non-blocking handler that runs after each sync.
    pub async fn add_async_handler(&self, handler: AsyncHandler<AUTH>) -> Result<(), ServiceError> {
        self.command_tx
            .send(Command::AddAsyncHandler(handler))
            .await
            .map_err(|_| ServiceError::ChannelClosed)
    }

    /// Gracefully stops the service.
    pub async fn stop(&self) -> Result<(), ServiceError> {
        self.command_tx
            .send(Command::Stop)
            .await
            .map_err(|_| ServiceError::ChannelClosed)
    }

    /// Runs a one-shot function with mutalbe access to the client.
    ///
    /// This is useful for ad-hoc operations that should not be stored as persistent handlers, like
    /// submitting a single transaction or updating configuration once:
    ///
    /// ```rust
    /// # use miden_client::service::ServiceError;
    /// # use miden_client::account::AccountId;
    /// # use miden_client::transaction::TransactionRequestBuilder;
    /// # async fn demo(handle: miden_client::service::ClientHandle<_>, account_id: AccountId) -> Result<(), ServiceError> {
    /// let tx_request = TransactionRequestBuilder::new().build()
    ///     .unwrap();
    ///
    /// handle.run_with_client(move |client| {
    ///     Box::pin(async move {
    ///         client.submit_new_transaction(account_id, tx_request).await.map_err(ServiceError::from)?;
    ///         Ok(())
    ///     })
    /// }).await?;
    /// # Ok(()) }
    /// ```
    pub async fn run_with_client<F, Fut>(&self, f: F) -> Result<(), ServiceError>
    where
        F: FnOnce(&mut Client<AUTH>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), ServiceError>> + Send + 'static,
    {
        let (resp_tx, resp_rx) = oneshot::channel();
        let mut f = Some(f);
        let job: ClientJob<AUTH> = Box::new(move |client| {
            let fut = (f.take().unwrap())(client);
            Box::pin(async move { fut.await })
        });
        self.command_tx
            .send(Command::RunWithClient { f: job, resp_tx })
            .await
            .map_err(|_| ServiceError::ChannelClosed)?;

        resp_rx.await.map_err(|_| ServiceError::ChannelClosed)?
    }
}

// CLIENT SERVICE
// ================================================================================================

/// A running client service and its handle.
pub struct ClientService<AUTH> {
    handle: ClientHandle<AUTH>,
    task: JoinHandle<()>,
}

impl<AUTH> ClientService<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    /// Spawns the client service on a Tokio local task using the built-in sync flow.
    pub fn spawn(client: Client<AUTH>, config: ServiceConfig) -> Self {
        let (command_tx, command_rx) = mpsc::channel(config.command_buffer);
        let handle = ClientHandle::new(command_tx.clone());
        let runtime = Runtime::new(client, command_rx, handle.clone(), config);
        // TODO: Add the fact that this runs with spawn_local to docs
        let task = spawn_local(runtime.run());

        Self { handle, task }
    }

    /// Returns a cloneable handle to interact with the running service.
    pub fn handle(&self) -> ClientHandle<AUTH> {
        self.handle.clone()
    }

    /// Stops the service and waits for the runtime task to finish.
    pub async fn shutdown(self) -> Result<(), ServiceError> {
        let _ = self.handle.stop().await;
        self.task.await.map_err(|_| ServiceError::Shutdown)?;
        Ok(())
    }
}

// RUNTIME
// ================================================================================================

struct Runtime<AUTH> {
    client: Client<AUTH>,
    command_rx: mpsc::Receiver<Command<AUTH>>,
    handle: ClientHandle<AUTH>,
    config: ServiceConfig,
    // TODO: structs for the queues and maybe type aliases for the inner trait objects
    registered_blocking_handlers: Vec<BlockingHandlerBox<AUTH>>,
    registered_async_handlers: Vec<AsyncHandler<AUTH>>,
}

impl<AUTH> Runtime<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    pub fn new(
        client: Client<AUTH>,
        command_rx: mpsc::Receiver<Command<AUTH>>,
        handle: ClientHandle<AUTH>,
        config: ServiceConfig,
    ) -> Self {
        Self {
            client,
            command_rx,
            handle,
            config,
            registered_blocking_handlers: Vec::new(),
            registered_async_handlers: Vec::new(),
        }
    }

    pub async fn run(mut self) {
        let mut interval = time::interval(self.config.sync_interval);

        loop {
            tokio::select! {
                _ = interval.tick(), if self.config.auto_sync => {
                    if let Err(err) = self.sync_and_dispatch().await {
                        error!(?err, "background sync failed");
                    }
                }
                maybe_cmd = self.command_rx.recv() => {
                    match maybe_cmd {
                        Some(cmd) => {
                            if matches!(cmd, Command::Stop) {
                                break;
                            }
                            if let Err(err) = self.handle_command(cmd).await {
                                error!(?err, "command failed");
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    }

    async fn sync_and_dispatch(&mut self) -> Result<SyncSummary, ServiceError> {
        let state_sync_update =
            self.client.sync_state_update().await.map_err(ServiceError::from)?;
        let summary: SyncSummary = (&state_sync_update).into();

        self.client
            .apply_state_sync(state_sync_update)
            .await
            .map_err(ServiceError::from)?;

        for handler in &mut self.registered_blocking_handlers {
            if let Err(err) = handler.call(&mut self.client, &summary).await {
                warn!(?err, "blocking handler returned error");
            }
        }

        // TODO: Run these in parallel?
        for handler in &self.registered_async_handlers {
            let handler = handler.clone();
            let summary = summary.clone();
            let handle = self.handle.clone();
            spawn_local(async move {
                handler(summary, handle).await;
            });
        }

        Ok(summary)
    }

    async fn handle_command(&mut self, command: Command<AUTH>) -> Result<(), ServiceError> {
        match command {
            Command::SubmitTransaction { account_id, tx_request, resp_tx } => {
                let result = self
                    .client
                    .submit_new_transaction(account_id, tx_request)
                    .await
                    .map_err(ServiceError::from);
                let _ = resp_tx.send(result);
            },
            Command::AddBlockingHandler(handler) => self.registered_blocking_handlers.push(handler),
            Command::AddAsyncHandler(handler) => self.registered_async_handlers.push(handler),
            Command::RunWithClient { mut f, resp_tx } => {
                let result = f(&mut self.client).await;
                let _ = resp_tx.send(result);
            },
            Command::Stop => {},
        }

        Ok(())
    }
}

// COMMANDS
// ================================================================================================

pub(crate) enum Command<AUTH> {
    SubmitTransaction {
        account_id: AccountId,
        tx_request: TransactionRequest,
        resp_tx: oneshot::Sender<Result<TransactionId, ServiceError>>,
    },
    AddBlockingHandler(BlockingHandlerBox<AUTH>),
    AddAsyncHandler(AsyncHandler<AUTH>),
    RunWithClient {
        f: ClientJob<AUTH>,
        resp_tx: oneshot::Sender<Result<(), ServiceError>>,
    },
    Stop,
}

// TODO: re-add tests
