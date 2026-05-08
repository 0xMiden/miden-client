use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

use miden_protocol::Word;
use miden_protocol::account::{AccountCode, AccountId};
use miden_protocol::address::NetworkId;
use miden_protocol::batch::{ProposedBatch, ProvenBatch};
use miden_protocol::block::{BlockHeader, BlockNumber, ProvenBlock};
use miden_protocol::crypto::merkle::MerklePath;
use miden_protocol::crypto::merkle::mmr::MmrProof;
use miden_protocol::note::{NoteId, NoteScript, NoteTag};
use miden_protocol::transaction::{ProvenTransaction, TransactionInputs};

use super::domain::account::{AccountProof, FetchedAccount};
use super::domain::account_vault::AccountVaultInfo;
use super::domain::note::{NoteSyncBlock, NoteSyncInfo};
use super::domain::nullifier::NullifierUpdate;
use super::domain::status::{NetworkNoteStatusInfo, RpcStatusInfo};
use super::domain::storage_map::StorageMapInfo;
use super::domain::sync::{ChainMmrInfo, SyncTarget};
use super::domain::transaction::TransactionsInfo;
use super::{AccountStateAt, FetchedNote, NodeRpcClient, RpcError, RpcLimits};

#[derive(Default)]
struct PaginatedNotesRpc {
    calls: AtomicUsize,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl NodeRpcClient for PaginatedNotesRpc {
    async fn set_genesis_commitment(&self, _commitment: Word) -> Result<(), RpcError> {
        Ok(())
    }

    fn has_genesis_commitment(&self) -> Option<Word> {
        None
    }

    async fn submit_proven_transaction(
        &self,
        _proven_transaction: ProvenTransaction,
        _transaction_inputs: TransactionInputs,
    ) -> Result<BlockNumber, RpcError> {
        unimplemented!()
    }

    async fn submit_proven_batch(
        &self,
        _proven_batch: ProvenBatch,
        _proposed_batch: ProposedBatch,
        _transaction_inputs: Vec<TransactionInputs>,
    ) -> Result<BlockNumber, RpcError> {
        unimplemented!()
    }

    async fn get_block_header_by_number(
        &self,
        _block_num: Option<BlockNumber>,
        _include_mmr_proof: bool,
    ) -> Result<(BlockHeader, Option<MmrProof>), RpcError> {
        unimplemented!()
    }

    async fn get_block_by_number(
        &self,
        _block_num: BlockNumber,
        _include_proof: bool,
    ) -> Result<ProvenBlock, RpcError> {
        unimplemented!()
    }

    async fn get_notes_by_id(&self, _note_ids: &[NoteId]) -> Result<Vec<FetchedNote>, RpcError> {
        Ok(vec![])
    }

    async fn sync_chain_mmr(
        &self,
        _block_from: BlockNumber,
        _upper_bound: SyncTarget,
    ) -> Result<ChainMmrInfo, RpcError> {
        unimplemented!()
    }

    async fn get_account_details(
        &self,
        _account_id: AccountId,
    ) -> Result<FetchedAccount, RpcError> {
        unimplemented!()
    }

    async fn sync_notes(
        &self,
        block_num: BlockNumber,
        block_to: Option<BlockNumber>,
        _note_tags: &BTreeSet<NoteTag>,
    ) -> Result<NoteSyncInfo, RpcError> {
        assert_eq!(block_to, Some(BlockNumber::from(10)));
        let call = self.calls.fetch_add(1, Ordering::SeqCst);

        match call {
            0 => {
                assert_eq!(block_num, BlockNumber::from(9));
                Ok(NoteSyncInfo {
                    chain_tip: 10_u32.into(),
                    block_to: 9_u32.into(),
                    blocks: vec![note_sync_block(9_u32.into())],
                })
            },
            1 => {
                assert_eq!(block_num, BlockNumber::from(10));
                Ok(NoteSyncInfo {
                    chain_tip: 10_u32.into(),
                    block_to: 10_u32.into(),
                    blocks: vec![note_sync_block(10_u32.into())],
                })
            },
            _ => panic!("sync_notes called too many times"),
        }
    }

    async fn sync_nullifiers(
        &self,
        _prefix: &[u16],
        _block_num: BlockNumber,
        _block_to: Option<BlockNumber>,
    ) -> Result<Vec<NullifierUpdate>, RpcError> {
        unimplemented!()
    }

    async fn get_account_proof(
        &self,
        _account_id: AccountId,
        _storage_requirements: super::domain::account::AccountStorageRequirements,
        _account_state: AccountStateAt,
        _known_account_code: Option<AccountCode>,
        _known_vault_commitment: Option<Word>,
    ) -> Result<(BlockNumber, AccountProof), RpcError> {
        unimplemented!()
    }

    async fn get_note_script_by_root(&self, _root: Word) -> Result<NoteScript, RpcError> {
        unimplemented!()
    }

    async fn sync_storage_maps(
        &self,
        _block_from: BlockNumber,
        _block_to: Option<BlockNumber>,
        _account_id: AccountId,
    ) -> Result<StorageMapInfo, RpcError> {
        unimplemented!()
    }

    async fn sync_account_vault(
        &self,
        _block_from: BlockNumber,
        _block_to: Option<BlockNumber>,
        _account_id: AccountId,
    ) -> Result<AccountVaultInfo, RpcError> {
        unimplemented!()
    }

    async fn sync_transactions(
        &self,
        _block_from: BlockNumber,
        _block_to: Option<BlockNumber>,
        _account_ids: Vec<AccountId>,
    ) -> Result<TransactionsInfo, RpcError> {
        unimplemented!()
    }

    async fn get_network_id(&self) -> Result<NetworkId, RpcError> {
        unimplemented!()
    }

    async fn get_rpc_limits(&self) -> Result<RpcLimits, RpcError> {
        unimplemented!()
    }

    fn has_rpc_limits(&self) -> Option<RpcLimits> {
        None
    }

    async fn set_rpc_limits(&self, _limits: RpcLimits) {}

    async fn get_status_unversioned(&self) -> Result<RpcStatusInfo, RpcError> {
        unimplemented!()
    }

    async fn get_network_note_status(
        &self,
        _note_id: NoteId,
    ) -> Result<NetworkNoteStatusInfo, RpcError> {
        unimplemented!()
    }
}

#[tokio::test]
async fn sync_notes_with_details_fetches_inclusive_upper_bound_page() {
    let rpc = PaginatedNotesRpc::default();

    let result = rpc
        .sync_notes_with_details(9_u32.into(), Some(10_u32.into()), &BTreeSet::new())
        .await
        .expect("sync notes should succeed");

    assert_eq!(result.blocks.len(), 2);
    assert_eq!(rpc.calls.load(Ordering::SeqCst), 2);
}

fn note_sync_block(block_num: BlockNumber) -> NoteSyncBlock {
    NoteSyncBlock {
        block_header: BlockHeader::mock(block_num, None, None, &[], Word::default()),
        mmr_path: MerklePath::new(vec![]),
        notes: BTreeMap::new(),
    }
}
