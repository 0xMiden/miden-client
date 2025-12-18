use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_objects::account::{AccountId, PartialAccount, StorageSlot};
use miden_objects::asset::{AssetVaultKey, AssetWitness};
use miden_objects::block::{BlockHeader, BlockNumber};
use miden_objects::crypto::merkle::{InOrderIndex, MerklePath, PartialMmr};
use miden_objects::note::NoteScript;
use miden_objects::transaction::{AccountInputs, PartialBlockchain};
use miden_objects::vm::FutureMaybeSend;
use miden_objects::{MastForest, Word};
use miden_tx::{DataStore, DataStoreError, MastForestStore, TransactionMastStore};

use super::{PartialBlockchainFilter, Store};
use crate::rpc::NodeRpcClient;
use super::{AccountStorageFilter, PartialBlockchainFilter, Store};
use crate::store::StoreError;
use crate::transaction::ForeignAccount;
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
    /// Optional RPC client for lazy loading of data not found in local store.
    rpc_client: Option<Arc<dyn NodeRpcClient>>,
}

impl ClientDataStore {
    /// Creates a new `ClientDataStore` with an optional RPC client for lazy loading.
    ///
    /// If an RPC client is provided, the data store will attempt to fetch missing data
    /// (such as note scripts and foreign account data) from the network when not found locally.
    pub fn new(store: alloc::sync::Arc<dyn Store>) -> Self {
        Self {
            store,
            transaction_mast_store: Arc::new(TransactionMastStore::new()),
            foreign_account_inputs: RwLock::new(BTreeMap::new()),
            rpc_client: None,
        }
    }

    /// Creates a new `ClientDataStore` with an RPC client for lazy loading.
    pub fn with_rpc(
        store: alloc::sync::Arc<dyn Store>,
        rpc_client: Arc<dyn NodeRpcClient>,
    ) -> Self {
        Self {
            store,
            transaction_mast_store: Arc::new(TransactionMastStore::new()),
            foreign_account_inputs: RwLock::new(BTreeMap::new()),
            rpc_client: Some(rpc_client),
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
        let partial_account: PartialAccount = partial_account_record
            .try_into()
            .map_err(|_| DataStoreError::AccountNotFound(account_id))?;

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
            match self.store.get_account_asset(account_id, vault_key.faucet_id_prefix()).await {
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

    async fn get_storage_map_witness(
        &self,
        account_id: AccountId,
        map_root: Word,
        map_key: Word,
    ) -> Result<miden_objects::account::StorageMapWitness, DataStoreError> {
        let account_storage = self
            .store
            .get_account_storage(account_id, AccountStorageFilter::Root(map_root))
            .await?;

        match account_storage.slots().first() {
            Some(StorageSlot::Map(map)) => {
                let witness = map.open(&map_key);
                Ok(witness)
            },
            Some(StorageSlot::Value(value)) => Err(DataStoreError::Other {
                error_msg: format!("found StorageSlot::Value with {value} as its value.").into(),
                source: None,
            }),
            _ => Err(DataStoreError::Other {
                error_msg: format!("did not find map with {map_root} as a root for {account_id}")
                    .into(),
                source: None,
            }),
        }
    }

    async fn get_foreign_account_inputs(
        &self,
        foreign_account_id: AccountId,
        _ref_block: BlockNumber,
    ) -> Result<AccountInputs, DataStoreError> {
        // First, check the cache
        {
            let cache = self.foreign_account_inputs.read();
            if let Some(inputs) = cache.get(&foreign_account_id) {
                return Ok(inputs.clone());
            }
        }

        // If not in cache and RPC client is available, try fetching from the network
        if let Some(rpc) = &self.rpc_client {
            // Try to fetch as a public account with empty storage requirements
            // This will work for public accounts, but won't work for private accounts
            // (which require PartialAccount to be provided upfront)
            if foreign_account_id.is_public() {
                let foreign_account = ForeignAccount::Public(
                    foreign_account_id,
                    crate::rpc::domain::account::AccountStorageRequirements::default(),
                );

                let known_account_codes = self
                    .store
                    .get_foreign_account_code(vec![foreign_account_id])
                    .await
                    .map_err(|err| {
                        DataStoreError::other(format!("Failed to get foreign account code: {err}"))
                    })?;

                match rpc
                    .get_account_proofs(
                        &[foreign_account].into_iter().collect(),
                        known_account_codes,
                    )
                    .await
                {
                    Ok((_block_num, account_proofs)) => {
                        if let Some(account_proof) = account_proofs
                            .into_iter()
                            .find(|proof| proof.account_id() == foreign_account_id)
                        {
                            let account_inputs: AccountInputs =
                                account_proof.try_into().map_err(|err| {
                                    DataStoreError::other(format!(
                                        "Failed to convert account proof to AccountInputs: {err}"
                                    ))
                                })?;

                            // Cache the fetched account inputs for future use
                            {
                                let mut cache = self.foreign_account_inputs.write();
                                cache.insert(foreign_account_id, account_inputs.clone());
                            }

                            // Update the foreign account code cache
                            if let Err(err) = self
                                .store
                                .upsert_foreign_account_code(
                                    foreign_account_id,
                                    account_inputs.code().clone(),
                                )
                                .await
                            {
                                // Log but don't fail - we still have the account inputs to return
                                let _ = err;
                            }

                            return Ok(account_inputs);
                        }
                    },
                    Err(rpc_err) => {
                        return Err(DataStoreError::other(format!(
                            "Failed to fetch foreign account {foreign_account_id} via RPC: {rpc_err}",
                        )));
                    },
                }
            }
        }

        Err(DataStoreError::AccountNotFound(foreign_account_id))
    }

    fn get_note_script(
        &self,
        script_root: Word,
    ) -> impl FutureMaybeSend<Result<Option<NoteScript>, DataStoreError>> {
        let store = self.store.clone();
        let rpc_client = self.rpc_client.clone();

        async move {
            // First, try to get the note script from the local store
            match store.get_note_script(script_root).await {
                Ok(note_script) => Ok(note_script),
                Err(_) => {
                    // If not found locally and RPC client is available, try fetching from the
                    // network
                    if let Some(rpc) = rpc_client {
                        match rpc.get_note_script_by_root(script_root).await {
                            Ok(note_script) => {
                                // Cache the fetched script in the local store for future use.
                                // Since we know the script wasn't in the local store
                                // (get_note_script failed),
                                // upsert should effectively be an insert. If it fails (e.g., due to
                                // database issues or concurrent
                                // writes), we continue anyway since caching is just an
                                // optimization - we still have the valid script to return.
                                if let Err(_err) = store
                                    .upsert_note_scripts(core::slice::from_ref(&note_script))
                                    .await
                                {
                                    // In a no_std environment, we can't easily log, so we just
                                    // continue
                                }
                                Ok(note_script)
                            },
                            Err(rpc_err) => Err(DataStoreError::other(format!(
                                "Note script with root {script_root} not found via RPC: {rpc_err}",
                            ))),
                        }
                    } else {
                        Err(DataStoreError::other(format!(
                            "Note script with root {script_root} not found in local store",
                        )))
                    }
                },
            }
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
