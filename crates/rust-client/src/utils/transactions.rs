use std::vec::Vec;

use miden_objects::{account::AccountId, note::Note, transaction::TransactionId};

use crate::{
    Client, ClientError,
    transaction::{NoteArgs, TransactionRequest, TransactionRequestBuilder},
};

/// Executes a transaction and returns the transaction ID.
pub async fn execute_tx(
    client: &mut Client,
    account_id: AccountId,
    tx_request: TransactionRequest,
) -> Result<TransactionId, ClientError> {
    let transaction_execution_result = client.new_transaction(account_id, tx_request).await?;
    let transaction_id = transaction_execution_result.executed_transaction().id();

    client.submit_transaction(transaction_execution_result).await?;

    Ok(transaction_id)
}

/// Executes a transaction and waits for it to be committed.
///
/// # Panics
/// - If the transaction is not found in the client's state.
pub async fn execute_tx_and_sync(
    client: &mut Client,
    account_id: AccountId,
    tx_request: TransactionRequest,
) -> Result<(), ClientError> {
    let transaction_id = execute_tx(client, account_id, tx_request).await?;
    wait_for_tx(client, transaction_id).await
}

/// Syncs the client and waits for the transaction to be committed.
///
/// # Panics
/// - If the transaction is discarded.
pub async fn wait_for_tx(
    client: &mut Client,
    transaction_id: TransactionId,
) -> Result<(), ClientError> {
    client
        .wait_until(|summary| {
            assert!(
                !summary.discarded_transactions.iter().any(|id| id == &transaction_id),
                "Transaction was discarded before it was committed"
            );

            summary.committed_transactions.iter().any(|id| id == &transaction_id)
        })
        .await
}

/// Executes a transaction and consumes the resulting unauthenticated notes inmediately without
/// waiting for the first transaction to be committed.
///
/// # Panics
/// - If the transaction is discarded.
pub async fn execute_tx_and_consume_output_notes(
    tx_request: TransactionRequest,
    client: &mut Client,
    executor: AccountId,
    consumer: AccountId,
) -> Result<(), ClientError> {
    let output_notes = tx_request
        .expected_output_own_notes()
        .into_iter()
        .map(|note| (note, None::<NoteArgs>))
        .collect::<Vec<(Note, Option<NoteArgs>)>>();

    execute_tx(client, executor, tx_request).await?;

    let tx_request = TransactionRequestBuilder::new()
        .unauthenticated_input_notes(output_notes)
        .build()?;
    let transaction_id = execute_tx(client, consumer, tx_request).await?;
    wait_for_tx(client, transaction_id).await
}
