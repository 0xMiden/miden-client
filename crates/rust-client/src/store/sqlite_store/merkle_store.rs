use miden_objects::Word;
use miden_objects::asset::{Asset, AssetVault};
use miden_objects::crypto::merkle::{MerklePath, MerkleStore, NodeIndex, SMT_DEPTH, SmtLeaf};

use crate::store::StoreError;

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

pub fn update_asset_node(
    merkle_store: &mut MerkleStore,
    old_root: Word,
    asset: &Asset,
) -> Result<Word, StoreError> {
    let new_root = merkle_store.set_node(
        old_root,
        NodeIndex::new(SMT_DEPTH, asset.vault_key()[3].as_int())?, /* Is this conversion exposed
                                                                    * in any way? */
        SmtLeaf::Single((asset.vault_key(), asset.into())).hash(), /* Is this conversion exposed
                                                                    * in any way? */
    )?;

    Ok(new_root.root)
}

pub fn insert_asset_paths(
    merkle_store: &mut MerkleStore,
    vault: &AssetVault,
) -> Result<(), StoreError> {
    for asset in vault.assets() {
        let path = vault.asset_tree().open(&asset.vault_key()).into_parts().0;
        merkle_store.add_merkle_path(
            asset.vault_key()[3].as_int(),
            SmtLeaf::Single((asset.vault_key(), asset.into())).hash(),
            path,
        )?;
    }

    Ok(())
}
