use alloc::boxed::Box;
use alloc::sync::Arc;

use miden_lib::account::interface::AccountInterface;
use miden_objects::account::Account;
use miden_objects::note::Note;
use miden_tx::auth::TransactionAuthenticator;
use miden_tx::{NoteConsumptionChecker, TransactionExecutor};

use super::note_screener::NoteScreenerError;
use crate::note::NoteRelevance;
use crate::store::Store;
use crate::store::data_store::ClientDataStore;
use crate::transaction::TransactionRequestBuilder;

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait NoteConsumabilityChecker: Send + Sync {
    async fn check_notes_consumability(
        &self,
        account: &Account,
        note: &Note,
    ) -> Result<Option<NoteRelevance>, NoteScreenerError>;
}

/// Default checker that tries to consume the note in a transaction to see if it's consumable.
pub struct StandardConsumabilityChecker<AUTH> {
    store: Arc<dyn Store>,
    authenticator: Arc<AUTH>,
}

impl<AUTH> StandardConsumabilityChecker<AUTH> {
    pub fn new(store: Arc<dyn Store>, authenticator: Arc<AUTH>) -> Self {
        Self { store, authenticator }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl<AUTH> NoteConsumabilityChecker for StandardConsumabilityChecker<AUTH>
where
    AUTH: TransactionAuthenticator + Sync + Send,
{
    async fn check_notes_consumability(
        &self,
        account: &Account,
        note: &Note,
    ) -> Result<Option<NoteRelevance>, NoteScreenerError> {
        let transaction_request =
            TransactionRequestBuilder::new().build_consume_notes(vec![note.id()])?;

        let tx_script = transaction_request.build_transaction_script(
            &AccountInterface::from(account),
            crate::DebugMode::Enabled,
        )?;

        let tx_args = transaction_request.clone().into_transaction_args(tx_script, vec![]);

        // Build a fresh executor per call to avoid lifetime issues.
        let data_store = ClientDataStore::new(self.store.clone());
        let mut transaction_executor = TransactionExecutor::new(&data_store);
        transaction_executor = transaction_executor.with_authenticator(self.authenticator.as_ref());

        let consumption_checker = NoteConsumptionChecker::new(&transaction_executor);

        data_store.mast_store().load_account_code(account.code());
        let note_execution_check = consumption_checker
            .check_notes_consumability(
                account.id(),
                self.store.get_sync_height().await?,
                vec![note.clone()],
                tx_args,
            )
            .await?;

        if !note_execution_check.successful.is_empty() {
            return Ok(Some(NoteRelevance::Now));
        }
        Ok(None)
    }
}
