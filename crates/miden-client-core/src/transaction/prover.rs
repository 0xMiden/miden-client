use alloc::boxed::Box;

use miden_protocol::transaction::{ProvenTransaction, TransactionInputs};
use miden_remote_prover_client::remote_prover::tx_prover::RemoteTransactionProver;
use miden_tx::{LocalTransactionProver, TransactionProverError};

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait TransactionProver {
    async fn prove(
        &self,
        tx_result: TransactionInputs,
    ) -> Result<ProvenTransaction, TransactionProverError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl TransactionProver for LocalTransactionProver {
    async fn prove(
        &self,
        witness: TransactionInputs,
    ) -> Result<ProvenTransaction, TransactionProverError> {
        LocalTransactionProver::prove(self, witness)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl TransactionProver for RemoteTransactionProver {
    async fn prove(
        &self,
        witness: TransactionInputs,
    ) -> Result<ProvenTransaction, TransactionProverError> {
        let fut = RemoteTransactionProver::prove(self, &witness);
        fut.await
    }
}
