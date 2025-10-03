use miden_client::Word;
use miden_client::account::{AccountStorage, StorageMap, StorageSlot};
use miden_client::asset::{Asset, AssetVault};
use miden_client::crypto::{MerklePath, MerkleStore, NodeIndex, SMT_DEPTH, SmtLeaf, SmtProof};
use miden_client::store::StoreError;
use miden_objects::crypto::merkle::Smt;

/// Retrieves the Merkle proof for a specific asset in the merkle store.
pub fn get_asset_proof(
    merkle_store: &MerkleStore,
    vault_root: Word,
    asset: &Asset,
) -> Result<SmtProof, StoreError> {
    let path = merkle_store.get_path(vault_root, get_node_index(asset.vault_key())?)?.path;
    let leaf = SmtLeaf::new_single(asset.vault_key(), (*asset).into());

    Ok(SmtProof::new(path, leaf)?)
}

/// Updates the merkle store with the new asset values.
pub fn update_asset_nodes(
    merkle_store: &mut MerkleStore,
    mut root: Word,
    assets: impl Iterator<Item = Asset>,
) -> Result<Word, StoreError> {
    for asset in assets {
        root = merkle_store
            .set_node(
                root,
                get_node_index(asset.vault_key())?,
                get_node_value(asset.vault_key(), asset.into()),
            )?
            .root;
    }

    Ok(root)
}

/// Inserts the asset vault SMT nodes to the merkle store.
pub fn insert_asset_nodes(merkle_store: &mut MerkleStore, vault: &AssetVault) {
    // We need to build the SMT from the vault iterable entries as
    // we don't have direct access to the vault's SMT nodes.
    // Safe unwrap as we are sure that the vault's SMT nodes are valid.
    let smt =
        Smt::with_entries(vault.assets().map(|asset| (asset.vault_key(), asset.into()))).unwrap();
    merkle_store.extend(smt.inner_nodes());
}

/// Retrieves the Merkle proof for a specific storage map item in the merkle store.
pub fn get_storage_map_item_proof(
    merkle_store: &MerkleStore,
    map_root: Word,
    key: Word,
) -> Result<(Word, MerklePath), StoreError> {
    let hashed_key = StorageMap::hash_key(key);
    let vp = merkle_store.get_path(map_root, get_node_index(hashed_key)?)?;
    Ok((vp.value, vp.path))
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
            .set_node(root, get_node_index(hashed_key)?, get_node_value(hashed_key, value))?
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

// HELPERS
// ================================================================================================

/// Builds the merkle node index for the given key.
///
/// This logic is based on the way [`miden_objects::crypto::merkle::Smt`] is structured internally.
/// It has a set depth and uses the third felt as the position. The reason we want to copy the smt's
/// internal structure is so that merkle paths and roots match. For more information, see the
/// [`miden_objects::crypto::merkle::Smt`] documentation and implementation.
fn get_node_index(key: Word) -> Result<NodeIndex, StoreError> {
    Ok(NodeIndex::new(SMT_DEPTH, key[3].as_int())?)
}

/// Builds the merkle node value for the given key and value.
///
/// This logic is based on the way [`miden_objects::crypto::merkle::Smt`] generates the values for
/// its internal merkle tree. It generates an [`SmtLeaf`] from the key and value, and then hashes it
/// to produce the node value.
fn get_node_value(key: Word, value: Word) -> Word {
    SmtLeaf::Single((key, value)).hash()
}
