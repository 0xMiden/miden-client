use miden_objects::Word;
use miden_objects::account::{AccountStorage, StorageMap, StorageSlot};
use miden_objects::asset::{Asset, AssetVault};
use miden_objects::crypto::merkle::{MerklePath, MerkleStore, NodeIndex, SMT_DEPTH, SmtLeaf};

use crate::store::StoreError;

/// Retrieves the Merkle proof for a specific asset in the merkle store.
pub fn get_asset_proof(
    merkle_store: &MerkleStore,
    vault_root: Word,
    asset: &Asset,
) -> Result<MerklePath, StoreError> {
    Ok(merkle_store
        .get_path(
            vault_root,
            NodeIndex::new(
                miden_objects::crypto::merkle::SMT_DEPTH,
                asset.vault_key()[3].as_int(),
            )?, // Is this conversion exposed in any way?
        )?
        .path)
}

/// Updates the merkle store with the new asset values.
pub fn update_asset_nodes(
    merkle_store: &mut MerkleStore,
    mut root: Word,
    assets: impl Iterator<Item = Asset>,
) -> Result<Word, StoreError> {
    for asset in assets {
        root =
            merkle_store
                .set_node(
                    root,
                    NodeIndex::new(SMT_DEPTH, asset.vault_key()[3].as_int())?, /* Is this conversion exposed
                                                                                * in any way? */
                    SmtLeaf::Single((asset.vault_key(), asset.into())).hash(), /* Is this conversion exposed
                                                                                * in any way? */
                )?
                .root;
    }

    Ok(root)
}

/// Inserts the asset vault SMT nodes to the merkle store.
pub fn insert_asset_nodes(merkle_store: &mut MerkleStore, vault: &AssetVault) {
    merkle_store.extend(vault.asset_tree().inner_nodes());
}

/// Retrieves the Merkle proof for a specific storage map item in the merkle store.
pub fn get_storage_map_item_proof(
    merkle_store: &MerkleStore,
    map_root: Word,
    key: Word,
) -> Result<MerklePath, StoreError> {
    let hashed_key = StorageMap::hash_key(key);
    Ok(merkle_store
        .get_path(
            map_root,
            NodeIndex::new(miden_objects::crypto::merkle::SMT_DEPTH, hashed_key[3].as_int())?, // Is this conversion exposed in any way?
        )?
        .path)
}

/// Updates the merkle store with the new storage map entries.
pub fn update_storage_map_nodes(
    merkle_store: &mut MerkleStore,
    mut root: Word,
    entries: impl Iterator<Item = (Word, Word)>,
) -> Result<Word, StoreError> {
    for (key, value) in entries {
        let hashed_key = StorageMap::hash_key(key);
        root = merkle_store
            .set_node(
                root,
                NodeIndex::new(SMT_DEPTH, hashed_key[3].as_int())?, /* Is this conversion
                                                                     * exposed
                                                                     * in any way? */
                SmtLeaf::Single((hashed_key, value)).hash(), /* Is this conversion exposed
                                                              * in any way? */
            )?
            .root;
    }

    Ok(root)
}

/// Inserts all storage map SMT nodes to the merkle store.
pub fn insert_storage_map_nodes(merkle_store: &mut MerkleStore, storage: &AccountStorage) {
    let maps = storage.slots().iter().filter_map(|slot| {
        if let StorageSlot::Map(map) = slot {
            Some(map)
        } else {
            None
        }
    });

    for map in maps {
        merkle_store.extend(map.inner_nodes());
    }
}
