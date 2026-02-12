use std::collections::HashMap;

use miden_client::account::{AccountId, AccountStorage, StorageMap, StorageSlotContent};
use miden_client::asset::{Asset, AssetVault, AssetWitness};
use miden_client::crypto::SMT_DEPTH;
use miden_client::store::StoreError;
use miden_client::{EMPTY_WORD, Word};
use miden_protocol::account::StorageMapWitness;
use miden_protocol::asset::AssetVaultKey;
use miden_protocol::crypto::merkle::smt::{Smt, SmtForest};
use miden_protocol::crypto::merkle::{EmptySubtreeRoots, MerkleError};

/// Thin wrapper around `SmtForest` for account vault/storage proofs and updates.
///
/// Tracks current SMT roots per account with reference counting to safely pop
/// roots from the underlying forest when no account references them anymore.
/// Supports staged updates for transaction rollback via a pending roots stack.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct AccountSmtForest {
    forest: SmtForest,
    /// Current roots per account (vault root + storage map roots).
    account_roots: HashMap<AccountId, Vec<Word>>,
    /// Stack of old roots saved during staging, awaiting commit or undo.
    pending_old_roots: HashMap<AccountId, Vec<Vec<Word>>>,
    /// Reference count for each SMT root across all accounts.
    root_refcounts: HashMap<Word, usize>,
}

impl AccountSmtForest {
    pub fn new() -> Self {
        Self::default()
    }

    // READERS
    // --------------------------------------------------------------------------------------------

    /// Returns the current roots for an account.
    pub fn get_roots(&self, account_id: &AccountId) -> Option<&Vec<Word>> {
        self.account_roots.get(account_id)
    }

    /// Retrieves the vault asset and its witness for a specific vault key.
    pub fn get_asset_and_witness(
        &self,
        vault_root: Word,
        vault_key: AssetVaultKey,
    ) -> Result<(Asset, AssetWitness), StoreError> {
        let vault_key_word = vault_key.into();
        let proof = self.forest.open(vault_root, vault_key_word)?;
        let asset_word =
            proof.get(&vault_key_word).ok_or(MerkleError::UntrackedKey(vault_key_word))?;
        if asset_word == EMPTY_WORD {
            return Err(MerkleError::UntrackedKey(vault_key_word).into());
        }

        let asset = Asset::try_from(asset_word)?;
        let witness = AssetWitness::new(proof)?;
        Ok((asset, witness))
    }

    /// Retrieves the storage map witness for a specific map item.
    pub fn get_storage_map_item_witness(
        &self,
        map_root: Word,
        key: Word,
    ) -> Result<StorageMapWitness, StoreError> {
        let hashed_key = StorageMap::hash_key(key);
        let proof = self.forest.open(map_root, hashed_key).map_err(StoreError::from)?;
        Ok(StorageMapWitness::new(proof, [key])?)
    }

    // ROOT LIFECYCLE
    // --------------------------------------------------------------------------------------------

    /// Stages new roots for an account, saving old roots for potential rollback.
    ///
    /// The old roots are pushed onto a pending stack and their refcounts are preserved.
    /// Call [`Self::commit_roots`] to release old roots or [`Self::discard_roots`] to
    /// restore them.
    pub fn stage_roots(&mut self, account_id: AccountId, new_roots: Vec<Word>) {
        increment_refcounts(&mut self.root_refcounts, &new_roots);
        if let Some(old_roots) = self.account_roots.insert(account_id, new_roots) {
            self.pending_old_roots.entry(account_id).or_default().push(old_roots);
        }
    }

    /// Commits staged changes: releases all pending old roots for the account.
    pub fn commit_roots(&mut self, account_id: AccountId) {
        if let Some(old_roots_stack) = self.pending_old_roots.remove(&account_id) {
            for old_roots in old_roots_stack {
                let to_pop = decrement_refcounts(&mut self.root_refcounts, &old_roots);
                self.forest.pop_smts(to_pop);
            }
        }
    }

    /// Discards the most recent staged change: restores old roots and releases new roots.
    ///
    /// If there are old roots to restore, the current roots are replaced with them.
    /// If there are no old roots (i.e., the account was first staged without prior state),
    /// the current roots are simply removed.
    pub fn discard_roots(&mut self, account_id: AccountId) {
        let old_roots = self.pending_old_roots.get_mut(&account_id).and_then(Vec::pop);

        // Release the current (staged) roots and restore old ones if available
        if let Some(old_roots) = old_roots {
            if let Some(new_roots) = self.account_roots.insert(account_id, old_roots) {
                let to_pop = decrement_refcounts(&mut self.root_refcounts, &new_roots);
                self.forest.pop_smts(to_pop);
            }
        } else if let Some(new_roots) = self.account_roots.remove(&account_id) {
            // No old roots to restore — just remove the staged roots entirely
            let to_pop = decrement_refcounts(&mut self.root_refcounts, &new_roots);
            self.forest.pop_smts(to_pop);
        }

        // Clean up empty stack
        if self.pending_old_roots.get(&account_id).is_some_and(Vec::is_empty) {
            self.pending_old_roots.remove(&account_id);
        }
    }

    /// Replaces roots atomically: sets new roots and immediately releases old roots.
    ///
    /// Use this when no rollback is needed (e.g., initial insert, network updates).
    ///
    /// # Panics
    ///
    /// Panics if there are pending staged changes for the account. Use
    /// [`Self::commit_roots`] or [`Self::discard_roots`] first.
    pub fn replace_roots(&mut self, account_id: AccountId, new_roots: Vec<Word>) {
        assert!(
            !self.pending_old_roots.contains_key(&account_id),
            "cannot replace roots while staged changes are pending for account {account_id}"
        );
        increment_refcounts(&mut self.root_refcounts, &new_roots);
        if let Some(old_roots) = self.account_roots.insert(account_id, new_roots) {
            let to_pop = decrement_refcounts(&mut self.root_refcounts, &old_roots);
            self.forest.pop_smts(to_pop);
        }
    }

    // TREE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Updates the SMT forest with the new asset values.
    pub fn update_asset_nodes(
        &mut self,
        root: Word,
        new_assets: impl Iterator<Item = Asset>,
        removed_vault_keys: impl Iterator<Item = AssetVaultKey>,
    ) -> Result<Word, StoreError> {
        let entries: Vec<(Word, Word)> = new_assets
            .map(|asset| {
                let key: Word = asset.vault_key().into();
                let value: Word = asset.into();
                (key, value)
            })
            .chain(removed_vault_keys.map(|key| (key.into(), EMPTY_WORD)))
            .collect();

        if entries.is_empty() {
            return Ok(root);
        }

        let new_root = self.forest.batch_insert(root, entries).map_err(StoreError::from)?;
        Ok(new_root)
    }

    /// Updates the SMT forest with the new storage map values.
    pub fn update_storage_map_nodes(
        &mut self,
        root: Word,
        entries: impl Iterator<Item = (Word, Word)>,
    ) -> Result<Word, StoreError> {
        let entries: Vec<(Word, Word)> =
            entries.map(|(key, value)| (StorageMap::hash_key(key), value)).collect();

        if entries.is_empty() {
            return Ok(root);
        }

        let new_root = self.forest.batch_insert(root, entries).map_err(StoreError::from)?;
        Ok(new_root)
    }

    /// Inserts the asset vault SMT nodes to the SMT forest.
    pub fn insert_asset_nodes(&mut self, vault: &AssetVault) -> Result<(), StoreError> {
        let smt = Smt::with_entries(vault.assets().map(|asset| {
            let key: Word = asset.vault_key().into();
            let value: Word = asset.into();
            (key, value)
        }))
        .map_err(StoreError::from)?;

        let empty_root = *EmptySubtreeRoots::entry(SMT_DEPTH, 0);
        let entries: Vec<(Word, Word)> = smt.entries().map(|(k, v)| (*k, *v)).collect();
        let new_root = self.forest.batch_insert(empty_root, entries).map_err(StoreError::from)?;
        debug_assert_eq!(new_root, smt.root());
        Ok(())
    }

    /// Inserts all storage map SMT nodes to the SMT forest.
    pub fn insert_storage_map_nodes(&mut self, storage: &AccountStorage) -> Result<(), StoreError> {
        let maps = storage.slots().iter().filter_map(|slot| match slot.content() {
            StorageSlotContent::Map(map) => Some(map),
            StorageSlotContent::Value(_) => None,
        });

        for map in maps {
            self.insert_storage_map_nodes_for_map(map)?;
        }
        Ok(())
    }

    /// Inserts the SMT nodes for an account's vault and storage maps into the
    /// forest, without tracking roots for the account.
    pub fn insert_account_state(
        &mut self,
        vault: &AssetVault,
        storage: &AccountStorage,
    ) -> Result<(), StoreError> {
        self.insert_storage_map_nodes(storage)?;
        self.insert_asset_nodes(vault)?;
        Ok(())
    }

    /// Inserts all SMT nodes for an account's vault and storage, then replaces
    /// the account's tracked roots atomically.
    pub fn insert_and_register_account_state(
        &mut self,
        account_id: AccountId,
        vault: &AssetVault,
        storage: &AccountStorage,
    ) -> Result<(), StoreError> {
        self.insert_account_state(vault, storage)?;
        let roots = Self::collect_account_roots(vault, storage);
        self.replace_roots(account_id, roots);
        Ok(())
    }

    pub fn insert_storage_map_nodes_for_map(&mut self, map: &StorageMap) -> Result<(), StoreError> {
        let empty_root = *EmptySubtreeRoots::entry(SMT_DEPTH, 0);
        let entries: Vec<(Word, Word)> =
            map.entries().map(|(k, v)| (StorageMap::hash_key(*k), *v)).collect();
        self.forest.batch_insert(empty_root, entries).map_err(StoreError::from)?;
        Ok(())
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

    /// Collects all SMT roots (vault root + storage map roots) for an account's state.
    fn collect_account_roots(vault: &AssetVault, storage: &AccountStorage) -> Vec<Word> {
        let mut roots = vec![vault.root()];
        for slot in storage.slots() {
            if let StorageSlotContent::Map(map) = slot.content() {
                roots.push(map.root());
            }
        }
        roots
    }
}

fn increment_refcounts(refcounts: &mut HashMap<Word, usize>, roots: &[Word]) {
    for root in roots {
        *refcounts.entry(*root).or_insert(0) += 1;
    }
}

/// Decrements refcounts for the given roots, returning those that reached zero.
fn decrement_refcounts(refcounts: &mut HashMap<Word, usize>, roots: &[Word]) -> Vec<Word> {
    let mut to_pop = Vec::new();
    for root in roots {
        if let Some(count) = refcounts.get_mut(root) {
            *count -= 1;
            if *count == 0 {
                refcounts.remove(root);
                to_pop.push(*root);
            }
        }
    }
    to_pop
}

#[cfg(test)]
mod tests {
    use miden_client::{ONE, ZERO};
    use miden_protocol::testing::account_id::{
        ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
        ACCOUNT_ID_PUBLIC_NON_FUNGIBLE_FAUCET,
    };

    use super::*;

    fn account_a() -> AccountId {
        AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET).unwrap()
    }

    fn account_b() -> AccountId {
        AccountId::try_from(ACCOUNT_ID_PUBLIC_NON_FUNGIBLE_FAUCET).unwrap()
    }

    /// Creates a StorageMap with a single entry and inserts its nodes into the forest.
    /// Returns the map's root.
    fn insert_map(forest: &mut AccountSmtForest, key: Word, value: Word) -> Word {
        let mut map = StorageMap::new();
        map.insert(key, value).unwrap();
        forest.insert_storage_map_nodes_for_map(&map).unwrap();
        map.root()
    }

    /// Returns true if the forest can still serve a proof for the given root.
    fn root_is_live(forest: &AccountSmtForest, root: Word, key: Word) -> bool {
        forest.get_storage_map_item_witness(root, key).is_ok()
    }

    #[test]
    fn stage_then_commit_releases_old_roots() {
        let mut forest = AccountSmtForest::new();
        let id = account_a();

        let key1: Word = [ONE, ZERO, ZERO, ZERO].into();
        let key2: Word = [ZERO, ONE, ZERO, ZERO].into();
        let val: Word = [ONE, ONE, ONE, ONE].into();

        let root1 = insert_map(&mut forest, key1, val);
        let root2 = insert_map(&mut forest, key2, val);

        // Initial state
        forest.replace_roots(id, vec![root1]);
        assert_eq!(forest.get_roots(&id), Some(&vec![root1]));

        // Stage new roots (apply_delta)
        forest.stage_roots(id, vec![root2]);
        assert_eq!(forest.get_roots(&id), Some(&vec![root2]));

        // Both roots alive during staging (old preserved for rollback)
        assert!(root_is_live(&forest, root1, key1));
        assert!(root_is_live(&forest, root2, key2));

        // Commit — old roots released
        forest.commit_roots(id);
        assert_eq!(forest.get_roots(&id), Some(&vec![root2]));
        assert!(!root_is_live(&forest, root1, key1));
        assert!(root_is_live(&forest, root2, key2));
    }

    #[test]
    fn stage_then_discard_restores_old_roots() {
        let mut forest = AccountSmtForest::new();
        let id = account_a();

        let key1: Word = [ONE, ZERO, ZERO, ZERO].into();
        let key2: Word = [ZERO, ONE, ZERO, ZERO].into();
        let val: Word = [ONE, ONE, ONE, ONE].into();

        let root1 = insert_map(&mut forest, key1, val);
        let root2 = insert_map(&mut forest, key2, val);

        forest.replace_roots(id, vec![root1]);

        // Stage and discard (rollback)
        forest.stage_roots(id, vec![root2]);
        forest.discard_roots(id);

        assert_eq!(forest.get_roots(&id), Some(&vec![root1]));
        assert!(root_is_live(&forest, root1, key1));
        assert!(!root_is_live(&forest, root2, key2));
    }

    #[test]
    fn shared_root_survives_single_account_replacement() {
        let mut forest = AccountSmtForest::new();
        let id1 = account_a();
        let id2 = account_b();

        let key: Word = [ONE, ZERO, ZERO, ZERO].into();
        let val: Word = [ONE, ONE, ONE, ONE].into();
        let shared_root = insert_map(&mut forest, key, val);

        // Both accounts reference the same root
        forest.replace_roots(id1, vec![shared_root]);
        forest.replace_roots(id2, vec![shared_root]);

        // Replace id1 with a different root
        let key2: Word = [ZERO, ONE, ZERO, ZERO].into();
        let other_root = insert_map(&mut forest, key2, val);
        forest.replace_roots(id1, vec![other_root]);

        // Shared root still alive (id2 still references it)
        assert!(root_is_live(&forest, shared_root, key));

        // Replace id2 too — now shared root should be popped
        forest.replace_roots(id2, vec![other_root]);
        assert!(!root_is_live(&forest, shared_root, key));
    }

    #[test]
    fn multiple_stages_discard_one_at_a_time() {
        let mut forest = AccountSmtForest::new();
        let id = account_a();

        let key_a: Word = [ONE, ZERO, ZERO, ZERO].into();
        let key_b: Word = [ZERO, ONE, ZERO, ZERO].into();
        let key_c: Word = [ZERO, ZERO, ONE, ZERO].into();
        let val: Word = [ONE, ONE, ONE, ONE].into();

        let root_a = insert_map(&mut forest, key_a, val);
        let root_b = insert_map(&mut forest, key_b, val);
        let root_c = insert_map(&mut forest, key_c, val);

        // A -> B -> C
        forest.replace_roots(id, vec![root_a]);
        forest.stage_roots(id, vec![root_b]);
        forest.stage_roots(id, vec![root_c]);
        assert_eq!(forest.get_roots(&id), Some(&vec![root_c]));

        // Discard C -> back to B
        forest.discard_roots(id);
        assert_eq!(forest.get_roots(&id), Some(&vec![root_b]));
        assert!(!root_is_live(&forest, root_c, key_c));
        assert!(root_is_live(&forest, root_b, key_b));
        assert!(root_is_live(&forest, root_a, key_a));

        // Discard B -> back to A
        forest.discard_roots(id);
        assert_eq!(forest.get_roots(&id), Some(&vec![root_a]));
        assert!(!root_is_live(&forest, root_b, key_b));
        assert!(root_is_live(&forest, root_a, key_a));
    }

    #[test]
    fn multiple_stages_commit_releases_all_old() {
        let mut forest = AccountSmtForest::new();
        let id = account_a();

        let key_a: Word = [ONE, ZERO, ZERO, ZERO].into();
        let key_b: Word = [ZERO, ONE, ZERO, ZERO].into();
        let key_c: Word = [ZERO, ZERO, ONE, ZERO].into();
        let val: Word = [ONE, ONE, ONE, ONE].into();

        let root_a = insert_map(&mut forest, key_a, val);
        let root_b = insert_map(&mut forest, key_b, val);
        let root_c = insert_map(&mut forest, key_c, val);

        // A -> B -> C, then commit
        forest.replace_roots(id, vec![root_a]);
        forest.stage_roots(id, vec![root_b]);
        forest.stage_roots(id, vec![root_c]);
        forest.commit_roots(id);

        // Only C survives
        assert_eq!(forest.get_roots(&id), Some(&vec![root_c]));
        assert!(!root_is_live(&forest, root_a, key_a));
        assert!(!root_is_live(&forest, root_b, key_b));
        assert!(root_is_live(&forest, root_c, key_c));
    }

    #[test]
    fn unchanged_root_survives_stage_commit() {
        let mut forest = AccountSmtForest::new();
        let id = account_a();

        let key1: Word = [ONE, ZERO, ZERO, ZERO].into();
        let key2: Word = [ZERO, ONE, ZERO, ZERO].into();
        let val: Word = [ONE, ONE, ONE, ONE].into();

        let shared_root = insert_map(&mut forest, key1, val);
        let changing_root = insert_map(&mut forest, key2, val);

        // Initial: [shared, changing]
        forest.replace_roots(id, vec![shared_root, changing_root]);

        // Delta only changes the second root; shared_root stays
        let key3: Word = [ZERO, ZERO, ONE, ZERO].into();
        let new_root = insert_map(&mut forest, key3, val);
        forest.stage_roots(id, vec![shared_root, new_root]);
        forest.commit_roots(id);

        // shared_root must survive (it's in both old and new)
        assert!(root_is_live(&forest, shared_root, key1));
        // changing_root should be popped
        assert!(!root_is_live(&forest, changing_root, key2));
        // new_root should be alive
        assert!(root_is_live(&forest, new_root, key3));
    }
}
