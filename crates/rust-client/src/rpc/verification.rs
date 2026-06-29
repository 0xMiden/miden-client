//! Response verification layer for [`NodeRpcClient`].
//!
//! A node's reply should match what the client asked for. [`VerifyingRpcClient`] wraps any
//! [`NodeRpcClient`] and checks each response against its request, returning
//! [`RpcError::InvalidResponse`] on a mismatch, so individual implementations only need to provide
//! transport. The client builder wraps every configured RPC client in this type, so these checks
//! always run regardless of the underlying implementation.

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::address::NetworkId;
use miden_protocol::batch::{ProposedBatch, ProvenBatch};
use miden_protocol::block::{BlockHeader, BlockNumber, ProvenBlock};
use miden_protocol::crypto::merkle::mmr::MmrProof;
use miden_protocol::note::{NoteId, NoteScript, NoteTag};
use miden_protocol::transaction::{ProvenTransaction, TransactionInputs};

use super::domain::account::{AccountProof, GetAccountRequest};
use super::domain::account_vault::AccountVaultInfo;
use super::domain::note::{CommittedNote, FetchedNote, NoteSyncBlock};
use super::domain::nullifier::NullifierUpdate;
use super::domain::storage_map::StorageMapInfo;
use super::domain::sync::{ChainMmrInfo, SyncTarget};
use super::domain::transaction::TransactionRecord;
use super::{
    AccountStateAt,
    NetworkNoteStatusInfo,
    NodeRpcClient,
    RpcError,
    RpcLimits,
    RpcStatusInfo,
};

// VERIFYING RPC CLIENT
// ================================================================================================

/// A [`NodeRpcClient`] decorator that verifies each response against its request, returning
/// [`RpcError::InvalidResponse`] on a mismatch. Each overridden method documents the specific
/// invariant it checks.
///
/// Checks that need cross-call or client-side state are not performed here; they live in the sync
/// layer.
pub struct VerifyingRpcClient {
    inner: Arc<dyn NodeRpcClient>,
}

impl VerifyingRpcClient {
    /// Wraps `inner` so that its responses are verified against their requests.
    pub fn new(inner: Arc<dyn NodeRpcClient>) -> Self {
        Self { inner }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl NodeRpcClient for VerifyingRpcClient {
    async fn set_genesis_commitment(&self, commitment: Word) -> Result<(), RpcError> {
        self.inner.set_genesis_commitment(commitment).await
    }

    fn has_genesis_commitment(&self) -> Option<Word> {
        self.inner.has_genesis_commitment()
    }

    async fn submit_proven_transaction(
        &self,
        proven_transaction: ProvenTransaction,
        transaction_inputs: TransactionInputs,
    ) -> Result<BlockNumber, RpcError> {
        self.inner
            .submit_proven_transaction(proven_transaction, transaction_inputs)
            .await
    }

    async fn submit_proven_batch(
        &self,
        proven_batch: ProvenBatch,
        proposed_batch: ProposedBatch,
        transaction_inputs: Vec<TransactionInputs>,
    ) -> Result<BlockNumber, RpcError> {
        self.inner
            .submit_proven_batch(proven_batch, proposed_batch, transaction_inputs)
            .await
    }

    /// Verifies the returned header is for the requested block, when a block number is given.
    async fn get_block_header_by_number(
        &self,
        block_num: Option<BlockNumber>,
        include_mmr_proof: bool,
    ) -> Result<(BlockHeader, Option<MmrProof>), RpcError> {
        let (header, mmr_proof) =
            self.inner.get_block_header_by_number(block_num, include_mmr_proof).await?;
        verify_block_num(block_num, header.block_num())?;
        Ok((header, mmr_proof))
    }

    /// Verifies the returned block is for the requested number.
    async fn get_block_by_number(
        &self,
        block_num: BlockNumber,
        include_proof: bool,
    ) -> Result<ProvenBlock, RpcError> {
        let block = self.inner.get_block_by_number(block_num, include_proof).await?;
        verify_block_num(Some(block_num), block.header().block_num())?;
        Ok(block)
    }

    /// Verifies every returned note's ID was requested.
    async fn get_notes_by_id(&self, note_ids: &[NoteId]) -> Result<Vec<FetchedNote>, RpcError> {
        let notes = self.inner.get_notes_by_id(note_ids).await?;
        let requested: BTreeSet<NoteId> = note_ids.iter().copied().collect();
        verify_note_ids(&requested, notes.iter().map(FetchedNote::id))?;
        Ok(notes)
    }

    async fn sync_chain_mmr(
        &self,
        current_block_height: BlockNumber,
        upper_bound: SyncTarget,
    ) -> Result<ChainMmrInfo, RpcError> {
        self.inner.sync_chain_mmr(current_block_height, upper_bound).await
    }

    /// Verifies every returned note's tag was requested.
    async fn sync_notes(
        &self,
        block_from: BlockNumber,
        block_to: BlockNumber,
        note_tags: &BTreeSet<NoteTag>,
    ) -> Result<Vec<NoteSyncBlock>, RpcError> {
        let blocks = self.inner.sync_notes(block_from, block_to, note_tags).await?;
        verify_note_tags(
            note_tags,
            blocks.iter().flat_map(|b| b.notes.values().map(CommittedNote::tag)),
        )?;
        Ok(blocks)
    }

    /// Verifies every returned nullifier's prefix was requested.
    async fn sync_nullifiers(
        &self,
        prefix: &[u16],
        block_from: BlockNumber,
        block_to: BlockNumber,
    ) -> Result<Vec<NullifierUpdate>, RpcError> {
        let nullifiers = self.inner.sync_nullifiers(prefix, block_from, block_to).await?;
        let requested: BTreeSet<u16> = prefix.iter().copied().collect();
        verify_nullifier_prefixes(&requested, &nullifiers)?;
        Ok(nullifiers)
    }

    /// Verifies the response is for the requested block, when a specific block is requested.
    async fn get_account(
        &self,
        account_id: AccountId,
        request: GetAccountRequest,
    ) -> Result<(BlockNumber, AccountProof), RpcError> {
        let requested = match request.at {
            AccountStateAt::Block(number) => Some(number),
            AccountStateAt::ChainTip => None,
        };
        let (block_num, proof) = self.inner.get_account(account_id, request).await?;
        verify_block_num(requested, block_num)?;
        Ok((block_num, proof))
    }

    /// Verifies the returned script's root matches the requested one.
    async fn get_note_script_by_root(&self, root: Word) -> Result<Option<NoteScript>, RpcError> {
        let script = self.inner.get_note_script_by_root(root).await?;
        if let Some(script) = &script {
            verify_note_script_root(root, script)?;
        }
        Ok(script)
    }

    async fn sync_storage_maps(
        &self,
        block_from: BlockNumber,
        block_to: BlockNumber,
        account_id: AccountId,
    ) -> Result<StorageMapInfo, RpcError> {
        self.inner.sync_storage_maps(block_from, block_to, account_id).await
    }

    async fn sync_account_vault(
        &self,
        block_from: BlockNumber,
        block_to: BlockNumber,
        account_id: AccountId,
    ) -> Result<AccountVaultInfo, RpcError> {
        self.inner.sync_account_vault(block_from, block_to, account_id).await
    }

    async fn sync_transactions(
        &self,
        block_from: BlockNumber,
        block_to: BlockNumber,
        account_ids: Vec<AccountId>,
    ) -> Result<Vec<TransactionRecord>, RpcError> {
        self.inner.sync_transactions(block_from, block_to, account_ids).await
    }

    async fn get_network_id(&self) -> Result<NetworkId, RpcError> {
        self.inner.get_network_id().await
    }

    async fn get_rpc_limits(&self) -> Result<RpcLimits, RpcError> {
        self.inner.get_rpc_limits().await
    }

    fn has_rpc_limits(&self) -> Option<RpcLimits> {
        self.inner.has_rpc_limits()
    }

    async fn set_rpc_limits(&self, limits: RpcLimits) {
        self.inner.set_rpc_limits(limits).await;
    }

    async fn get_status_unversioned(&self) -> Result<RpcStatusInfo, RpcError> {
        self.inner.get_status_unversioned().await
    }

    async fn get_network_note_status(
        &self,
        note_id: NoteId,
    ) -> Result<NetworkNoteStatusInfo, RpcError> {
        self.inner.get_network_note_status(note_id).await
    }
}

// RESPONSE CHECKS
// ================================================================================================

/// Returns [`RpcError::InvalidResponse`] if `requested` is `Some` and `returned` does not equal it.
fn verify_block_num(requested: Option<BlockNumber>, returned: BlockNumber) -> Result<(), RpcError> {
    if let Some(requested) = requested
        && returned != requested
    {
        return Err(RpcError::InvalidResponse(format!(
            "node returned block {returned} but block {requested} was requested"
        )));
    }
    Ok(())
}

/// Returns [`RpcError::InvalidResponse`] if any returned note ID was not in `requested`.
fn verify_note_ids(
    requested: &BTreeSet<NoteId>,
    returned: impl IntoIterator<Item = NoteId>,
) -> Result<(), RpcError> {
    for id in returned {
        if !requested.contains(&id) {
            let list = requested.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
            return Err(RpcError::InvalidResponse(format!(
                "node returned note {id} but [{list}] were requested"
            )));
        }
    }
    Ok(())
}

/// Returns [`RpcError::InvalidResponse`] if any returned note tag was not in `requested`.
fn verify_note_tags(
    requested: &BTreeSet<NoteTag>,
    returned: impl IntoIterator<Item = NoteTag>,
) -> Result<(), RpcError> {
    for tag in returned {
        if !requested.contains(&tag) {
            let list = requested.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
            return Err(RpcError::InvalidResponse(format!(
                "node returned note with tag {tag} but [{list}] were requested"
            )));
        }
    }
    Ok(())
}

/// Returns [`RpcError::InvalidResponse`] if any update carries a nullifier whose prefix was not in
/// `requested_prefixes`.
fn verify_nullifier_prefixes(
    requested_prefixes: &BTreeSet<u16>,
    batch: &[NullifierUpdate],
) -> Result<(), RpcError> {
    for update in batch {
        let prefix = update.nullifier.prefix();
        if !requested_prefixes.contains(&prefix) {
            let requested = requested_prefixes
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(RpcError::InvalidResponse(format!(
                "node returned nullifier with prefix {prefix} but [{requested}] were requested"
            )));
        }
    }
    Ok(())
}

/// Returns [`RpcError::InvalidResponse`] if `script`'s root does not equal the `requested` root.
fn verify_note_script_root(requested: Word, script: &NoteScript) -> Result<(), RpcError> {
    let fetched_root = script.root();
    if Word::from(fetched_root) != requested {
        return Err(RpcError::InvalidResponse(format!(
            "node returned note script with root {fetched_root} for requested root {requested}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use core::slice;
    use std::collections::BTreeSet;

    use miden_protocol::note::{NoteId, NoteTag, Nullifier};
    use miden_protocol::{Felt, Word};

    use super::{NullifierUpdate, verify_note_ids, verify_note_tags, verify_nullifier_prefixes};
    use crate::rpc::RpcError;

    fn nullifier_with_prefix(prefix: u16) -> Nullifier {
        Nullifier::from_raw(Word::new([
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::new_unchecked(u64::from(prefix) << 48),
        ]))
    }

    #[test]
    fn verify_nullifier_prefixes_rejects_unrequested() {
        let requested = NullifierUpdate {
            nullifier: nullifier_with_prefix(0x1234),
            block_num: 1u32.into(),
        };
        let unrequested = NullifierUpdate {
            nullifier: nullifier_with_prefix(0xabcd),
            block_num: 2u32.into(),
        };

        let requested_prefixes: BTreeSet<u16> = BTreeSet::from([0x1234]);

        verify_nullifier_prefixes(&requested_prefixes, slice::from_ref(&requested))
            .expect("requested prefix must be accepted");

        let err = verify_nullifier_prefixes(&requested_prefixes, &[requested, unrequested])
            .expect_err("unrequested prefix must be rejected");
        assert!(matches!(err, RpcError::InvalidResponse(_)));
    }

    #[test]
    fn verify_note_tags_rejects_unrequested() {
        let requested = NoteTag::new(1);
        let other = NoteTag::new(2);
        let requested_set = BTreeSet::from([requested]);

        verify_note_tags(&requested_set, [requested]).expect("requested tag must be accepted");

        let err = verify_note_tags(&requested_set, [other])
            .expect_err("unrequested tag must be rejected");
        assert!(matches!(err, RpcError::InvalidResponse(_)));
    }

    fn note_id(n: u32) -> NoteId {
        NoteId::from_raw(Word::from([n, 0, 0, 0]))
    }

    #[test]
    fn verify_note_ids_rejects_unrequested() {
        let requested = note_id(1);
        let other = note_id(2);
        let requested_set = BTreeSet::from([requested]);

        verify_note_ids(&requested_set, [requested]).expect("requested note id must be accepted");

        let err = verify_note_ids(&requested_set, [other])
            .expect_err("unrequested note id must be rejected");
        assert!(matches!(err, RpcError::InvalidResponse(_)));
    }
}
