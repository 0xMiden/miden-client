use miden_client::account::{AccountStorage, StorageMap, StorageSlotContent};
use miden_client::asset::{Asset, AssetVault, AssetWitness};
use miden_client::crypto::SMT_DEPTH;
use miden_client::store::StoreError;
use miden_client::{EMPTY_WORD, Word};
use miden_protocol::account::StorageMapWitness;
use miden_protocol::crypto::merkle::smt::{Smt, SmtForest};
use miden_protocol::crypto::merkle::{EmptySubtreeRoots, MerkleError};

/// Thin wrapper around `SmtForest` for account vault/storage proofs and updates.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct AccountSmtForest {
    forest: SmtForest,
}

impl AccountSmtForest {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves the vault asset and its witness for a specific vault key.
    pub fn get_asset_and_witness(
        &self,
        vault_root: Word,
        vault_key: Word,
    ) -> Result<(Asset, AssetWitness), StoreError> {
        let proof = self.forest.open(vault_root, vault_key)?;
        let asset_word = proof.get(&vault_key).ok_or(MerkleError::UntrackedKey(vault_key))?;
        if asset_word == EMPTY_WORD {
            return Err(MerkleError::UntrackedKey(vault_key).into());
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

    // MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Updates the SMT forest with the new asset values.
    pub fn update_asset_nodes(
        &mut self,
        root: Word,
        assets: impl Iterator<Item = Asset>,
        removed_vault_keys: impl Iterator<Item = Word>,
    ) -> Result<Word, StoreError> {
        let entries: Vec<(Word, Word)> = assets
            .map(|asset| {
                let key: Word = asset.vault_key().into();
                let value: Word = asset.into();
                (key, value)
            })
            .chain(removed_vault_keys.map(|key| (key, EMPTY_WORD)))
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
    pub fn insert_asset_nodes(&mut self, vault: &AssetVault) {
        // We need to build the SMT from the vault iterable entries as we don't have direct access
        // to the vault's SMT nodes.
        // SAFETY: Safe to unwrap as we are sure that the vault's SMT nodes are valid.
        let smt =
            Smt::with_entries(vault.assets().map(|asset| (asset.vault_key().into(), asset.into())))
                .unwrap();

        let empty_root = *EmptySubtreeRoots::entry(SMT_DEPTH, 0);
        let entries: Vec<(Word, Word)> = smt.entries().map(|(k, v)| (*k, *v)).collect();
        let new_root = self.forest.batch_insert(empty_root, entries).unwrap(); // TODO: handle unwrap
        debug_assert_eq!(new_root, smt.root());
    }

    /// Inserts all storage map SMT nodes to the SMT forest.
    pub fn insert_storage_map_nodes(&mut self, storage: &AccountStorage) {
        let maps = storage.slots().iter().filter_map(|slot| match slot.content() {
            StorageSlotContent::Map(map) => Some(map),
            StorageSlotContent::Value(_) => None,
        });

        for map in maps {
            self.insert_storage_map_nodes_for_map(map);
        }
    }

    pub fn insert_account_state(&mut self, vault: &AssetVault, storage: &AccountStorage) {
        self.insert_storage_map_nodes(storage);
        self.insert_asset_nodes(vault);
    }

    pub fn insert_storage_map_nodes_for_map(&mut self, map: &StorageMap) {
        let empty_root = *EmptySubtreeRoots::entry(SMT_DEPTH, 0);
        let entries: Vec<(Word, Word)> =
            map.entries().map(|(k, v)| (StorageMap::hash_key(*k), *v)).collect();
        self.forest.batch_insert(empty_root, entries).unwrap(); // TODO: handle unwrap
    }
}
