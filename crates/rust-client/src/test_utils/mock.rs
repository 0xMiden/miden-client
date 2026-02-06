use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::delta::AccountUpdateDetails;
use miden_protocol::account::{AccountCode, AccountId, StorageSlot, StorageSlotContent};
use miden_protocol::address::NetworkId;
use miden_protocol::block::{BlockHeader, BlockNumber, ProvenBlock};
use miden_protocol::crypto::merkle::mmr::{Forest, Mmr, MmrProof};
use miden_protocol::crypto::merkle::smt::SmtProof;
use miden_protocol::note::{NoteHeader, NoteId, NoteScript, NoteTag, Nullifier};
use miden_protocol::transaction::{ProvenTransaction, TransactionInputs};
use miden_testing::{MockChain, MockChainNote};
use miden_tx::utils::sync::RwLock;

use crate::Client;
use crate::rpc::domain::account::{
    AccountDetails,
    AccountProof,
    AccountStorageDetails,
    AccountStorageMapDetails,
    AccountUpdateSummary,
    AccountVaultDetails,
    FetchedAccount,
    StorageMapEntries,
    StorageMapEntry,
};
use crate::rpc::domain::account_vault::{AccountVaultInfo, AccountVaultUpdate};
use crate::rpc::domain::note::{CommittedNote, FetchedNote, NoteSyncInfo};
use crate::rpc::domain::nullifier::NullifierUpdate;
use crate::rpc::domain::storage_map::{StorageMapInfo, StorageMapUpdate};
use crate::rpc::domain::sync::StateSyncInfo;
use crate::rpc::domain::transaction::{TransactionRecord, TransactionsInfo};
use crate::rpc::generated::account::AccountSummary;
use crate::rpc::generated::note::NoteSyncRecord;
use crate::rpc::generated::rpc::{BlockRange, SyncStateResponse};
use crate::rpc::generated::transaction::TransactionSummary;
use crate::rpc::{AccountStateAt, NodeRpcClient, RpcError};
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
    // Constant to use in mocked pagination.
    const PAGINATION_BLOCK_LIMIT: u32 = 5;

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
            .body()
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
        request_block_range: BlockRange,
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
                if note_tags.contains(&note.metadata().tag())
                    && block_num.as_u32() > request_block_range.block_from
                {
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
        let from_block_num =
            if request_block_range.block_from == self.get_chain_tip_block_num().as_u32() {
                next_block_num.as_usize()
            } else {
                request_block_range.block_from as usize + 1
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
                block.header().block_num().as_u32() > request_block_range.block_from
                    && block.header().block_num() <= next_block_num
            })
            .flat_map(|block| {
                block.body().transactions().as_slice().iter().map(|tx| TransactionSummary {
                    transaction_id: Some(tx.id().into()),
                    block_num: next_block_num.as_u32(),
                    account_id: Some(tx.account_id().into()),
                })
            })
            .collect();

        let mut accounts = vec![];

        for (block_num, updates) in self.account_commitment_updates.read().iter() {
            if block_num.as_u32() > request_block_range.block_from && *block_num <= next_block_num {
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

    /// Retrieves account vault updates in a given block range.
    /// This method tries to simulate pagination by limiting the number of blocks processed per
    /// request.
    fn get_sync_account_vault_request(
        &self,
        block_from: BlockNumber,
        block_to: Option<BlockNumber>,
        account_id: AccountId,
    ) -> AccountVaultInfo {
        let chain_tip = self.get_chain_tip_block_num();
        let target_block = block_to.unwrap_or(chain_tip).min(chain_tip);

        let page_end_block: BlockNumber = (block_from.as_u32() + Self::PAGINATION_BLOCK_LIMIT)
            .min(target_block.as_u32())
            .into();

        let mut updates = vec![];
        for block in self.mock_chain.read().proven_blocks() {
            let block_number = block.header().block_num();
            // Only include blocks in range (block_from, page_end_block]
            if block_number <= block_from || block_number > page_end_block {
                continue;
            }

            for update in block
                .body()
                .updated_accounts()
                .iter()
                .filter(|block_acc_update| block_acc_update.account_id() == account_id)
            {
                let AccountUpdateDetails::Delta(account_delta) = update.details().clone() else {
                    continue;
                };

                let vault_delta = account_delta.vault();

                for asset in vault_delta.added_assets() {
                    let account_vault_update = AccountVaultUpdate {
                        block_num: block_number,
                        asset: Some(asset),
                        vault_key: asset.vault_key(),
                    };
                    updates.push(account_vault_update);
                }
            }
        }

        AccountVaultInfo {
            chain_tip,
            block_number: page_end_block,
            updates,
        }
    }

    /// Retrieves transactions in a given block range that match the provided account IDs
    fn get_sync_transactions_request(
        &self,
        block_from: BlockNumber,
        block_to: Option<BlockNumber>,
        account_ids: &[AccountId],
    ) -> TransactionsInfo {
        let chain_tip = self.get_chain_tip_block_num();
        let block_to = match block_to {
            Some(block_to) => block_to,
            None => chain_tip,
        };

        let mut transaction_records = vec![];
        for block in self.mock_chain.read().proven_blocks() {
            let block_number = block.header().block_num();
            if block_number <= block_from || block_number > block_to {
                continue;
            }

            for transaction_header in block.body().transactions().as_slice() {
                if !account_ids.contains(&transaction_header.account_id()) {
                    continue;
                }

                transaction_records.push(TransactionRecord {
                    block_num: block_number,
                    transaction_header: transaction_header.clone(),
                });
            }
        }

        TransactionsInfo {
            chain_tip,
            block_num: block_to,
            transaction_records,
        }
    }

    /// Retrieves storage map updates in a given block range.
    ///
    /// This method tries to simulate pagination of the real node.
    fn get_sync_storage_maps_request(
        &self,
        block_from: BlockNumber,
        block_to: Option<BlockNumber>,
        account_id: AccountId,
    ) -> StorageMapInfo {
        let chain_tip = self.get_chain_tip_block_num();
        let target_block = block_to.unwrap_or(chain_tip).min(chain_tip);

        let page_end_block: BlockNumber = (block_from.as_u32() + Self::PAGINATION_BLOCK_LIMIT)
            .min(target_block.as_u32())
            .into();

        let mut updates = vec![];
        for block in self.mock_chain.read().proven_blocks() {
            let block_number = block.header().block_num();
            if block_number <= block_from || block_number > page_end_block {
                continue;
            }

            for update in block
                .body()
                .updated_accounts()
                .iter()
                .filter(|block_acc_update| block_acc_update.account_id() == account_id)
            {
                let AccountUpdateDetails::Delta(account_delta) = update.details().clone() else {
                    continue;
                };

                let storage_delta = account_delta.storage();

                for (slot_name, map_delta) in storage_delta.maps() {
                    for (key, value) in map_delta.entries() {
                        let storage_map_info = StorageMapUpdate {
                            block_num: block_number,
                            slot_name: slot_name.clone(),
                            key: (*key).into(),
                            value: *value,
                        };
                        updates.push(storage_map_info);
                    }
                }
            }
        }

        StorageMapInfo {
            chain_tip,
            block_number: page_end_block,
            updates,
        }
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
                        metadata: Some(note.metadata().clone().into()),
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

    pub fn get_public_available_notes(&self) -> Vec<MockChainNote> {
        self.mock_chain
            .read()
            .committed_notes()
            .values()
            .filter(|n| matches!(n, MockChainNote::Public(_, _)))
            .cloned()
            .collect()
    }

    pub fn get_private_available_notes(&self) -> Vec<MockChainNote> {
        self.mock_chain
            .read()
            .committed_notes()
            .values()
            .filter(|n| matches!(n, MockChainNote::Private(_, _, _)))
            .cloned()
            .collect()
    }

    pub fn advance_blocks(&self, num_blocks: u32) {
        let current_height = self.get_chain_tip_block_num();
        let mut mock_chain = self.mock_chain.write();
        mock_chain.prove_until_block(current_height + num_blocks).unwrap();
    }
}
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
        block_to: Option<BlockNumber>,
        note_tags: &BTreeSet<NoteTag>,
    ) -> Result<NoteSyncInfo, RpcError> {
        let block_range = BlockRange {
            block_from: block_num.as_u32(),
            block_to: block_to.map(|b| b.as_u32()),
        };

        let response = self.get_sync_state_request(block_range, note_tags, &[])?;

        let response = NoteSyncInfo {
            chain_tip: response.chain_tip.into(),
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
        let block_range = BlockRange {
            block_from: block_num.as_u32(),
            block_to: None,
        };
        let response = self.get_sync_state_request(block_range, note_tags, account_ids)?;

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
                    let note_header = NoteHeader::new(*note_id, note_metadata.clone());
                    FetchedNote::Private(note_header, note_inclusion_proof.clone())
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
        _tx_inputs: TransactionInputs, // Unnecessary for testing client itself.
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
                    last_block_num: *block_num,
                })
            })
            .unwrap();

        if let Ok(account) = self.mock_chain.read().committed_account(account_id) {
            Ok(FetchedAccount::new_public(account.clone(), summary))
        } else {
            Ok(FetchedAccount::new_private(account_id, summary))
        }
    }

    /// Returns the account proof for the specified account. The `known_account_code` parameter
    /// is ignored in the mock implementation and the latest account code is always returned.
    async fn get_account(
        &self,
        foreign_account: ForeignAccount,
        account_state: AccountStateAt,
        _known_account_code: Option<AccountCode>,
    ) -> Result<(BlockNumber, AccountProof), RpcError> {
        let mock_chain = self.mock_chain.read();

        let block_number = match account_state {
            AccountStateAt::Block(number) => number,
            AccountStateAt::ChainTip => mock_chain.latest_block_header().block_num(),
        };

        let headers = match &foreign_account {
            ForeignAccount::Public(account_id, account_storage_requirements) => {
                let account = mock_chain.committed_account(*account_id).unwrap();

                let mut map_details = vec![];
                for slot_name in account_storage_requirements.inner().keys() {
                    if let Some(StorageSlotContent::Map(storage_map)) =
                        account.storage().get(slot_name).map(StorageSlot::content)
                    {
                        let entries: Vec<StorageMapEntry> = storage_map
                            .entries()
                            .map(|(key, value)| StorageMapEntry { key: *key, value: *value })
                            .collect();

                        let too_many_entries = entries.len() > 1000;
                        let account_storage_map_detail = AccountStorageMapDetails {
                            slot_name: slot_name.clone(),
                            too_many_entries,
                            entries: StorageMapEntries::AllEntries(entries),
                        };

                        map_details.push(account_storage_map_detail);
                    } else {
                        panic!("Storage slot {slot_name} is not a map");
                    }
                }

                let storage_details = AccountStorageDetails {
                    header: account.storage().to_header(),
                    map_details,
                };

                let mut assets = vec![];
                for asset in account.vault().assets() {
                    assets.push(asset);
                }
                let vault_details = AccountVaultDetails {
                    too_many_assets: assets.len() > 1000,
                    assets,
                };

                Some(AccountDetails {
                    header: account.into(),
                    storage_details,
                    code: account.code().clone(),
                    vault_details,
                })
            },
            ForeignAccount::Private(_) => None,
        };

        let witness = mock_chain.account_tree().open(foreign_account.account_id());

        let proof = AccountProof::new(witness, headers).unwrap();

        Ok((block_number, proof))
    }

    /// Returns the nullifiers created after the specified block number that match the provided
    /// prefixes.
    async fn sync_nullifiers(
        &self,
        prefixes: &[u16],
        from_block_num: BlockNumber,
        block_to: Option<BlockNumber>,
    ) -> Result<Vec<NullifierUpdate>, RpcError> {
        let nullifiers = self
            .mock_chain
            .read()
            .nullifier_tree()
            .entries()
            .filter_map(|(nullifier, block_num)| {
                let within_range = if let Some(to_block) = block_to {
                    block_num >= from_block_num && block_num <= to_block
                } else {
                    block_num >= from_block_num
                };

                if prefixes.contains(&nullifier.prefix()) && within_range {
                    Some(NullifierUpdate { nullifier, block_num })
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

    async fn get_note_script_by_root(&self, root: Word) -> Result<NoteScript, RpcError> {
        let note = self
            .get_available_notes()
            .iter()
            .find(|note| note.note().is_some_and(|n| n.script().root() == root))
            .unwrap()
            .clone();

        Ok(note.note().unwrap().script().clone())
    }

    async fn sync_storage_maps(
        &self,
        block_from: BlockNumber,
        block_to: Option<BlockNumber>,
        account_id: AccountId,
    ) -> Result<StorageMapInfo, RpcError> {
        let mut all_updates = Vec::new();
        let mut current_block_from = block_from;
        let chain_tip = self.get_chain_tip_block_num();
        let target_block = block_to.unwrap_or(chain_tip).min(chain_tip);

        loop {
            let response =
                self.get_sync_storage_maps_request(current_block_from, block_to, account_id);
            all_updates.extend(response.updates);

            if response.block_number >= target_block {
                return Ok(StorageMapInfo {
                    chain_tip: response.chain_tip,
                    block_number: response.block_number,
                    updates: all_updates,
                });
            }

            current_block_from = (response.block_number.as_u32() + 1).into();
        }
    }

    async fn sync_account_vault(
        &self,
        block_from: BlockNumber,
        block_to: Option<BlockNumber>,
        account_id: AccountId,
    ) -> Result<AccountVaultInfo, RpcError> {
        let mut all_updates = Vec::new();
        let mut current_block_from = block_from;
        let chain_tip = self.get_chain_tip_block_num();
        let target_block = block_to.unwrap_or(chain_tip).min(chain_tip);

        loop {
            let response =
                self.get_sync_account_vault_request(current_block_from, block_to, account_id);
            all_updates.extend(response.updates);

            if response.block_number >= target_block {
                return Ok(AccountVaultInfo {
                    chain_tip: response.chain_tip,
                    block_number: response.block_number,
                    updates: all_updates,
                });
            }

            current_block_from = (response.block_number.as_u32() + 1).into();
        }
    }

    async fn sync_transactions(
        &self,
        block_from: BlockNumber,
        block_to: Option<BlockNumber>,
        account_ids: Vec<AccountId>,
    ) -> Result<TransactionsInfo, RpcError> {
        let response = self.get_sync_transactions_request(block_from, block_to, &account_ids);
        Ok(response)
    }

    async fn get_network_id(&self) -> Result<NetworkId, RpcError> {
        Ok(NetworkId::Testnet)
    }

    async fn get_rpc_limits(&self) -> Result<crate::rpc::RpcLimits, RpcError> {
        Ok(crate::rpc::RpcLimits::default())
    }

    async fn set_rpc_limits(&self, _limits: crate::rpc::RpcLimits) {
        // Not needed for mock client
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

        for update in block.body().updated_accounts() {
            updates.insert(update.account_id(), update.final_state_commitment());
        }

        if updates.is_empty() {
            continue;
        }

        account_commitment_updates.insert(block_num, updates);
    }
    account_commitment_updates
}
