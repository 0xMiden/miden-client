use std::sync::Arc;

use miden_client_core::Client;
use miden_client_core::transaction::TransactionAuthenticator;
use tokio::select;
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use super::config::ClientServiceConfig;
use super::error::ClientServiceError;
use super::event::SyncEvent;
use super::handle::ClientHandle;
use super::inner::ServiceInner;

pub struct ClientRuntime<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    inner: Arc<ServiceInner<AUTH>>,
    sync_task: Option<JoinHandle<()>>,
}

impl<AUTH> std::fmt::Debug for ClientRuntime<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientRuntime").finish_non_exhaustive()
    }
}

impl<AUTH> ClientRuntime<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    pub async fn start(
        client: Client<AUTH>,
        config: ClientServiceConfig,
    ) -> Result<Self, ClientServiceError> {
        let inner = ServiceInner::new(client, config);
        let handle = ClientHandle::new(inner.clone());

        if handle.config().initial_sync {
            let summary = ServiceInner::sync_once(&inner).await?;
            let event = SyncEvent::new(summary.clone());
            ServiceInner::dispatch_handlers(&inner, handle.clone(), event).await?;
        }

        let sync_task = Some(spawn_sync_loop(inner.clone()));

        Ok(Self { inner, sync_task })
    }

    pub fn handle(&self) -> ClientHandle<AUTH> {
        ClientHandle::new(self.inner.clone())
    }

    pub fn config(&self) -> ClientServiceConfig {
        self.inner.config.clone()
    }

    pub async fn shutdown(mut self) {
        self.inner.signal_shutdown();
        if let Some(task) = self.sync_task.take() {
            task.abort();
            let _ = task.await;
        }
    }
}

impl<AUTH> Drop for ClientRuntime<AUTH>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.inner.signal_shutdown();
        if let Some(task) = &self.sync_task {
            task.abort();
        }
    }
}

fn spawn_sync_loop<AUTH>(inner: Arc<ServiceInner<AUTH>>) -> JoinHandle<()>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    tokio::spawn(async move {
        let mut interval = inner.config.sync_interval.map(tokio::time::interval);
        let handle = ClientHandle::new(inner.clone());

        loop {
            if inner.is_shutdown() {
                break;
            }

            match &mut interval {
                Some(interval) => {
                    select! {
                        _ = interval.tick() => {},
                        _ = inner.sync_trigger.notified() => {},
                    }
                },
                None => {
                    inner.sync_trigger.notified().await;
                },
            }

            if inner.is_shutdown() {
                break;
            }

            match ServiceInner::sync_once(&inner).await {
                Ok(summary) => {
                    let event = SyncEvent::new(summary.clone());
                    if let Err(err) =
                        ServiceInner::dispatch_handlers(&inner, handle.clone(), event).await
                    {
                        error!(?err, "error running sync handlers");
                    } else {
                        trace!("client state synced");
                    }
                },
                Err(err) => error!(?err, "background sync failed"),
            }
        }

        debug!("client sync loop stopped");
    })
}
