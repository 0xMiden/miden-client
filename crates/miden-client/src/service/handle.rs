use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use miden_client_core::account::AccountId;
use miden_client_core::block::BlockNumber;
use miden_client_core::note::NoteTag;
use miden_client_core::store::Store;
use miden_client_core::transaction::{
    ProvenTransaction,
    TransactionAuthenticator,
    TransactionInputs,
    TransactionProver,
    TransactionRequest,
    TransactionResult,
};
use miden_client_core::{Client, ClientError};

use super::config::ClientServiceConfig;
use super::error::{ClientServiceError, HandlerError};
use super::event::{HandlerId, SyncEvent};
use super::inner::{BlockingHandlerFuture, NonBlockingHandlerFuture, ServiceInner};

pub(super) type ClientFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ClientError>> + 'a>>;

pub struct ClientHandle<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    pub(super) inner: Arc<ServiceInner<AUTH>>,
}

impl<AUTH> Clone for ClientHandle<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

impl<AUTH> ClientHandle<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    pub(super) fn new(inner: Arc<ServiceInner<AUTH>>) -> Self {
        Self { inner }
    }

    pub fn config(&self) -> ClientServiceConfig {
        self.inner.config.clone()
    }

    pub fn store(&self) -> Arc<dyn Store> {
        self.inner.store()
    }

    pub async fn with_client<T, F>(&self, f: F) -> Result<T, ClientServiceError>
    where
        F: for<'a> FnOnce(&'a mut Client<AUTH>) -> ClientFuture<'a, T>,
    {
        if self.inner.is_shutdown() {
            return Err(ClientServiceError::ShuttingDown);
        }

        let mut guard = self.inner.client.lock().await;
        let result = f(&mut *guard).await?;
        Ok(result)
    }

    pub async fn sync_now(&self) -> Result<SyncEvent, ClientServiceError> {
        if self.inner.is_shutdown() {
            return Err(ClientServiceError::ShuttingDown);
        }

        let summary = ServiceInner::sync_once(&self.inner).await?;
        let event = SyncEvent::new(summary.clone());
        ServiceInner::dispatch_handlers(&self.inner, self.clone(), event.clone()).await?;
        Ok(event)
    }

    pub fn trigger_sync(&self) {
        self.inner.sync_trigger.notify_one();
    }

    pub async fn register_blocking_handler<F, Fut>(&self, handler: F) -> HandlerId
    where
        F: Fn(ClientHandle<AUTH>, SyncEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), HandlerError>> + Send + 'static,
    {
        let id = self.inner.generate_handler_id();
        let handler = Arc::new(move |handle: ClientHandle<AUTH>, event: SyncEvent| {
            Box::pin(handler(handle, event)) as BlockingHandlerFuture
        });

        self.inner.blocking_handlers.write().await.insert(id, handler);
        id
    }

    pub async fn register_non_blocking_handler<F, Fut>(&self, handler: F) -> HandlerId
    where
        F: Fn(ClientHandle<AUTH>, SyncEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), HandlerError>> + Send + 'static,
    {
        let id = self.inner.generate_handler_id();
        let handler = Arc::new(move |handle: ClientHandle<AUTH>, event: SyncEvent| {
            Box::pin(handler(handle, event)) as NonBlockingHandlerFuture
        });

        self.inner.non_blocking_handlers.write().await.insert(id, handler);
        id
    }

    pub async fn unregister_blocking_handler(&self, id: HandlerId) {
        self.inner.blocking_handlers.write().await.remove(&id);
    }

    pub async fn unregister_non_blocking_handler(&self, id: HandlerId) {
        self.inner.non_blocking_handlers.write().await.remove(&id);
    }

    pub async fn execute_transaction(
        &self,
        account_id: AccountId,
        request: TransactionRequest,
    ) -> Result<TransactionResult, ClientServiceError> {
        self.with_client(move |client| {
            Box::pin(async move { client.execute_transaction(account_id, request).await })
        })
        .await
    }

    pub async fn prove_transaction(
        &self,
        tx_result: &TransactionResult,
        prover: Option<Arc<dyn TransactionProver>>,
    ) -> Result<ProvenTransaction, ClientServiceError> {
        let prover = if let Some(prover) = prover {
            prover
        } else {
            let client = self.inner.client.lock().await;
            client.transaction_prover()
        };

        let executed = tx_result.executed_transaction().clone().into();
        let proven_tx = prover.prove(executed).await.map_err(ClientError::from)?;

        Ok(proven_tx)
    }

    pub async fn submit_proven_transaction(
        &self,
        proven_transaction: ProvenTransaction,
        tx_inputs: TransactionInputs,
    ) -> Result<BlockNumber, ClientServiceError> {
        self.with_client(move |client| {
            Box::pin(async move {
                client.submit_proven_transaction(proven_transaction, tx_inputs).await
            })
        })
        .await
    }

    pub async fn apply_transaction(
        &self,
        submission_height: BlockNumber,
        tx_result: TransactionResult,
    ) -> Result<(), ClientServiceError> {
        self.with_client(move |client| {
            Box::pin(async move { client.apply_transaction(&tx_result, submission_height).await })
        })
        .await
    }

    pub async fn prove_submit_and_apply(
        &self,
        tx_result: TransactionResult,
        prover: Option<Arc<dyn TransactionProver>>,
    ) -> Result<(), ClientServiceError> {
        let tx_inputs = tx_result.executed_transaction().tx_inputs().clone();
        let proven_tx = self.prove_transaction(&tx_result, prover).await?;
        let block_num = self.submit_proven_transaction(proven_tx, tx_inputs).await?;
        self.apply_transaction(block_num, tx_result).await
    }

    pub async fn execute_full_transaction(
        &self,
        account_id: AccountId,
        request: TransactionRequest,
        prover: Option<Arc<dyn TransactionProver>>,
    ) -> Result<(), ClientServiceError> {
        let tx_result = self.execute_transaction(account_id, request).await?;
        self.prove_submit_and_apply(tx_result, prover).await
    }

    pub async fn add_note_tag(&self, tag: NoteTag) -> Result<(), ClientServiceError> {
        self.with_client(move |client| Box::pin(async move { client.add_note_tag(tag).await }))
            .await
    }

    pub async fn remove_note_tag(&self, tag: NoteTag) -> Result<(), ClientServiceError> {
        self.with_client(move |client| Box::pin(async move { client.remove_note_tag(tag).await }))
            .await
    }
}
