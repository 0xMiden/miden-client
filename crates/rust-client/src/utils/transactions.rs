use std::vec::Vec;

use miden_objects::{account::AccountId, note::Note, transaction::TransactionId};

use crate::{
    Client, ClientError,
    transaction::{NoteArgs, TransactionRequest, TransactionRequestBuilder},
};

impl Client {
    /// Executes a transaction and returns the transaction ID.
    pub async fn execute_tx(
        &mut self,
        account_id: AccountId,
        tx_request: TransactionRequest,
    ) -> Result<TransactionId, ClientError> {
        let transaction_execution_result = self.new_transaction(account_id, tx_request).await?;
        let transaction_id = transaction_execution_result.executed_transaction().id();

        self.submit_transaction(transaction_execution_result).await?;

        Ok(transaction_id)
    }

    /// Executes a transaction and waits for it to be committed.
    ///
    /// # Panics
    /// - If the transaction is not found in the client's state.
    pub async fn execute_tx_and_sync(
        &mut self,
        account_id: AccountId,
        tx_request: TransactionRequest,
    ) -> Result<(), ClientError> {
        let transaction_id = self.execute_tx(account_id, tx_request).await?;
        self.wait_for_tx(transaction_id).await
    }

    /// Syncs the client and waits for the transaction to be committed.
    ///
    /// # Panics
    /// - If the transaction is discarded.
    pub async fn wait_for_tx(&mut self, transaction_id: TransactionId) -> Result<(), ClientError> {
        self.wait_until(|summary| {
            assert!(
                !summary.discarded_transactions.iter().any(|id| id == &transaction_id),
                "Transaction was discarded before it was committed"
            );

            summary.committed_transactions.iter().any(|id| id == &transaction_id)
        })
        .await
    }

    /// Executes a transaction and consumes the resulting unauthenticated notes immediately without
    /// waiting for the first transaction to be committed.
    ///
    /// # Panics
    /// - If the transaction is discarded.
    pub async fn execute_tx_and_consume_output_notes(
        &mut self,
        tx_request: TransactionRequest,
        executor: AccountId,
        consumer: AccountId,
    ) -> Result<(), ClientError> {
        let output_notes = tx_request
            .expected_output_own_notes()
            .into_iter()
            .map(|note| (note, None::<NoteArgs>))
            .collect::<Vec<(Note, Option<NoteArgs>)>>();

        self.execute_tx(executor, tx_request).await?;

        let tx_request = TransactionRequestBuilder::new()
            .unauthenticated_input_notes(output_notes)
            .build()?;
        let transaction_id = self.execute_tx(consumer, tx_request).await?;
        self.wait_for_tx(transaction_id).await
    }
}
