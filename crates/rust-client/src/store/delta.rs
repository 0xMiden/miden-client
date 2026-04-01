use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::delta::NonFungibleDeltaAction;
use miden_protocol::account::{
    AccountDelta,
    AccountId,
    AccountIdPrefix,
    StorageMap,
    StorageSlotName,
    StorageSlotType,
};
use miden_protocol::asset::{Asset, AssetVaultKey, FungibleAsset};
use miden_protocol::crypto::merkle::MerkleError;

use super::StoreError;
use super::smt_forest::AccountSmtForest;

/// Computes updated storage slot roots from the delta using the SMT forest.
///
/// Value slots are taken directly from the delta. Map slots are computed incrementally
/// by applying the map delta entries to the old root via the SMT forest.
pub fn compute_storage_delta(
    smt_forest: &mut AccountSmtForest,
    old_map_roots: &BTreeMap<StorageSlotName, Word>,
    delta: &AccountDelta,
) -> Result<BTreeMap<StorageSlotName, (Word, StorageSlotType)>, StoreError> {
    let mut updated_slots: BTreeMap<StorageSlotName, (Word, StorageSlotType)> = delta
        .storage()
        .values()
        .map(|(slot_name, value)| (slot_name.clone(), (*value, StorageSlotType::Value)))
        .collect();

    let default_map_root = StorageMap::default().root();

    for (slot_name, map_delta) in delta.storage().maps() {
        let old_root = old_map_roots.get(slot_name).copied().unwrap_or(default_map_root);
        let new_root = smt_forest.update_storage_map_nodes(
            old_root,
            map_delta.entries().iter().map(|(key, value)| (*key.inner(), *value)),
        )?;
        updated_slots.insert(slot_name.clone(), (new_root, StorageSlotType::Map));
    }

    Ok(updated_slots)
}

/// Computes the new vault state from old assets and the vault delta.
///
/// Returns (`updated_assets`, `removed_vault_keys`) where:
/// - `updated_assets` contains assets with their new values (for DB insertion and SMT update)
/// - `removed_vault_keys` contains vault keys for assets removed from the vault
pub fn compute_vault_delta(
    old_vault_assets: &[Asset],
    delta: &AccountDelta,
) -> Result<(Vec<Asset>, Vec<AssetVaultKey>), StoreError> {
    let mut updated_assets = Vec::new();
    let mut removed_vault_keys = Vec::new();

    // Build lookup map from faucet ID prefix to FungibleAsset
    let mut fungible_map: BTreeMap<AccountIdPrefix, FungibleAsset> = old_vault_assets
        .iter()
        .filter_map(|asset| match asset {
            Asset::Fungible(fa) => Some((fa.faucet_id_prefix(), *fa)),
            Asset::NonFungible(_) => None,
        })
        .collect();

    // Process fungible deltas
    for (faucet_id, delta_amount) in delta.vault().fungible().iter() {
        let delta_asset = FungibleAsset::new(*faucet_id, delta_amount.unsigned_abs())?;

        let asset = match fungible_map.remove(&faucet_id.prefix()) {
            Some(existing) => {
                if *delta_amount >= 0 {
                    existing.add(delta_asset)?
                } else {
                    existing.sub(delta_asset)?
                }
            },
            None => delta_asset,
        };

        if asset.amount() > 0 {
            updated_assets.push(Asset::Fungible(asset));
        } else {
            removed_vault_keys.push(asset.vault_key());
        }
    }

    // Process non-fungible deltas
    for (nft, action) in delta.vault().non_fungible().iter() {
        match action {
            NonFungibleDeltaAction::Add => {
                updated_assets.push(Asset::NonFungible(*nft));
            },
            NonFungibleDeltaAction::Remove => {
                removed_vault_keys.push(nft.vault_key());
            },
        }
    }

    Ok((updated_assets, removed_vault_keys))
}

/// Result of applying an account delta to the SMT forest.
///
/// Contains the computed values that the store needs to persist to its backend.
pub struct AppliedAccountDelta {
    /// Updated storage slot roots (value slots and map slots with new roots).
    pub updated_storage_slots: BTreeMap<StorageSlotName, (Word, StorageSlotType)>,
    /// Assets with updated values (new or changed).
    pub updated_assets: Vec<Asset>,
    /// Vault keys for assets removed from the vault.
    pub removed_vault_keys: Vec<AssetVaultKey>,
}

/// Applies an account delta to the SMT forest: computes storage and vault changes,
/// updates the forest, verifies the vault root, and stages roots for later commit/discard.
///
/// Returns the computed values so the store can persist them to its backend.
pub fn apply_account_delta_to_forest(
    smt_forest: &mut AccountSmtForest,
    account_id: AccountId,
    old_vault_assets: &[Asset],
    old_map_roots: &BTreeMap<StorageSlotName, Word>,
    expected_vault_root: Word,
    delta: &AccountDelta,
) -> Result<AppliedAccountDelta, StoreError> {
    // Get current tracked roots to build the final roots from
    let mut final_roots = smt_forest
        .get_roots(&account_id)
        .cloned()
        .ok_or(StoreError::AccountDataNotFound(account_id))?;

    // Vault must be updated before storage to avoid SmtForest shared-root issues:
    // when vault and storage maps start from the same empty root, updating storage
    // first would modify the tree that vault also needs.
    let (updated_assets, removed_vault_keys) = compute_vault_delta(old_vault_assets, delta)?;
    let old_vault_root = final_roots[0];
    let new_vault_root = smt_forest.update_asset_nodes(
        old_vault_root,
        updated_assets.iter().copied(),
        removed_vault_keys.iter().copied(),
    )?;

    if new_vault_root != expected_vault_root {
        return Err(StoreError::MerkleStoreError(MerkleError::ConflictingRoots {
            expected_root: expected_vault_root,
            actual_root: new_vault_root,
        }));
    }

    final_roots[0] = new_vault_root;

    // Compute storage delta via SMT forest (after vault to avoid shared-root conflicts)
    let updated_storage_slots = compute_storage_delta(smt_forest, old_map_roots, delta)?;

    // Update map roots in final_roots with new values from the delta
    let default_map_root = StorageMap::default().root();
    for (slot_name, (new_root, slot_type)) in &updated_storage_slots {
        if *slot_type == StorageSlotType::Map {
            let old_root = old_map_roots.get(slot_name).copied().unwrap_or(default_map_root);
            if let Some(root) = final_roots.iter_mut().find(|r| **r == old_root) {
                *root = *new_root;
            } else {
                // New map slot not in the old roots — append it
                final_roots.push(*new_root);
            }
        }
    }

    // Stage roots for later commit/discard
    smt_forest.stage_roots(account_id, final_roots);

    Ok(AppliedAccountDelta {
        updated_storage_slots,
        updated_assets,
        removed_vault_keys,
    })
}
