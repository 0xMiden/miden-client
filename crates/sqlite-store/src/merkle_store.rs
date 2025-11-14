use miden_client::Word;
use miden_client::account::{AccountStorage, StorageMap, StorageSlot};
use miden_client::asset::{Asset, AssetVault};
use miden_client::crypto::{MerklePath, NodeIndex, SMT_DEPTH, SmtLeaf, SmtProof};
use miden_client::store::StoreError;
use miden_objects::asset::AssetVaultKey;
use miden_objects::crypto::merkle::{EmptySubtreeRoots, Smt, SmtForest};

/// Retrieves the Merkle proof for a specific asset in the merkle store.
pub fn get_asset_proof(
    smt_forest: &SmtForest,
    vault_root: Word,
    asset: &Asset,
) -> Result<SmtProof, StoreError> {
    let path = smt_forest.open(vault_root, asset.vault_key().into())?.path().clone();
    let vault_key: Word = asset.vault_key().into();
    let leaf = SmtLeaf::new_single(vault_key, (*asset).into());

    Ok(SmtProof::new(path, leaf)?)
}

/// Updates the merkle store with the new asset values.
pub fn update_asset_nodes(
    smt_forest: &mut SmtForest,
    root: Word,
    assets: impl Iterator<Item = Asset>,
) -> Result<Word, StoreError> {
    let entries: Vec<(Word, Word)> = assets
        .map(|asset| {
            let key: Word = asset.vault_key().into();
            let value: Word = asset.into();
            (key, value)
        })
        .collect();

    let empty_root = *EmptySubtreeRoots::entry(SMT_DEPTH, 0);
    smt_forest.batch_insert(empty_root, entries).map_err(StoreError::from)
}

/// Inserts the asset vault SMT nodes to the merkle store.
pub fn insert_asset_nodes(smt_forest: &mut SmtForest, vault: &AssetVault) {
    // We need to build the SMT from the vault iterable entries as
    // we don't have direct access to the vault's SMT nodes.
    // Safe unwrap as we are sure that the vault's SMT nodes are valid.
    let smt =
        Smt::with_entries(vault.assets().map(|asset| (asset.vault_key().into(), asset.into())))
            .unwrap();

    let empty_root = *EmptySubtreeRoots::entry(SMT_DEPTH, 0);
    let entries: Vec<(Word, Word)> = smt.entries().map(|(k, v)| (*k, *v)).collect();
    let new_root = smt_forest.batch_insert(empty_root, entries).unwrap(); // TODO: handle unwrap
    debug_assert_eq!(new_root, smt.root());
}

/// Retrieves the Merkle proof for a specific storage map item in the merkle store.
pub fn get_storage_map_item_proof(
    smt_forest: &SmtForest,
    map_root: Word,
    key: Word,
) -> Result<(Word, MerklePath), StoreError> {
    let hashed_key = AssetVaultKey::new_unchecked(StorageMap::hash_key(key));
    let proof = smt_forest.open(map_root, hashed_key.into()).map_err(StoreError::from)?;
    Ok((proof.compute_root(), proof.path().clone().into()))
}

pub fn update_storage_map_nodes(
    smt_forest: &mut SmtForest,
    root: Word,
    entries: impl Iterator<Item = (Word, Word)> + Clone,
) -> Result<Word, StoreError> {
    let empty_root = *EmptySubtreeRoots::entry(SMT_DEPTH, 0);
    let new_root = smt_forest.batch_insert(empty_root, entries).map_err(StoreError::from)?;
    // Resulting root should match the map's root
    debug_assert_eq!(new_root, root);
    Ok(new_root)
}

/// Inserts all storage map SMT nodes to the merkle store.
pub fn insert_storage_map_nodes(smt_forest: &mut SmtForest, storage: &AccountStorage) {
    let maps = storage.slots().iter().filter_map(|slot| {
        if let StorageSlot::Map(map) = slot {
            Some(map)
        } else {
            None
        }
    });

    for map in maps {
        let empty_root = *EmptySubtreeRoots::entry(SMT_DEPTH, 0);
        let entries: Vec<(Word, Word)> = map.entries().map(|(k, v)| (*k, *v)).collect();
        let new_root = smt_forest.batch_insert(empty_root, entries).unwrap(); // TODO: handle unwrap
        // Resulting root should match the map's root
        debug_assert_eq!(new_root, map.root());
    }
}

// HELPERS
// ================================================================================================

/// Builds the merkle node index for the given key.
///
/// This logic is based on the way [`miden_objects::crypto::merkle::Smt`] is structured internally.
/// It has a set depth and uses the third felt as the position. The reason we want to copy the smt's
/// internal structure is so that merkle paths and roots match. For more information, see the
/// [`miden_objects::crypto::merkle::Smt`] documentation and implementation.
fn get_node_index(key: AssetVaultKey) -> Result<NodeIndex, StoreError> {
    let vault_key_word: Word = key.into();
    Ok(NodeIndex::new(SMT_DEPTH, vault_key_word[3].as_int())?)
}

/// Builds the merkle node value for the given key and value.
///
/// This logic is based on the way [`miden_objects::crypto::merkle::Smt`] generates the values for
/// its internal merkle tree. It generates an [`SmtLeaf`] from the key and value, and then hashes it
/// to produce the node value.
fn get_node_value(key: AssetVaultKey, value: Word) -> Word {
    let vault_key_word: Word = key.into();
    SmtLeaf::Single((vault_key_word, value)).hash()
}
