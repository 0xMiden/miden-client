use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_protocol::account::{
    Account,
    AccountId,
    PartialAccount,
    StorageMapKey,
    StorageMapWitness,
    StorageSlotContent,
};
use miden_protocol::asset::{AssetVaultKey, AssetWitness};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::note::{NoteScript, NoteScriptRoot};
use miden_protocol::transaction::{AccountInputs, PartialBlockchain};
use miden_protocol::vm::FutureMaybeSend;
use miden_protocol::{MastForest, Word};
use miden_tx::{DataStore, DataStoreError, MastForestStore, TransactionMastStore};

use crate::store::data_store::ClientDataStore;

// IN-MEMORY BATCH DATA STORE
// ================================================================================================

/// A [`DataStore`] scoped to a single batch on one local account. Wraps an inner
/// [`ClientDataStore`] and, for the batch's account, substitutes the
/// current in-batch account state (produced by the most recent successful push)
/// in place of the state read from the store. All other reads remain unchanged.
pub(crate) struct InMemoryBatchDataStore {
    inner: ClientDataStore,
    batch_account_id: AccountId,
    current_account: Account,
}

impl InMemoryBatchDataStore {
    pub(crate) fn new(
        inner: ClientDataStore,
        batch_account_id: AccountId,
        initial_account: Account,
    ) -> Self {
        Self {
            inner,
            batch_account_id,
            current_account: initial_account,
        }
    }

    /// Replace the current account state. Called by `BatchBuilder::push` after
    /// a successful execute+prove to expose the post-tx state to the next push.
    pub(crate) fn set_current_account(&mut self, new_state: Account) {
        self.current_account = new_state;
    }

    /// Returns a reference to the current in-batch account state.
    pub(crate) fn current_account(&self) -> &Account {
        &self.current_account
    }

    /// Returns the inner [`ClientDataStore`]'s MAST forest store. Used by `BatchBuilder::push`
    /// to load account and foreign-account code before execution.
    pub(crate) fn mast_store(&self) -> Arc<TransactionMastStore> {
        self.inner.mast_store()
    }

    /// Registers foreign account inputs on the inner [`ClientDataStore`] so that the executor
    /// can find them during batch-level transaction execution.
    pub(crate) fn register_foreign_account_inputs(
        &self,
        foreign_accounts: impl IntoIterator<Item = AccountInputs>,
    ) {
        self.inner.register_foreign_account_inputs(foreign_accounts);
    }
}

// DATA STORE IMPL
// ================================================================================================

impl DataStore for InMemoryBatchDataStore {
    async fn get_transaction_inputs(
        &self,
        account_id: AccountId,
        ref_blocks: BTreeSet<BlockNumber>,
    ) -> Result<(PartialAccount, BlockHeader, PartialBlockchain), DataStoreError> {
        let (mut partial_account, block_header, partial_blockchain) =
            self.inner.get_transaction_inputs(account_id, ref_blocks).await?;

        if account_id == self.batch_account_id {
            partial_account = PartialAccount::from(&self.current_account);
        }

        Ok((partial_account, block_header, partial_blockchain))
    }

    async fn get_vault_asset_witnesses(
        &self,
        account_id: AccountId,
        vault_root: Word,
        vault_keys: BTreeSet<AssetVaultKey>,
    ) -> Result<Vec<AssetWitness>, DataStoreError> {
        if account_id == self.batch_account_id {
            // Serve witnesses directly from the in-batch account state, as inner store's
            // vault may be stale relative to updates made by previous pushes in this batch,
            // which would cause a vault root mismatch when the executor compares the root
            // exposed by the substituted PartialAccount against what the store returns.
            let vault = self.current_account.vault();
            let in_batch_root = vault.root();
            if in_batch_root != vault_root {
                return Err(DataStoreError::other(format!(
                    "vault root mismatch for account {account_id}: in-batch root = {in_batch_root:?}, requested root = {vault_root:?}",
                )));
            }
            let witnesses = vault_keys.into_iter().map(|key| vault.open(key)).collect();
            Ok(witnesses)
        } else {
            self.inner.get_vault_asset_witnesses(account_id, vault_root, vault_keys).await
        }
    }

    async fn get_storage_map_witness(
        &self,
        account_id: AccountId,
        map_root: Word,
        map_key: StorageMapKey,
    ) -> Result<StorageMapWitness, DataStoreError> {
        if account_id == self.batch_account_id {
            // Serve witnesses directly from the in-batch account state. If a previous push in
            // this batch mutated a storage map, the inner store's view of that map is stale
            // relative to `self.current_account`, so we must open against the in-batch state.
            for slot in self.current_account.storage().slots() {
                if let StorageSlotContent::Map(map) = slot.content()
                    && map.root() == map_root
                {
                    return Ok(map.open(&map_key));
                }
            }
            return Err(DataStoreError::other(format!(
                "storage map root not found in in-batch account state for account {account_id}: requested root = {map_root:?}",
            )));
        }
        self.inner.get_storage_map_witness(account_id, map_root, map_key).await
    }

    async fn get_foreign_account_inputs(
        &self,
        foreign_account_id: AccountId,
        ref_block: BlockNumber,
    ) -> Result<AccountInputs, DataStoreError> {
        self.inner.get_foreign_account_inputs(foreign_account_id, ref_block).await
    }

    fn get_note_script(
        &self,
        script_root: NoteScriptRoot,
    ) -> impl FutureMaybeSend<Result<Option<NoteScript>, DataStoreError>> {
        self.inner.get_note_script(script_root)
    }
}

// MAST FOREST STORE IMPL
// ================================================================================================

impl MastForestStore for InMemoryBatchDataStore {
    fn get(&self, procedure_hash: &Word) -> Option<Arc<MastForest>> {
        self.inner.get(procedure_hash)
    }
}
