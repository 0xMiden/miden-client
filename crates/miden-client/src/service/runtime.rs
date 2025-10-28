use std::sync::Arc;

use miden_client_core::account::AccountId;
use miden_client_core::sync::SyncSummary;
use miden_client_core::transaction::{
    TransactionAuthenticator, TransactionRequest, TransactionResult,
};
use miden_client_core::{Client, ClientError};
use tokio::select;
use tokio::sync::{Semaphore, mpsc, oneshot};
use tokio::time::{MissedTickBehavior, interval};
use tracing::error;

use crate::service::config::ClientServiceConfig;
use crate::service::transaction::{
    ProvenOutcome, SubmissionEntry, SubmissionMaterial, SubmissionState, spawn_prover,
};

pub(crate) enum Command {
    SyncNow {
        respond_to: oneshot::Sender<Result<SyncSummary, ClientError>>,
    },
    Transaction(TransactionCommand),
    Shutdown {
        respond_to: oneshot::Sender<()>,
    },
}

pub(crate) struct TransactionCommand {
    pub account_id: AccountId,
    pub request: TransactionRequest,
    pub execution: oneshot::Sender<Result<TransactionResult, ClientError>>,
    pub completion: oneshot::Sender<Result<(), ClientError>>,
}

pub(crate) async fn run_service<AUTH>(
    mut client: Client<AUTH>,
    config: ClientServiceConfig,
    mut command_rx: mpsc::Receiver<Command>,
    mut proven_rx: mpsc::Receiver<ProvenOutcome>,
    proof_limiter: Arc<Semaphore>,
    proven_tx: mpsc::Sender<ProvenOutcome>,
) -> Result<(), ClientError>
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    let mut submissions = SubmissionState::new();
    let mut ticker = config.sync_interval.map(|period| {
        let mut interval = interval(period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        interval
    });

    if config.initial_sync {
        if let Err(err) = client.sync_state().await {
            error!("initial sync failed: {err}");
        }
    }

    loop {
        select! {
            biased;
            command = command_rx.recv() => {
                match command {
                    Some(Command::SyncNow { respond_to }) => {
                        let result = client.sync_state().await;
                        let _ = respond_to.send(result);
                    }
                    Some(Command::Transaction(tx_command)) => {
                        handle_transaction_command(
                            &mut client,
                            tx_command,
                            &mut submissions,
                            &proven_tx,
                            Arc::clone(&proof_limiter),
                        ).await;
                        process_submission_queue(&mut client, &mut submissions).await;
                    }
                    Some(Command::Shutdown { respond_to }) => {
                        let _ = respond_to.send(());
                        break;
                    }
                    None => break,
                }
            }
            outcome = proven_rx.recv() => {
                if let Some(outcome) = outcome {
                    handle_proven_outcome(outcome, &mut submissions);
                    process_submission_queue(&mut client, &mut submissions).await;
                } else {
                    break;
                }
            }
            _ = async {
                if let Some(interval) = &mut ticker {
                    interval.tick().await;
                }
            }, if ticker.is_some() => {
                if let Err(err) = client.sync_state().await {
                    error!("background sync failed: {err}");
                }
            }
        }
    }

    Ok(())
}

async fn handle_transaction_command<AUTH>(
    client: &mut Client<AUTH>,
    command: TransactionCommand,
    submissions: &mut SubmissionState,
    proven_tx: &mpsc::Sender<ProvenOutcome>,
    proof_limiter: Arc<Semaphore>,
) where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    let seq = submissions.next_seq();
    match client.new_transaction(command.account_id, command.request).await {
        Ok(result) => {
            let _ = command.execution.send(Ok(result.clone()));
            spawn_prover(
                seq,
                result,
                client.transaction_prover(),
                proof_limiter,
                proven_tx.clone(),
                command.completion,
            );
        },
        Err(err) => {
            let message = err.to_string();
            let _ = command.execution.send(Err(err));
            let _ = command.completion.send(Err(ClientError::ClientInitializationError(message)));
            submissions.register_failure(seq);
        },
    }
}

fn handle_proven_outcome(outcome: ProvenOutcome, submissions: &mut SubmissionState) {
    match outcome {
        ProvenOutcome::Ready {
            seq,
            tx_result,
            proven_transaction,
            completion,
        } => {
            submissions.insert_ready(
                seq,
                SubmissionMaterial {
                    tx_result,
                    proven_transaction,
                    completion,
                },
            );
        },
        ProvenOutcome::Failed { seq } => {
            submissions.register_failure(seq);
        },
    }
}

async fn process_submission_queue<AUTH>(
    client: &mut Client<AUTH>,
    submissions: &mut SubmissionState,
) where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    loop {
        match submissions.pop_current() {
            Some(SubmissionEntry::Failed) => {
                submissions.advance();
            },
            Some(SubmissionEntry::Ready(material)) => {
                let SubmissionMaterial {
                    tx_result,
                    proven_transaction,
                    completion,
                } = material;
                match client.finalize_proven_transaction(proven_transaction, tx_result).await {
                    Ok(()) => {
                        let _ = completion.send(Ok(()));
                    },
                    Err(err) => {
                        let message = err.to_string();
                        let _ = completion.send(Err(err));
                        error!("failed to finalize proven transaction: {message}");
                    },
                }
                submissions.advance();
            },
            None => break,
        }
    }
}
