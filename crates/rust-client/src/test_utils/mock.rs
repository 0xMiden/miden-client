use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_objects::Word;
use miden_objects::account::{AccountCode, AccountId, StorageSlot};
use miden_objects::block::{BlockHeader, BlockNumber, ProvenBlock};
use miden_objects::crypto::merkle::{Forest, Mmr, MmrProof, SmtProof};
use miden_objects::note::{NoteId, NoteTag, Nullifier};
use miden_objects::transaction::ProvenTransaction;
use miden_testing::{MockChain, MockChainNote};
use miden_tx::utils::sync::RwLock;

use crate::Client;
use crate::rpc::domain::account::{
    AccountProof,
    AccountProofs,
    AccountUpdateSummary,
    FetchedAccount,
    StateHeaders,
};
use crate::rpc::domain::note::{CommittedNote, FetchedNote, NoteSyncInfo};
use crate::rpc::domain::nullifier::NullifierUpdate;
use crate::rpc::domain::sync::StateSyncInfo;
use crate::rpc::generated::account::AccountSummary;
use crate::rpc::generated::note::NoteSyncRecord;
use crate::rpc::generated::rpc_store::SyncStateResponse;
use crate::rpc::generated::transaction::TransactionSummary;
use crate::rpc::{NodeRpcClient, RpcError};
use crate::transaction::ForeignAccount;

pub type MockClient<AUTH> = Client<AUTH>;

/// Mock RPC API
///
/// This struct implements the RPC API used by the client to communicate with the node. It simulates
/// most of the functionality of the actual node, with some small differences:
/// - It uses a [`MockChain`] to simulate the blockchain state.
/// - Blocks are not automatically created after time passes, but rather new blocks are created when
///   calling the `prove_block` method.
/// - Network account and transactions aren't supported in the current version.
/// - Account update block numbers aren't tracked, so any endpoint that returns when certain account
///   updates were made will return the chain tip block number instead.
#[derive(Clone)]
pub struct MockRpcApi {
    account_commitment_updates: Arc<RwLock<BTreeMap<BlockNumber, BTreeMap<AccountId, Word>>>>,
    pub mock_chain: Arc<RwLock<MockChain>>,
}

impl Default for MockRpcApi {
    fn default() -> Self {
        Self::new(MockChain::new())
    }
}

impl MockRpcApi {
    /// Creates a new [`MockRpcApi`] instance with the state of the provided [`MockChain`].
    pub fn new(mock_chain: MockChain) -> Self {
        Self {
            account_commitment_updates: Arc::new(RwLock::new(build_account_updates(&mock_chain))),
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

    /// Advances the mock chain by proving the next block, committing all pending objects to the
    /// chain in the process.
    pub fn prove_block(&self) {
        let proven_block = self.mock_chain.write().prove_next_block().unwrap();
        let mut account_commitment_updates = self.account_commitment_updates.write();
        let block_num = proven_block.header().block_num();
        let updates: BTreeMap<AccountId, Word> = proven_block
            .updated_accounts()
            .iter()
            .map(|update| (update.account_id(), update.final_state_commitment()))
            .collect();

        if !updates.is_empty() {
            account_commitment_updates.insert(block_num, updates);
        }
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
        account_ids: &[AccountId],
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
        let notes = self.get_notes_in_block(next_block_num, note_tags, account_ids);

        let transactions = self
            .mock_chain
            .read()
            .proven_blocks()
            .iter()
            .filter(|block| {
                block.header().block_num() > request_block_num
                    && block.header().block_num() <= next_block_num
            })
            .flat_map(|block| {
                block.transactions().as_slice().iter().map(|tx| TransactionSummary {
                    transaction_id: Some(tx.id().into()),
                    block_num: next_block_num.as_u32(),
                    account_id: Some(tx.account_id().into()),
                })
            })
            .collect();

        let mut accounts = vec![];

        for (block_num, updates) in self.account_commitment_updates.read().iter() {
            if *block_num > request_block_num && *block_num <= next_block_num {
                accounts.extend(updates.iter().map(|(account_id, commitment)| AccountSummary {
                    account_id: Some((*account_id).into()),
                    account_commitment: Some(commitment.into()),
                    block_num: block_num.as_u32(),
                }));
            }
        }

        Ok(SyncStateResponse {
            chain_tip: self.get_chain_tip_block_num().as_u32(),
            block_header: Some(next_block.into()),
            mmr_delta: Some(mmr_delta.try_into()?),
            accounts,
            transactions,
            notes,
        })
    }

    /// Retrieves notes that are included in the specified block number.
    fn get_notes_in_block(
        &self,
        block_num: BlockNumber,
        note_tags: &BTreeSet<NoteTag>,
        account_ids: &[AccountId],
    ) -> Vec<NoteSyncRecord> {
        self.mock_chain
            .read()
            .committed_notes()
            .values()
            .filter_map(move |note| {
                if note.inclusion_proof().location().block_num() == block_num
                    && (note_tags.contains(&note.metadata().tag())
                        || account_ids.contains(&note.metadata().sender()))
                {
                    Some(NoteSyncRecord {
                        note_index_in_block: u32::from(
                            note.inclusion_proof().location().node_index_in_block(),
                        ),
                        note_id: Some(note.id().into()),
                        metadata: Some((*note.metadata()).into()),
                        inclusion_path: Some(note.inclusion_proof().note_path().clone().into()),
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
    async fn set_genesis_commitment(&self, _commitment: Word) -> Result<(), RpcError> {
        // The mock client doesn't use accept headers, so we don't need to do anything here.
        Ok(())
    }

    /// Returns the next note updates after the specified block number. Only notes that match the
    /// provided tags will be returned.
    async fn sync_notes(
        &self,
        block_num: BlockNumber,
        note_tags: &BTreeSet<NoteTag>,
    ) -> Result<NoteSyncInfo, RpcError> {
        let response = self.get_sync_state_request(block_num, note_tags, &[])?;

        let response = NoteSyncInfo {
            chain_tip: response.chain_tip,
            block_header: response.block_header.unwrap().try_into().unwrap(),
            mmr_path: self.get_mmr().open(block_num.as_usize()).unwrap().merkle_path,
            notes: response
                .notes
                .into_iter()
                .map(|note| {
                    let note_id: NoteId = note.note_id.unwrap().try_into().unwrap();
                    let note_index = u16::try_from(note.note_index_in_block).unwrap();
                    let merkle_path = note.inclusion_path.unwrap().try_into().unwrap();
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
        account_ids: &[AccountId],
        note_tags: &BTreeSet<NoteTag>,
    ) -> Result<StateSyncInfo, RpcError> {
        let response = self.get_sync_state_request(block_num, note_tags, account_ids)?;

        Ok(response.try_into().unwrap())
    }

    /// Retrieves the block header for the specified block number. If the block number is not
    /// provided, the chain tip block header will be returned.
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

    /// Returns the node's tracked notes that match the provided note IDs.
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

    /// Simulates the submission of a proven transaction to the node. This will create a new block
    /// just for the new transaction and return the block number of the newly created block.
    async fn submit_proven_transaction(
        &self,
        proven_transaction: ProvenTransaction,
    ) -> Result<BlockNumber, RpcError> {
        // TODO: add some basic validations to test error cases

        {
            let mut mock_chain = self.mock_chain.write();
            mock_chain.add_pending_proven_transaction(proven_transaction.clone());
        };

        let block_num = self.get_chain_tip_block_num();

        Ok(block_num)
    }

    /// Returns the node's tracked account details for the specified account ID.
    async fn get_account_details(&self, account_id: AccountId) -> Result<FetchedAccount, RpcError> {
        let summary = self
            .account_commitment_updates
            .read()
            .iter()
            .rev()
            .find_map(|(block_num, updates)| {
                updates.get(&account_id).map(|commitment| AccountUpdateSummary {
                    commitment: *commitment,
                    last_block_num: block_num.as_u32(),
                })
            })
            .unwrap();

        if let Ok(account) = self.mock_chain.read().committed_account(account_id) {
            Ok(FetchedAccount::Public(account.clone(), summary))
        } else {
            Ok(FetchedAccount::Private(account_id, summary))
        }
    }

    /// Returns the account proofs for the specified accounts. The `known_account_codes` parameter
    /// is ignored in the mock implementation and the latest account code is always returned.
    async fn get_account_proofs(
        &self,
        account_storage_requests: &BTreeSet<ForeignAccount>,
        _known_account_codes: BTreeMap<AccountId, AccountCode>,
    ) -> Result<AccountProofs, RpcError> {
        let mock_chain = self.mock_chain.read();

        let chain_tip = mock_chain.latest_block_header().block_num();
        let mut proofs = vec![];
        for account in account_storage_requests {
            let headers = match account {
                ForeignAccount::Public(account_id, account_storage_requirements) => {
                    let account = mock_chain.committed_account(*account_id).unwrap();

                    let mut storage_slots = BTreeMap::new();
                    for (index, storage_keys) in account_storage_requirements.inner() {
                        if let Some(StorageSlot::Map(storage_map)) =
                            account.storage().slots().get(*index as usize)
                        {
                            let proofs = storage_keys
                                .iter()
                                .map(|map_key| storage_map.open(map_key))
                                .collect::<Vec<_>>();
                            storage_slots.insert(*index, proofs);
                        } else {
                            panic!("Storage slot at index {} is not a map", index);
                        }
                    }

                    Some(StateHeaders {
                        account_header: account.into(),
                        storage_header: account.storage().to_header(),
                        code: account.code().clone(),
                        storage_slots,
                    })
                },
                ForeignAccount::Private(_) => None,
            };

            let witness = mock_chain.account_tree().open(account.account_id());

            proofs.push(AccountProof::new(witness, headers).unwrap());
        }

        Ok((chain_tip, proofs))
    }

    /// Returns the nullifiers created after the specified block number that match the provided
    /// prefixes.
    async fn sync_nullifiers(
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

    /// Returns proofs for all the provided nullifiers.
    async fn check_nullifiers(&self, nullifiers: &[Nullifier]) -> Result<Vec<SmtProof>, RpcError> {
        Ok(nullifiers
            .iter()
            .map(|nullifier| self.mock_chain.read().nullifier_tree().open(nullifier).into_proof())
            .collect())
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

    async fn get_note_script_by_root(&self, _script_root: Word) -> Result<NoteScript, RpcError> {
        // For the mock implementation, we'll return a simple note script
        // In a real implementation, this would look up the script by its root hash
        use alloc::sync::Arc;

        use miden_objects::MastForest;
        use miden_objects::note::NoteScript;
        use miden_objects::vm::Program;

        let mast_forest = Arc::new(MastForest::new());
        let program = Program::new(mast_forest, miden_core::MastNodeId::new(0)); // Simple placeholder program
        let script = NoteScript::new(program);

        Ok(script)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<MockChain> for MockRpcApi {
    fn from(mock_chain: MockChain) -> Self {
        MockRpcApi::new(mock_chain)
    }
}

// HELPERS
// ================================================================================================

fn build_account_updates(
    mock_chain: &MockChain,
) -> BTreeMap<BlockNumber, BTreeMap<AccountId, Word>> {
    let mut account_commitment_updates = BTreeMap::new();
    for block in mock_chain.proven_blocks() {
        let block_num = block.header().block_num();
        let mut updates = BTreeMap::new();

        for update in block.updated_accounts() {
            updates.insert(update.account_id(), update.final_state_commitment());
        }

        if updates.is_empty() {
            continue;
        }

        account_commitment_updates.insert(block_num, updates);
    }
    account_commitment_updates
}
