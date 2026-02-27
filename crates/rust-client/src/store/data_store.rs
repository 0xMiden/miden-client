use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_protocol::account::{
    Account,
    AccountId,
    PartialAccount,
    PartialStorageMap,
    StorageMap,
    StorageMapWitness,
    StorageSlot,
    StorageSlotContent,
};
use miden_protocol::asset::{AssetVaultKey, AssetWitness};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::MerklePath;
use miden_protocol::crypto::merkle::mmr::{InOrderIndex, PartialMmr};
use miden_protocol::note::NoteScript;
use miden_protocol::transaction::{AccountInputs, PartialBlockchain};
use miden_protocol::vm::FutureMaybeSend;
use miden_protocol::{MastForest, Word, ZERO};
use miden_tx::{DataStore, DataStoreError, MastForestStore, TransactionMastStore};

use super::{AccountStorageFilter, PartialBlockchainFilter, Store};
use crate::rpc::domain::account::{AccountStorageRequirements, StorageMapEntries};
use crate::rpc::{AccountStateAt, NodeRpcClient};
use crate::store::StoreError;
use crate::transaction::account_proof_into_inputs;
use crate::utils::RwLock;

// DATA STORE
// ================================================================================================

/// Wrapper structure that implements [`DataStore`] over any [`Store`].
pub struct ClientDataStore {
    /// Local database containing information about the accounts managed by this client.
    store: alloc::sync::Arc<dyn Store>,
    /// Store used to provide MAST nodes to the transaction executor.
    transaction_mast_store: Arc<TransactionMastStore>,
    /// Cache of foreign account inputs that should be returned to the executor on demand.
    foreign_account_inputs: RwLock<BTreeMap<AccountId, AccountInputs>>,
    /// RPC client used to lazy-load foreign account data on cache miss.
    rpc_api: Arc<dyn NodeRpcClient>,
}

impl ClientDataStore {
    pub fn new(store: alloc::sync::Arc<dyn Store>, rpc_api: Arc<dyn NodeRpcClient>) -> Self {
        Self {
            store,
            transaction_mast_store: Arc::new(TransactionMastStore::new()),
            foreign_account_inputs: RwLock::new(BTreeMap::new()),
            rpc_api,
        }
    }

    pub fn mast_store(&self) -> Arc<TransactionMastStore> {
        self.transaction_mast_store.clone()
    }

    /// Stores the provided foreign account inputs so they can be served to the executor upon
    /// request.
    pub fn register_foreign_account_inputs(
        &self,
        foreign_accounts: impl IntoIterator<Item = AccountInputs>,
    ) {
        let mut cache = self.foreign_account_inputs.write();
        cache.clear();

        for account_inputs in foreign_accounts {
            cache.insert(account_inputs.id(), account_inputs);
        }
    }

    /// Attempts to resolve a storage map witness from the local store.
    ///
    /// Returns `Ok(None)` when the map is not found locally.
    async fn get_local_storage_map_witness(
        &self,
        account_id: AccountId,
        map_root: Word,
        map_key: Word,
    ) -> Result<Option<StorageMapWitness>, DataStoreError> {
        if let Ok(account_storage) = self
            .store
            .get_account_storage(account_id, AccountStorageFilter::Root(map_root))
            .await
        {
            match account_storage.slots().first().map(StorageSlot::content) {
                Some(StorageSlotContent::Map(map)) => return Ok(Some(map.open(&map_key))),
                Some(StorageSlotContent::Value(value)) => {
                    return Err(DataStoreError::Other {
                        error_msg: format!(
                            "found StorageSlotContent::Value with {value} as its value."
                        )
                        .into(),
                        source: None,
                    });
                },
                _ => {},
            }
        }

        Ok(None)
    }

    /// Fetches a storage map witness from the network for a foreign account.
    async fn fetch_remote_storage_map_witness(
        &self,
        account_id: AccountId,
        map_root: Word,
        map_key: Word,
    ) -> Result<StorageMapWitness, DataStoreError> {
        let (slot_name, known_code) = {
            let cache = self.foreign_account_inputs.read();
            let inputs = cache.get(&account_id).ok_or_else(|| DataStoreError::Other {
                error_msg: format!("did not find map with {map_root} as a root for {account_id}")
                    .into(),
                source: None,
            })?;

            let storage_header = inputs.storage().header();
            let slot_name = storage_header
                .slots()
                .find(|slot| slot.slot_type().is_map() && slot.value() == map_root)
                .map(|slot| slot.name().clone())
                .ok_or_else(|| DataStoreError::Other {
                    error_msg: format!(
                        "did not find map slot with root {map_root} for foreign account \
                         {account_id}"
                    )
                    .into(),
                    source: None,
                })?;

            (slot_name, inputs.code().clone())
        };

        let storage_requirements = AccountStorageRequirements::new([(slot_name, &[map_key])]);
        let (_, account_proof) = self
            .rpc_api
            .get_account_proof(
                account_id,
                storage_requirements,
                AccountStateAt::ChainTip,
                Some(known_code),
            )
            .await
            .map_err(|err| {
                DataStoreError::other_with_source("failed to fetch storage map via RPC", err)
            })?;

        let (_, account_details) = account_proof.into_parts();
        let details = account_details.ok_or_else(|| DataStoreError::Other {
            error_msg: format!("RPC returned no account details for public account {account_id}")
                .into(),
            source: None,
        })?;

        let map_detail =
            details.storage_details.map_details.into_iter().next().ok_or_else(|| {
                DataStoreError::Other {
                    error_msg: format!(
                        "RPC returned no storage map details for account {account_id}"
                    )
                    .into(),
                    source: None,
                }
            })?;

        match map_detail.entries {
            StorageMapEntries::AllEntries(entries) => {
                let storage_entries_iter = entries.iter().map(|e| (e.key, e.value));
                let map = StorageMap::with_entries(storage_entries_iter).map_err(|err| {
                    DataStoreError::other_with_source(
                        "failed to build storage map from entries",
                        err,
                    )
                })?;
                Ok(map.open(&map_key))
            },
            StorageMapEntries::EntriesWithProofs(witnesses) => {
                let partial_map = PartialStorageMap::with_witnesses(witnesses).map_err(|err| {
                    DataStoreError::other_with_source(
                        "failed to build partial storage map from witnesses",
                        err,
                    )
                })?;
                partial_map.open(&map_key).map_err(|err| {
                    DataStoreError::other_with_source("failed to open storage map witness", err)
                })
            },
        }
    }
}

impl DataStore for ClientDataStore {
    async fn get_transaction_inputs(
        &self,
        account_id: AccountId,
        mut block_refs: BTreeSet<BlockNumber>,
    ) -> Result<(PartialAccount, BlockHeader, PartialBlockchain), DataStoreError> {
        // Pop last block, used as reference (it does not need to be authenticated manually)
        let ref_block = block_refs.pop_last().ok_or(DataStoreError::other("block set is empty"))?;

        let partial_account_record = self
            .store
            .get_minimal_partial_account(account_id)
            .await?
            .ok_or(DataStoreError::AccountNotFound(account_id))?;

        // New accounts (nonce == 0) need full storage maps as advice inputs for the
        // kernel to validate during account creation. For these, fetch the full account
        // and convert to PartialAccount (which includes full storage for new accounts).
        // Existing accounts use the minimal partial record directly.
        let partial_account: PartialAccount = if partial_account_record.nonce() == ZERO {
            let full_record = self
                .store
                .get_account(account_id)
                .await?
                .ok_or(DataStoreError::AccountNotFound(account_id))?;
            let account: Account = full_record
                .try_into()
                .map_err(|_| DataStoreError::AccountNotFound(account_id))?;
            PartialAccount::from(&account)
        } else {
            partial_account_record
                .try_into()
                .map_err(|_| DataStoreError::AccountNotFound(account_id))?
        };

        // Get header data
        let (block_header, _had_notes) = self
            .store
            .get_block_header_by_num(ref_block)
            .await?
            .ok_or(DataStoreError::BlockNotFound(ref_block))?;

        let block_headers: Vec<BlockHeader> = self
            .store
            .get_block_headers(&block_refs)
            .await?
            .into_iter()
            .map(|(header, _has_notes)| header)
            .collect();

        let partial_mmr =
            build_partial_mmr_with_paths(&self.store, ref_block.as_u32(), &block_headers).await?;

        let partial_blockchain =
            PartialBlockchain::new(partial_mmr, block_headers).map_err(|err| {
                DataStoreError::other_with_source(
                    "error creating PartialBlockchain from internal data",
                    err,
                )
            })?;
        Ok((partial_account, block_header, partial_blockchain))
    }

    async fn get_vault_asset_witnesses(
        &self,
        account_id: AccountId,
        vault_root: Word,
        vault_keys: BTreeSet<AssetVaultKey>,
    ) -> Result<Vec<AssetWitness>, DataStoreError> {
        let mut asset_witnesses = vec![];
        for vault_key in vault_keys {
            match self.store.get_account_asset(account_id, vault_key).await {
                Ok(Some((_, asset_witness))) => {
                    asset_witnesses.push(asset_witness);
                },
                Ok(_) => {
                    let vault = self.store.get_account_vault(account_id).await?;

                    if vault.root() != vault_root {
                        return Err(DataStoreError::Other {
                            error_msg: "Vault root mismatch".into(),
                            source: None,
                        });
                    }

                    let asset_witness =
                        AssetWitness::new(vault.open(vault_key).into()).map_err(|err| {
                            DataStoreError::Other {
                                error_msg: "Failed to open vault asset tree".into(),
                                source: Some(Box::new(err)),
                            }
                        })?;
                    asset_witnesses.push(asset_witness);
                },
                Err(err) => {
                    return Err(DataStoreError::Other {
                        error_msg: "Failed to get account asset".into(),
                        source: Some(Box::new(err)),
                    });
                },
            }
        }
        Ok(asset_witnesses)
    }

    /// Retrieves the [`StorageMapWitness`] requested from the store. Alternatively fetching it from
    /// the RPC if not available locally.
    async fn get_storage_map_witness(
        &self,
        account_id: AccountId,
        map_root: Word,
        map_key: Word,
    ) -> Result<StorageMapWitness, DataStoreError> {
        if let Some(witness) =
            self.get_local_storage_map_witness(account_id, map_root, map_key).await?
        {
            return Ok(witness);
        }

        self.fetch_remote_storage_map_witness(account_id, map_root, map_key).await
    }

    /// Returns the [`AccountInputs`] for the given foreign account from the cache or
    /// alternatively fetching them from the RPC if not available locally.
    ///
    /// # Errors
    /// Returns an [`DataStoreError::AccountNotFound`] error if account is private and has not been
    /// pre-registered with [`Self::register_foreign_account_inputs`].
    async fn get_foreign_account_inputs(
        &self,
        foreign_account_id: AccountId,
        ref_block: BlockNumber,
    ) -> Result<AccountInputs, DataStoreError> {
        // Fast path: check the cache first (drop the read guard before any async work).
        {
            let cache = self.foreign_account_inputs.read();
            if let Some(inputs) = cache.get(&foreign_account_id).cloned() {
                return Ok(inputs);
            }
        }

        // Cache miss, lazy loading is only possible for public accounts.
        if !foreign_account_id.is_public() {
            return Err(DataStoreError::AccountNotFound(foreign_account_id));
        }

        let known_account_code = self
            .store
            .get_foreign_account_code(vec![foreign_account_id])
            .await
            .map_err(|err| {
                DataStoreError::other_with_source("failed to query foreign account code cache", err)
            })?
            .into_values()
            .next();

        let (_, account_proof) = self
            .rpc_api
            .get_account_proof(
                foreign_account_id,
                AccountStorageRequirements::default(),
                AccountStateAt::Block(ref_block),
                known_account_code,
            )
            .await
            .map_err(|err| {
                DataStoreError::other_with_source(
                    "failed to fetch foreign account proof via RPC",
                    err,
                )
            })?;

        let account_inputs = account_proof_into_inputs(account_proof).map_err(|err| {
            DataStoreError::other_with_source("failed to convert account proof to inputs", err)
        })?;

        // Load the account code into the MAST store so the executor can resolve
        // procedure hashes during execution.
        self.transaction_mast_store.load_account_code(account_inputs.code());

        // Persist the fetched code for future transactions.
        let _ = self
            .store
            .upsert_foreign_account_code(foreign_account_id, account_inputs.code().clone())
            .await
            .inspect_err(|err| {
                tracing::warn!(
                    account_id = %foreign_account_id,
                    %err,
                    "Failed to persist foreign account code to store"
                );
            });

        // Cache the result for subsequent calls within the same transaction.
        self.foreign_account_inputs
            .write()
            .insert(foreign_account_id, account_inputs.clone());

        Ok(account_inputs)
    }

    /// Returns the [`NoteScript`] for the given script root from the store or alternatively
    /// fetching it from the RPC if not available locally.
    fn get_note_script(
        &self,
        script_root: Word,
    ) -> impl FutureMaybeSend<Result<Option<NoteScript>, DataStoreError>> {
        let store = self.store.clone();
        let rpc_api = self.rpc_api.clone();

        async move {
            // Fast path: check the local store first.
            if let Ok(note_script) = store.get_note_script(script_root).await {
                return Ok(Some(note_script));
            }

            // Store miss, fetch from the network via RPC.
            let note_script =
                rpc_api.get_note_script_by_root(script_root).await.map_err(|err| {
                    DataStoreError::other_with_source("failed to fetch note script via RPC", err)
                })?;

            // Persist for future lookups.
            if let Err(err) = store.upsert_note_scripts(core::slice::from_ref(&note_script)).await {
                tracing::warn!(
                    %err,
                    "Failed to persist fetched note script to store"
                );
            }

            Ok(Some(note_script))
        }
    }
}

// MAST FOREST STORE
// ================================================================================================

impl MastForestStore for ClientDataStore {
    fn get(&self, procedure_hash: &Word) -> Option<Arc<MastForest>> {
        self.transaction_mast_store.get(procedure_hash)
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Builds a [`PartialMmr`] with a specified forest number and a list of blocks that should be
/// authenticated.
///
/// `authenticated_blocks` cannot contain `forest`. For authenticating the last block we have,
/// the kernel extends the MMR which is why it's not needed here.
async fn build_partial_mmr_with_paths(
    store: &alloc::sync::Arc<dyn Store>,
    forest: u32,
    authenticated_blocks: &[BlockHeader],
) -> Result<PartialMmr, DataStoreError> {
    let mut partial_mmr: PartialMmr = {
        let current_peaks = store
            .get_partial_blockchain_peaks_by_block_num(BlockNumber::from(forest))
            .await?;

        PartialMmr::from_peaks(current_peaks)
    };

    let block_nums: Vec<BlockNumber> =
        authenticated_blocks.iter().map(BlockHeader::block_num).collect();

    let authentication_paths =
        get_authentication_path_for_blocks(store, &block_nums, partial_mmr.forest().num_leaves())
            .await?;

    for (header, path) in authenticated_blocks.iter().zip(authentication_paths.iter()) {
        partial_mmr
            .track(header.block_num().as_usize(), header.commitment(), path)
            .map_err(|err| DataStoreError::other(format!("error constructing MMR: {err}")))?;
    }

    Ok(partial_mmr)
}

/// Retrieves all Partial Blockchain nodes required for authenticating the set of blocks, and then
/// constructs the path for each of them.
///
/// This function assumes `block_nums` doesn't contain values above or equal to `forest`.
/// If there are any such values, the function will panic when calling `mmr_merkle_path_len()`.
async fn get_authentication_path_for_blocks(
    store: &alloc::sync::Arc<dyn Store>,
    block_nums: &[BlockNumber],
    forest: usize,
) -> Result<Vec<MerklePath>, StoreError> {
    let mut node_indices = BTreeSet::new();

    // Calculate all needed nodes indices for generating the paths
    for block_num in block_nums {
        let path_depth = mmr_merkle_path_len(block_num.as_usize(), forest);

        let mut idx = InOrderIndex::from_leaf_pos(block_num.as_usize());

        for _ in 0..path_depth {
            node_indices.insert(idx.sibling());
            idx = idx.parent();
        }
    }

    // Get all MMR nodes based on collected indices
    let node_indices: Vec<InOrderIndex> = node_indices.into_iter().collect();

    let filter = PartialBlockchainFilter::List(node_indices);
    let mmr_nodes = store.get_partial_blockchain_nodes(filter).await?;

    // Construct authentication paths
    let mut authentication_paths = vec![];
    for block_num in block_nums {
        let mut merkle_nodes = vec![];
        let mut idx = InOrderIndex::from_leaf_pos(block_num.as_usize());

        while let Some(node) = mmr_nodes.get(&idx.sibling()) {
            merkle_nodes.push(*node);
            idx = idx.parent();
        }
        let path = MerklePath::new(merkle_nodes);
        authentication_paths.push(path);
    }

    Ok(authentication_paths)
}

/// Calculates the merkle path length for an MMR of a specific forest and a leaf index
/// `leaf_index` is a 0-indexed leaf number and `forest` is the total amount of leaves
/// in the MMR at this point.
fn mmr_merkle_path_len(leaf_index: usize, forest: usize) -> usize {
    let before: usize = forest & leaf_index;
    let after = forest ^ before;

    after.ilog2() as usize
}
