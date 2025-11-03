use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use miden_client_core::Client;
use miden_client_core::store::Store;
use miden_client_core::sync::SyncSummary;
use miden_client_core::transaction::TransactionAuthenticator;
use tokio::sync::{Mutex, Notify, RwLock};
use tracing::{error, trace};

use super::config::ClientServiceConfig;
use super::error::{ClientServiceError, HandlerError};
use super::event::{HandlerId, SyncEvent};
use super::handle::ClientHandle;

pub(super) type BlockingHandlerFuture =
    Pin<Box<dyn Future<Output = Result<(), HandlerError>> + Send + 'static>>;
pub(super) type NonBlockingHandlerFuture =
    Pin<Box<dyn Future<Output = Result<(), HandlerError>> + Send + 'static>>;

pub(super) type BlockingHandler<AUTH> =
    Arc<dyn Fn(ClientHandle<AUTH>, SyncEvent) -> BlockingHandlerFuture + Send + Sync>;
pub(super) type NonBlockingHandler<AUTH> =
    Arc<dyn Fn(ClientHandle<AUTH>, SyncEvent) -> NonBlockingHandlerFuture + Send + Sync>;

pub(super) struct ServiceInner<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    pub(super) client: Mutex<Client<AUTH>>,
    pub(super) store: Arc<dyn Store>,
    pub(super) blocking_handlers: RwLock<HashMap<HandlerId, BlockingHandler<AUTH>>>,
    pub(super) non_blocking_handlers: RwLock<HashMap<HandlerId, NonBlockingHandler<AUTH>>>,
    pub(super) handler_seq: AtomicU64,

    pub(super) sync_trigger: Notify,
    pub(super) shutdown: AtomicBool,
    pub(super) sync_lock: Mutex<()>,
    pub(super) config: ClientServiceConfig,
}

impl<AUTH> ServiceInner<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    pub(super) fn new(client: Client<AUTH>, config: ClientServiceConfig) -> Arc<Self> {
        Arc::new(Self {
            store: client.store(),
            client: Mutex::new(client),
            blocking_handlers: RwLock::new(HashMap::new()),
            non_blocking_handlers: RwLock::new(HashMap::new()),
            handler_seq: AtomicU64::new(0),
            sync_trigger: Notify::new(),
            shutdown: AtomicBool::new(false),
            sync_lock: Mutex::new(()),
            config,
        })
    }

    pub(super) fn store(&self) -> Arc<dyn Store> {
        self.store.clone()
    }

    pub(super) fn generate_handler_id(&self) -> HandlerId {
        HandlerId::next(&self.handler_seq)
    }

    pub(super) fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    pub(super) fn signal_shutdown(&self) {
        if !self.shutdown.swap(true, Ordering::Relaxed) {
            self.sync_trigger.notify_waiters();
        }
    }

    pub(super) async fn sync_once(this: &Arc<Self>) -> Result<SyncSummary, ClientServiceError> {
        let _sync_guard = this.sync_lock.lock().await;
        let mut client = this.client.lock().await;
        let summary = client.sync_state().await?;
        drop(client);

        Ok(summary)
    }

    pub(super) async fn dispatch_handlers(
        this: &Arc<Self>,
        handle: ClientHandle<AUTH>,
        event: SyncEvent,
    ) -> Result<(), ClientServiceError> {
        let blocking_handlers: Vec<_> = {
            let guard = this.blocking_handlers.read().await;
            guard.values().cloned().collect()
        };

        for handler in blocking_handlers {
            handler(handle.clone(), event.clone()).await?;
        }

        let non_blocking_handlers: Vec<_> = {
            let guard = this.non_blocking_handlers.read().await;
            guard.values().cloned().collect()
        };

        for handler in non_blocking_handlers {
            let handle = handle.clone();
            let event = event.clone();
            tokio::spawn(async move {
                if let Err(err) = handler(handle, event).await {
                    error!(?err, "non-blocking sync handler failed");
                }
            });
        }

        trace!("handlers dispatched");

        Ok(())
    }
}
