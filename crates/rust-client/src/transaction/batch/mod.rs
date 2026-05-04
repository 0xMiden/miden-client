//! Stacks multiple transactions against a single local account and submits them as one
//! proven batch via the node's `SubmitProvenBatch` endpoint.
//!
//! ## Flow
//!
//! 1. Open a builder with [`Client::new_transaction_batch`](crate::Client::new_transaction_batch)
//!    for a tracked local account.
//! 2. Add transactions via [`BatchBuilder::push`]. Each push executes the request against the
//!    batch's in-memory account state (so later pushes see the post-state of earlier ones), proves
//!    it locally, and appends the proven transaction to the batch.
//! 3. Finalize with [`BatchBuilder::submit`]. This assembles a `ProposedBatch`, proves it, submits
//!    it to the node, and atomically applies the per-transaction updates to the local store.
//!    Returns the [`BlockNumber`] the batch was accepted into.
//!
//! ## Constraints
//!
//! - All transactions in a batch belong to the same local account (fixed at construction).
//! - No two transactions in a batch may consume the same input note (rejected with
//!   [`BatchBuilderError::DuplicateInputNote`]).
//! - At least one successful [`push`](BatchBuilder::push) is required before
//!   [`submit`](BatchBuilder::submit) (otherwise [`BatchBuilderError::Empty`]).
//!
//! ## Error semantics after RPC accept
//!
//! Once the node accepts the batch, the local store still needs to be updated. If that step
//! fails, the caller receives one of two errors that both carry the accepted `block_num`:
//!
//! - [`BatchBuilderError::BatchSubmittedButUpdateBuildFailed`] — building one of the per-tx
//!   [`TransactionStoreUpdate`]s failed.
//! - [`BatchBuilderError::BatchSubmittedButApplyFailed`] — applying the updates atomically to the
//!   local store failed.
//!
//! In both cases the recovery path is to trigger `sync_state` to reconcile.

mod data_store;
mod error;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

pub(crate) use data_store::InMemoryBatchDataStore;
pub use error::BatchBuilderError;
use miden_protocol::MIN_PROOF_SECURITY_LEVEL;
use miden_protocol::account::AccountId;
use miden_protocol::batch::ProposedBatch;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::note::Nullifier;
use miden_protocol::transaction::{
    PartialBlockchain,
    ProvenTransaction,
    ToInputNoteCommitments,
    TransactionInputs,
};
use miden_tx::auth::TransactionAuthenticator;
use miden_tx_batch_prover::LocalBatchProver;

use crate::store::data_store::build_partial_mmr_with_paths;
use crate::transaction::{TransactionRequest, TransactionResult, TransactionStoreUpdate};
use crate::{Client, ClientError};

/// Accumulates transactions for a single local account and submits them as one
/// proven batch via the node's `SubmitProvenBatch` endpoint.
///
/// See the module-level docs for usage.
pub struct BatchBuilder<'c, AUTH> {
    pub(crate) client: &'c Client<AUTH>,
    pub(crate) account_id: AccountId,
    pub(crate) data_store: InMemoryBatchDataStore,
    pub(crate) proven_txs: Vec<Arc<ProvenTransaction>>,
    pub(crate) transaction_inputs: Vec<TransactionInputs>,
    pub(crate) tx_results: Vec<TransactionResult>,
    pub(crate) consumed_nullifiers: BTreeSet<Nullifier>,
}

impl<AUTH> BatchBuilder<'_, AUTH> {
    /// Number of successfully-pushed transactions in this batch.
    pub fn len(&self) -> usize {
        self.proven_txs.len()
    }

    /// True if no transaction has been pushed yet.
    pub fn is_empty(&self) -> bool {
        self.proven_txs.is_empty()
    }

    /// The account id this batch is locked to.
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }
}

impl<AUTH> BatchBuilder<'_, AUTH>
where
    AUTH: TransactionAuthenticator + Sync + 'static,
{
    /// Assemble the `ProposedBatch`, prove it, submit
    /// it via the client's RPC, and atomically apply the per-transaction
    /// updates to the local store. Returns the block number the batch was
    /// accepted into.
    pub async fn submit(self) -> Result<BlockNumber, ClientError> {
        if self.proven_txs.is_empty() {
            return Err(ClientError::from(BatchBuilderError::Empty));
        }

        // 1. Treat the largest ref as the reference block and the rest as authenticated.
        let ref_block_num = self
            .proven_txs
            .iter()
            .map(|tx| tx.ref_block_num())
            .max()
            .expect("non-empty — proven_txs.is_empty() was checked above");

        let lower_refs: BTreeSet<BlockNumber> = self
            .proven_txs
            .iter()
            .map(|tx| tx.ref_block_num())
            .filter(|&r| r < ref_block_num)
            .collect();

        // 2. Fetch the reference block header (from the store).
        let (ref_block_header, _) = self
            .client
            .store
            .get_block_header_by_num(ref_block_num)
            .await
            .map_err(ClientError::StoreError)?
            .ok_or_else(|| {
                ClientError::StoreError(crate::store::StoreError::BlockHeaderNotFound(
                    ref_block_num,
                ))
            })?;

        // 3. Fetch block headers for each lower ref (the ones needing authentication).
        let fetched = self
            .client
            .store
            .get_block_headers(&lower_refs)
            .await
            .map_err(ClientError::StoreError)?;
        let mut authenticated_blocks: Vec<BlockHeader> = Vec::with_capacity(fetched.len());
        for (header, _relevance) in fetched {
            authenticated_blocks.push(header);
        }

        // 4. Build PartialMmr + PartialBlockchain using `ref_block_num` as the forest — this
        //    matches the MMR convention used by `ClientDataStore::get_transaction_inputs`.
        let forest = ref_block_num.as_u32();
        let partial_mmr =
            build_partial_mmr_with_paths(&self.client.store, forest, &authenticated_blocks).await?;
        let partial_blockchain = PartialBlockchain::new(partial_mmr, authenticated_blocks)?;

        // 5. Build ProposedBatch.
        let proposed_batch = ProposedBatch::new(
            self.proven_txs.clone(),
            ref_block_header,
            partial_blockchain,
            BTreeMap::new(),
        )?;

        // 6. Prove synchronously.
        let proven_batch =
            LocalBatchProver::new(MIN_PROOF_SECURITY_LEVEL).prove(proposed_batch.clone())?;

        // 7. Submit via RPC.
        let mut updates: Vec<TransactionStoreUpdate> = Vec::with_capacity(self.len());
        let block_num = self
            .client
            .rpc_api
            .submit_proven_batch(proven_batch, proposed_batch, self.transaction_inputs)
            .await?;

        // 8. Build per-tx TransactionStoreUpdates.
        for tx_result in &self.tx_results {
            let update =
                self.client.get_transaction_store_update(tx_result, block_num).await.map_err(
                    |source| BatchBuilderError::BatchSubmittedButUpdateBuildFailed {
                        block_num,
                        source,
                    },
                )?;
            updates.push(update);
        }

        // 9. Apply atomically; if it fails, return BatchSubmittedButApplyFailed.
        if let Err(source) = self.client.store.apply_transaction_batch(updates).await {
            return Err(ClientError::from(BatchBuilderError::BatchSubmittedButApplyFailed {
                block_num,
                source,
            }));
        }

        Ok(block_num)
    }

    /// Execute requested `tx` against the batch's stacked in-memory state, prove it using
    /// the client's configured [`crate::transaction::TransactionProver`], and append the
    /// resulting proven transaction to the batch.
    pub async fn push(mut self, req: TransactionRequest) -> Result<Self, ClientError> {
        // Execute against the batch data store (uses the in-memory account state).
        let tx_result = self
            .client
            .execute_transaction_for_batch(&self.data_store, self.account_id, req)
            .await?;

        // Extract TransactionInputs for later batch submission.
        let tx_inputs = tx_result.executed_transaction().tx_inputs().clone();

        // Check for duplicate input notes against earlier pushes.
        for note in tx_result.consumed_notes().iter() {
            if self.consumed_nullifiers.contains(&note.nullifier()) {
                return Err(ClientError::from(BatchBuilderError::DuplicateInputNote(note.id())));
            }
        }

        // Prove using the client's default tx prover.
        let proven_tx =
            self.client.prove_transaction_with(&tx_result, self.client.prover()).await?;

        // Update the in-batch account state by applying the transaction delta.
        let mut post_account = self.data_store.current_account().clone();
        post_account
            .apply_delta(tx_result.executed_transaction().account_delta())
            .map_err(ClientError::AccountError)?;
        self.data_store.set_current_account(post_account);

        // Record consumed nullifiers.
        for note in tx_result.consumed_notes().iter() {
            self.consumed_nullifiers.insert(note.nullifier());
        }

        // Append proven tx, inputs, result.
        self.proven_txs.push(Arc::new(proven_tx));
        self.transaction_inputs.push(tx_inputs);
        self.tx_results.push(tx_result);

        Ok(self)
    }
}
