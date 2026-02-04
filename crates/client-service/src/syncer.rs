//! Syncer trait for customizable sync behavior.

use async_trait::async_trait;
use miden_client::auth::TransactionAuthenticator;
use miden_client::sync::StateSyncUpdate;
use miden_client::{Client, ClientError};

/// Trait for customizing sync behavior.
///
/// Implementations control what data is synced and how. The `ClientService` uses
/// this trait for both manual `sync_state()` calls and background sync.
///
/// # Example
///
/// ```rust,ignore
/// use miden_client_service::{Syncer, DefaultSyncer};
///
/// // Use the default syncer (standard client sync behavior)
/// let syncer = DefaultSyncer;
///
/// // Or implement your own
/// struct CustomSyncer {
///     note_tags: BTreeSet<NoteTag>,
/// }
///
/// #[async_trait]
/// impl<AUTH> Syncer<AUTH> for CustomSyncer
/// where
///     AUTH: TransactionAuthenticator + Send + Sync + 'static,
/// {
///     async fn sync(&self, client: &Client<AUTH>) -> Result<StateSyncUpdate, ClientError> {
///         // Custom sync logic using self.note_tags
///     }
/// }
/// ```
#[async_trait]
pub trait Syncer<AUTH>: Send + Sync
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    /// Performs a sync operation and returns the update to apply.
    ///
    /// The implementation has full control over:
    /// - Which accounts to sync
    /// - Which note tags to request
    /// - Which notes/transactions to track
    /// - How to handle received notes
    ///
    /// The returned `StateSyncUpdate` will be applied to the store by the `ClientService`.
    async fn sync(&self, client: &Client<AUTH>) -> Result<StateSyncUpdate, ClientError>;
}

/// Default syncer that uses the standard `Client::sync_state()` behavior.
///
/// This syncer:
/// - Syncs all tracked accounts
/// - Requests notes for all stored note tags
/// - Tracks all unspent input/output notes
/// - Uses the `NoteScreener` to filter received notes
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultSyncer;

#[async_trait]
impl<AUTH> Syncer<AUTH> for DefaultSyncer
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    async fn sync(&self, client: &Client<AUTH>) -> Result<StateSyncUpdate, ClientError> {
        client.get_sync_update().await
    }
}
