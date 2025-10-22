use std::path::PathBuf;
use std::sync::Arc;

use miden_client::builder::{BuilderAuthenticator, ClientBuilder, StoreBuilder, StoreFactory};
use miden_client::store::{Store, StoreError};

use crate::SqliteStore;

/// Extends the [`ClientBuilder`] with a method to add a [`SqliteStore`].
pub trait ClientBuilderSqliteExt<AUTH> {
    fn sqlite_store(self, database_filepath: PathBuf) -> ClientBuilder<AUTH>;
}

impl<AUTH: BuilderAuthenticator> ClientBuilderSqliteExt<AUTH> for ClientBuilder<AUTH> {
    /// Sets a [`SqliteStore`] to the [`ClientBuilder`]. The store will be instantiated when the
    /// [`build`](ClientBuilder::build) method is called.
    fn sqlite_store(mut self, database_filepath: PathBuf) -> ClientBuilder<AUTH> {
        self.store =
            Some(StoreBuilder::Factory(Box::new(SqliteStoreFactory { database_filepath })));
        self
    }
}

/// Factory for building a [`SqliteStore`].
struct SqliteStoreFactory {
    database_filepath: PathBuf,
}

#[async_trait::async_trait]
impl StoreFactory for SqliteStoreFactory {
    async fn build(&self) -> Result<Arc<dyn Store>, StoreError> {
        let sqlite_store = SqliteStore::new(self.database_filepath.clone()).await?;
        Ok(Arc::new(sqlite_store))
    }
}
