use alloc::{collections::BTreeSet, sync::Arc, vec::Vec};

use miden_lib::transaction::TransactionKernel;
use miden_objects::{
    Felt, Word,
    account::{AccountCode, AccountDelta, AccountId},
    block::{BlockHeader, BlockNumber, ProvenBlock},
    crypto::{
        merkle::{Forest, MerklePath, Mmr, MmrProof, SmtProof},
        rand::RpoRandomCoin,
    },
    note::{NoteExecutionMode, NoteId, NoteTag, NoteType, Nullifier},
    testing::note::NoteBuilder,
    transaction::{OutputNote, ProvenTransaction},
};
use miden_testing::{MockChain, MockChainBuilder, MockChainNote};
use miden_tx::utils::sync::RwLock;

use crate::{
    Client,
    rpc::{
        NodeRpcClient, RpcError,
        domain::{
            account::{AccountProofs, FetchedAccount},
            note::{CommittedNote, FetchedNote, NoteSyncInfo},
            nullifier::NullifierUpdate,
            sync::StateSyncInfo,
        },
        generated::{
            note::NoteSyncRecord, responses::SyncStateResponse, transaction::TransactionSummary,
        },
    },
    transaction::ForeignAccount,
};

pub type MockClient = Client;

/// Mock RPC API
///
/// This struct implements the RPC API used by the client to communicate with the node. It is
/// intended to be used for testing purposes only.
#[derive(Clone)]
pub struct MockRpcApi {
    committed_transactions: Arc<RwLock<Vec<TransactionSummary>>>, /* TODO: Should this be tracked by the mock_chain? */
    pub mock_chain: Arc<RwLock<MockChain>>,
}

impl MockRpcApi {
    /// Creates a new `MockRpcApi` instance with pre-populated blocks and notes.
    pub async fn new() -> Self {
        let mut mock_chain_builder = MockChainBuilder::new();
        let mock_account = mock_chain_builder
            .add_existing_mock_account(miden_testing::Auth::IncrNonce)
            .unwrap();

        let note_first = NoteBuilder::new(
            mock_account.id(),
            RpoRandomCoin::new([0, 0, 0, 0].map(Felt::new).into()),
        )
        .tag(NoteTag::for_public_use_case(0, 0, NoteExecutionMode::Local).unwrap().into())
        .build(&TransactionKernel::assembler())
        .unwrap();

        let note_second = NoteBuilder::new(
            mock_account.id(),
            RpoRandomCoin::new([0, 0, 0, 1].map(Felt::new).into()),
        )
        .note_type(NoteType::Private)
        .tag(NoteTag::for_local_use_case(0, 0).unwrap().into())
        .build(&TransactionKernel::assembler())
        .unwrap();
        let mut mock_chain = mock_chain_builder.build().unwrap();

        // Block 1: Create first note
        mock_chain.add_pending_note(OutputNote::Full(note_first));
        mock_chain.prove_next_block().unwrap();

        // Block 2
        mock_chain.prove_next_block().unwrap();

        // Block 3
        mock_chain.prove_next_block().unwrap();

        // Block 4: Create second note
        mock_chain.add_pending_note(OutputNote::Full(note_second.clone()));
        mock_chain.prove_next_block().unwrap();

        let transaction = mock_chain
            .build_tx_context(mock_account, &[note_second.id()], &[])
            .unwrap()
            .build()
            .unwrap()
            .execute()
            .await
            .unwrap();

        // Block 5: Consume (nullify) second note
        mock_chain.add_pending_executed_transaction(&transaction).unwrap();
        mock_chain.prove_next_block().unwrap();

        Self {
            committed_transactions: Arc::new(RwLock::new(vec![])),
            mock_chain: Arc::new(RwLock::new(mock_chain)),
        }
    }

    /// Returns the current MMR of the blockchain.
    pub fn get_mmr(&self) -> Mmr {
        self.mock_chain.read().blockchain().as_mmr().clone()
    }

    /// Returns the chain tip block number.
    pub fn get_chain_tip_block_num(&self) -> BlockNumber {
        self.mock_chain.read().latest_block_header().block_num()
    }

    /// Retrieves a block by its block number.
    fn get_block_by_num(&self, block_num: BlockNumber) -> BlockHeader {
        self.mock_chain.read().block_header(block_num.as_usize())
    }

    /// Generates a sync state response based on the request block number.
    fn get_sync_state_request(
        &self,
        request_block_num: BlockNumber,
        note_tags: &BTreeSet<NoteTag>,
    ) -> Result<SyncStateResponse, RpcError> {
        // Determine the next block number to sync
        let next_block_num = self
            .mock_chain
            .read()
            .committed_notes()
            .values()
            .filter_map(|note| {
                let block_num = note.inclusion_proof().location().block_num();
                if note_tags.contains(&note.metadata().tag()) && block_num > request_block_num {
                    Some(block_num)
                } else {
                    None
                }
            })
            .min()
            .unwrap_or_else(|| self.get_chain_tip_block_num());

        // Retrieve the next block
        let next_block = self.get_block_by_num(next_block_num);

        // Prepare the MMR delta
        let from_block_num = if request_block_num == self.get_chain_tip_block_num() {
            next_block_num.as_usize()
        } else {
            request_block_num.as_usize() + 1
        };

        let mmr_delta = self
            .get_mmr()
            .get_delta(Forest::new(from_block_num), Forest::new(next_block_num.as_usize()))
            .unwrap();

        // Collect notes that are in the next block
        let notes = self.get_notes_in_block(next_block_num, note_tags);

        let transactions = self
            .committed_transactions
            .read()
            .iter()
            .filter(|tx| tx.block_num == next_block_num.as_u32())
            .cloned()
            .collect::<Vec<_>>();

        Ok(SyncStateResponse {
            chain_tip: self.get_chain_tip_block_num().as_u32(),
            block_header: Some(next_block.into()),
            mmr_delta: Some(mmr_delta.try_into()?),
            accounts: vec![],
            transactions,
            notes,
        })
    }

    /// Retrieves notes that are included in the specified block number.
    fn get_notes_in_block(
        &self,
        block_num: BlockNumber,
        note_tags: &BTreeSet<NoteTag>,
    ) -> Vec<NoteSyncRecord> {
        self.mock_chain
            .read()
            .committed_notes()
            .values()
            .filter_map(move |note| {
                if note.inclusion_proof().location().block_num() == block_num
                    && note_tags.contains(&note.metadata().tag())
                {
                    Some(NoteSyncRecord {
                        note_index: u32::from(
                            note.inclusion_proof().location().node_index_in_block(),
                        ),
                        note_id: Some(note.id().into()),
                        metadata: Some((*note.metadata()).into()),
                        merkle_path: Some(
                            MerklePath::from(note.inclusion_proof().note_path().clone()).into(),
                        ),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_available_notes(&self) -> Vec<MockChainNote> {
        self.mock_chain.read().committed_notes().values().cloned().collect()
    }

    pub fn advance_blocks(&self, num_blocks: u32) {
        let current_height = self.get_chain_tip_block_num();
        let mut mock_chain = self.mock_chain.write();
        mock_chain.prove_until_block(current_height + num_blocks).unwrap();
    }
}
use alloc::boxed::Box;
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl NodeRpcClient for MockRpcApi {
    async fn sync_notes(
        &self,
        block_num: BlockNumber,
        note_tags: &BTreeSet<NoteTag>,
    ) -> Result<NoteSyncInfo, RpcError> {
        let response = self.get_sync_state_request(block_num, note_tags)?;

        let response = NoteSyncInfo {
            chain_tip: response.chain_tip,
            block_header: response.block_header.unwrap().try_into().unwrap(),
            mmr_path: MerklePath::default(),
            notes: response
                .notes
                .into_iter()
                .map(|note| {
                    let digest: Word = note.note_id.unwrap().try_into().unwrap();
                    let note_id: NoteId = NoteId::from(digest);
                    let note_index = u16::try_from(note.note_index).unwrap();
                    let merkle_path = note.merkle_path.unwrap().try_into().unwrap();
                    let metadata = note.metadata.unwrap().try_into().unwrap();

                    CommittedNote::new(note_id, note_index, merkle_path, metadata)
                })
                .collect(),
        };

        Ok(response)
    }

    /// Executes the specified sync state request and returns the response.
    async fn sync_state(
        &self,
        block_num: BlockNumber,
        _account_ids: &[AccountId],
        note_tags: &BTreeSet<NoteTag>,
    ) -> Result<StateSyncInfo, RpcError> {
        let response = self.get_sync_state_request(block_num, note_tags)?;

        Ok(response.try_into().unwrap())
    }

    /// Creates and executes a `GetBlockHeaderByNumberRequest`. Will retrieve the block header
    /// for the specified block number. If the block number is not provided, the chain tip block
    /// header will be returned.
    async fn get_block_header_by_number(
        &self,
        block_num: Option<BlockNumber>,
        include_mmr_proof: bool,
    ) -> Result<(BlockHeader, Option<MmrProof>), RpcError> {
        let block = if let Some(block_num) = block_num {
            self.mock_chain.read().block_header(block_num.as_usize())
        } else {
            self.mock_chain.read().latest_block_header()
        };

        let mmr_proof = if include_mmr_proof {
            Some(self.get_mmr().open(block_num.unwrap().as_usize()).unwrap())
        } else {
            None
        };

        Ok((block, mmr_proof))
    }

    async fn get_notes_by_id(&self, note_ids: &[NoteId]) -> Result<Vec<FetchedNote>, RpcError> {
        // assume all public notes for now
        let notes = self.mock_chain.read().committed_notes().clone();

        let hit_notes = note_ids.iter().filter_map(|id| notes.get(id));
        let mut return_notes = vec![];
        for note in hit_notes {
            let fetched_note = match note {
                MockChainNote::Private(note_id, note_metadata, note_inclusion_proof) => {
                    FetchedNote::Private(*note_id, *note_metadata, note_inclusion_proof.clone())
                },
                MockChainNote::Public(note, note_inclusion_proof) => {
                    FetchedNote::Public(note.clone(), note_inclusion_proof.clone())
                },
            };
            return_notes.push(fetched_note);
        }
        Ok(return_notes)
    }

    async fn submit_proven_transaction(
        &self,
        proven_transaction: ProvenTransaction,
    ) -> Result<BlockNumber, RpcError> {
        // TODO: add some basic validations to test error cases

        {
            let mut mock_chain = self.mock_chain.write();
            mock_chain.add_pending_proven_transaction(proven_transaction.clone());

            mock_chain.prove_next_block().unwrap();
        };

        let block_num = self.get_chain_tip_block_num();

        self.committed_transactions.write().push(TransactionSummary {
            transaction_id: Some(proven_transaction.id().into()),
            block_num: block_num.as_u32(),
            account_id: Some(proven_transaction.account_id().into()),
        });

        Ok(block_num)
    }

    async fn get_account_details(
        &self,
        _account_id: AccountId,
    ) -> Result<FetchedAccount, RpcError> {
        unimplemented!("shouldn't be used for now")
    }

    async fn get_account_proofs(
        &self,
        _: &BTreeSet<ForeignAccount>,
        _code_commitments: Vec<AccountCode>,
    ) -> Result<AccountProofs, RpcError> {
        // TODO: Implement fully
        unimplemented!("shouldn't be used for now")
    }

    async fn check_nullifiers_by_prefix(
        &self,
        prefixes: &[u16],
        from_block_num: BlockNumber,
    ) -> Result<Vec<NullifierUpdate>, RpcError> {
        let nullifiers = self
            .mock_chain
            .read()
            .nullifier_tree()
            .entries()
            .filter_map(|(nullifier, block_num)| {
                if prefixes.contains(&nullifier.prefix()) && block_num >= from_block_num {
                    Some(NullifierUpdate { nullifier, block_num: block_num.as_u32() })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(nullifiers)
    }

    async fn check_nullifiers(&self, _nullifiers: &[Nullifier]) -> Result<Vec<SmtProof>, RpcError> {
        unimplemented!("shouldn't be used for now")
    }

    async fn get_account_state_delta(
        &self,
        _account_id: AccountId,
        _from_block: BlockNumber,
        _to_block: BlockNumber,
    ) -> Result<AccountDelta, RpcError> {
        unimplemented!("shouldn't be used for now")
    }

    async fn get_block_by_number(&self, block_num: BlockNumber) -> Result<ProvenBlock, RpcError> {
        let block = self
            .mock_chain
            .read()
            .proven_blocks()
            .iter()
            .find(|b| b.header().block_num() == block_num)
            .unwrap()
            .clone();

        Ok(block)
    }
}
