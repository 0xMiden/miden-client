use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_protocol::account::{
    Account,
    AccountDelta,
    AccountId,
    PartialAccount,
    StorageMapKey,
    StorageMapWitness,
    StorageSlotContent,
};
use miden_protocol::asset::{AssetVaultKey, AssetWitness};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::errors::AccountDeltaError;
use miden_protocol::note::{NoteScript, NoteScriptRoot};
use miden_protocol::transaction::{AccountInputs, PartialBlockchain};
use miden_protocol::vm::FutureMaybeSend;
use miden_protocol::{MastForest, Word};
use miden_tx::{DataStore, DataStoreError, MastForestStore, TransactionMastStore};

use crate::store::data_store::ClientDataStore;

// IN-MEMORY BATCH DATA STORE
// ================================================================================================

/// A [`DataStore`] that lets a [`crate::transaction::BatchBuilder`] stack in-memory account
/// inputs for any number of local accounts. For each account pushed into the batch, an
/// accumulated [`AccountDelta`] is kept; the executor sees the in-batch account state (the
/// persisted account with that delta applied) instead of the stale store state. All other
/// reads pass through to the inner [`ClientDataStore`].
pub(crate) struct InMemoryBatchDataStore {
    inner: ClientDataStore,
    account_deltas: BTreeMap<AccountId, AccountDelta>,
}

impl InMemoryBatchDataStore {
    /// Wraps the provided [`ClientDataStore`] with an empty in-batch account cache.
    pub(crate) fn new(inner: ClientDataStore) -> Self {
        Self { inner, account_deltas: BTreeMap::new() }
    }

    /// Merges `delta` into the accumulated in-batch delta for `account_id`, so later
    /// transactions in the same batch targeting `account_id` observe its post-state.
    pub(crate) fn cache_account(
        &mut self,
        account_id: AccountId,
        delta: AccountDelta,
    ) -> Result<(), AccountDeltaError> {
        if let Some(existing) = self.account_deltas.get_mut(&account_id) {
            existing.merge(delta)?;
        } else {
            self.account_deltas.insert(account_id, delta);
        }
        Ok(())
    }

    /// Returns the inner [`ClientDataStore`]'s MAST store so callers can load account
    /// or note code prior to execution.
    pub(crate) fn mast_store(&self) -> Arc<TransactionMastStore> {
        self.inner.mast_store()
    }

    /// Registers foreign account inputs on the inner [`ClientDataStore`] so the executor
    /// can resolve foreign-procedure invocations during transaction execution.
    pub(crate) fn register_foreign_account_inputs(
        &self,
        foreign_accounts: impl IntoIterator<Item = AccountInputs>,
    ) {
        self.inner.register_foreign_account_inputs(foreign_accounts);
    }

    /// Registers note scripts on the inner [`ClientDataStore`] so the executor can resolve
    /// the request's output note scripts during transaction execution.
    pub(crate) fn register_note_scripts(&self, note_scripts: impl IntoIterator<Item = NoteScript>) {
        self.inner.register_note_scripts(note_scripts);
    }

    pub(crate) async fn current_account(
        &self,
        account_id: AccountId,
    ) -> Result<Account, DataStoreError> {
        let mut account = self.inner.load_account(account_id).await?;
        if let Some(delta) = self.account_deltas.get(&account_id) {
            account.apply_delta(delta).map_err(|err| {
                DataStoreError::other_with_source("failed to apply in-batch account delta", err)
            })?;
        }

        Ok(account)
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

        if self.account_deltas.contains_key(&account_id) {
            partial_account = PartialAccount::from(&self.current_account(account_id).await?);
        }

        Ok((partial_account, block_header, partial_blockchain))
    }

    async fn get_vault_asset_witnesses(
        &self,
        account_id: AccountId,
        vault_root: Word,
        vault_keys: BTreeSet<AssetVaultKey>,
    ) -> Result<Vec<AssetWitness>, DataStoreError> {
        if self.account_deltas.contains_key(&account_id) {
            let account = self.current_account(account_id).await?;
            let vault = account.vault();
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
        if self.account_deltas.contains_key(&account_id) {
            let account = self.current_account(account_id).await?;
            for slot in account.storage().slots() {
                if let StorageSlotContent::Map(map) = slot.content()
                    && map.root() == map_root
                {
                    return Ok(map.open(&map_key));
                }
            }
            Err(DataStoreError::other(format!(
                "storage map root not found in in-batch account state for account {account_id}: requested root = {map_root:?}",
            )))
        } else {
            self.inner.get_storage_map_witness(account_id, map_root, map_key).await
        }
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
