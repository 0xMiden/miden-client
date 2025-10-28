use std::collections::BTreeMap;
use std::sync::Arc;

use miden_client_core::transaction::{ProvenTransaction, TransactionInputs, TransactionResult};
use miden_client_core::{ClientError, LocalTransactionProver};
use tokio::sync::{Semaphore, mpsc, oneshot};
use tokio::task;

pub(crate) enum ProvenOutcome {
    Ready {
        seq: u64,
        tx_result: TransactionResult,
        proven_transaction: ProvenTransaction,
        completion: oneshot::Sender<Result<(), ClientError>>,
    },
    Failed {
        seq: u64,
    },
}

pub(crate) struct SubmissionMaterial {
    pub tx_result: TransactionResult,
    pub proven_transaction: ProvenTransaction,
    pub completion: oneshot::Sender<Result<(), ClientError>>,
}

pub(crate) enum SubmissionEntry {
    Ready(SubmissionMaterial),
    Failed,
}

pub(crate) struct SubmissionState {
    next_sequence: u64,
    next_submission: u64,
    pending: BTreeMap<u64, SubmissionEntry>,
}

impl SubmissionState {
    pub(crate) fn new() -> Self {
        Self {
            next_sequence: 0,
            next_submission: 0,
            pending: BTreeMap::new(),
        }
    }

    pub(crate) fn next_seq(&mut self) -> u64 {
        let seq = self.next_sequence;
        self.next_sequence += 1;
        seq
    }

    pub(crate) fn insert_ready(&mut self, seq: u64, material: SubmissionMaterial) {
        self.pending.insert(seq, SubmissionEntry::Ready(material));
    }

    pub(crate) fn register_failure(&mut self, seq: u64) {
        if seq == self.next_submission {
            self.next_submission += 1;
        } else {
            self.pending.insert(seq, SubmissionEntry::Failed);
        }
    }

    pub(crate) fn pop_current(&mut self) -> Option<SubmissionEntry> {
        self.pending.remove(&self.next_submission)
    }

    pub(crate) fn advance(&mut self) {
        self.next_submission += 1;
    }
}

pub(crate) fn spawn_prover(
    seq: u64,
    tx_result: TransactionResult,
    prover: Arc<LocalTransactionProver>,
    proof_limiter: Arc<Semaphore>,
    proven_tx: mpsc::Sender<ProvenOutcome>,
    completion: oneshot::Sender<Result<(), ClientError>>,
) {
    task::spawn_local(async move {
        let permit = proof_limiter.acquire_owned().await;
        if permit.is_err() {
            let _ = completion.send(Err(ClientError::ClientInitializationError(
                "transaction proof limiter closed".into(),
            )));
            return;
        }
        let _permit = permit.unwrap();

        let witness: TransactionInputs = tx_result.executed_transaction().clone().into();
        let mut completion = Some(completion);

        match LocalTransactionProver::prove(&prover, witness) {
            Ok(proven_transaction) => {
                let message = ProvenOutcome::Ready {
                    seq,
                    tx_result,
                    proven_transaction,
                    completion: completion.take().expect("completion sender available"),
                };

                if proven_tx.clone().send(message).await.is_err() {
                    if let Some(sender) = completion.take() {
                        let _ = sender.send(Err(ClientError::ClientInitializationError(
                            "client service stopped while dispatching proof".into(),
                        )));
                    }
                }
            },
            Err(err) => {
                if let Some(sender) = completion.take() {
                    let _ = sender.send(Err(ClientError::TransactionProvingError(err)));
                }
                let _ = proven_tx.clone().send(ProvenOutcome::Failed { seq }).await;
            },
        }
    });
}
