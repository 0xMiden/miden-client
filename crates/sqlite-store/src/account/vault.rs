//! Vault/asset-related database operations for accounts.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::vec::Vec;

use miden_client::Word;
use miden_client::account::{AccountDelta, AccountHeader, AccountIdPrefix};
use miden_client::asset::{Asset, FungibleAsset, NonFungibleDeltaAction};
use miden_client::store::StoreError;
use miden_protocol::asset::AssetVaultKey;
use miden_protocol::crypto::merkle::MerkleError;
use rusqlite::types::Value;
use rusqlite::{Connection, Transaction, params};

use crate::account::helpers::query_vault_assets;
use crate::smt_forest::AccountSmtForest;
use crate::sql_error::SqlResultExt;
use crate::{SqliteStore, insert_sql, subst};

impl SqliteStore {
    // READER METHODS
    // --------------------------------------------------------------------------------------------

    /// Fetches the relevant fungible assets of an account that will be updated by the account
    /// delta.
    pub(crate) fn get_account_fungible_assets_for_delta(
        conn: &Connection,
        header: &AccountHeader,
        delta: &AccountDelta,
    ) -> Result<BTreeMap<AccountIdPrefix, FungibleAsset>, StoreError> {
        let fungible_faucet_prefixes = delta
            .vault()
            .fungible()
            .iter()
            .map(|(faucet_id, _)| Value::Text(faucet_id.prefix().to_hex()))
            .collect::<Vec<Value>>();

        Ok(query_vault_assets(
            conn,
            "root = ? AND faucet_id_prefix IN rarray(?)",
            params![header.vault_root().to_hex(), Rc::new(fungible_faucet_prefixes)]
                )?
                .into_iter()
                // SAFETY: all retrieved assets should be fungible
                .map(|asset| (asset.faucet_id_prefix(), asset.unwrap_fungible()))
                .collect())
    }

    // MUTATOR/WRITER METHODS
    // --------------------------------------------------------------------------------------------

    /// Inserts assets into the vault for a specific vault root.
    pub(crate) fn insert_assets(
        tx: &Transaction<'_>,
        root: Word,
        assets: impl Iterator<Item = Asset>,
    ) -> Result<(), StoreError> {
        const QUERY: &str =
            insert_sql!(account_assets { root, vault_key, faucet_id_prefix, asset } | REPLACE);
        for asset in assets {
            let vault_key_word: Word = asset.vault_key().into();
            tx.execute(
                QUERY,
                params![
                    root.to_hex(),
                    vault_key_word.to_hex(),
                    asset.faucet_id_prefix().to_hex(),
                    Word::from(asset).to_hex(),
                ],
            )
            .into_store_error()?;
        }

        Ok(())
    }

    /// Applies vault delta changes to the account state, updating fungible and non-fungible assets.
    ///
    /// The function updates the SMT forest with all asset changes and verifies that the resulting
    /// vault root matches the expected final state. It deletes removed assets and inserts updated
    /// ones into the database.
    pub(crate) fn apply_account_vault_delta(
        tx: &Transaction<'_>,
        smt_forest: &mut AccountSmtForest,
        init_account_state: &AccountHeader,
        final_account_state: &AccountHeader,
        mut updated_fungible_assets: BTreeMap<AccountIdPrefix, FungibleAsset>,
        delta: &AccountDelta,
    ) -> Result<(), StoreError> {
        // Apply vault delta. This map will contain all updated assets (indexed by vault key), both
        // fungible and non-fungible.
        let mut updated_assets: BTreeMap<AssetVaultKey, Asset> = BTreeMap::new();
        let mut removed_vault_keys: Vec<AssetVaultKey> = Vec::new();

        // We first process the fungible assets. Adding or subtracting them from the vault as
        // requested.
        for (faucet_id, delta) in delta.vault().fungible().iter() {
            let delta_asset = FungibleAsset::new(*faucet_id, delta.unsigned_abs())?;

            let asset = match updated_fungible_assets.remove(&faucet_id.prefix()) {
                Some(asset) => {
                    // If the asset exists, update it accordingly.
                    if *delta >= 0 {
                        asset.add(delta_asset)?
                    } else {
                        asset.sub(delta_asset)?
                    }
                },
                None => {
                    // If the asset doesn't exist, we add it to the map to be inserted.
                    delta_asset
                },
            };

            if asset.amount() > 0 {
                updated_assets.insert(asset.vault_key(), Asset::Fungible(asset));
            } else {
                removed_vault_keys.push(asset.vault_key());
            }
        }

        // Process non-fungible assets. Here additions or removals don't depend on previous state as
        // each asset is unique.
        let (added_nonfungible_assets, removed_nonfungible_assets) =
            delta.vault().non_fungible().iter().partition::<Vec<_>, _>(|(_, action)| {
                matches!(action, NonFungibleDeltaAction::Add)
            });

        updated_assets.extend(
            added_nonfungible_assets
                .into_iter()
                .map(|(asset, _)| (asset.vault_key(), Asset::NonFungible(*asset))),
        );

        removed_vault_keys
            .extend(removed_nonfungible_assets.iter().map(|(asset, _)| asset.vault_key()));

        const DELETE_QUERY: &str =
            "DELETE FROM account_assets WHERE root = ? AND vault_key IN rarray(?)";

        tx.execute(
            DELETE_QUERY,
            params![
                final_account_state.vault_root().to_hex(),
                Rc::new(
                    removed_vault_keys
                        .iter()
                        .map(|k| Value::from(Word::from(*k).to_hex()))
                        .collect::<Vec<Value>>(),
                ),
            ],
        )
        .into_store_error()?;

        let updated_assets_values: Vec<Asset> = updated_assets.values().copied().collect();
        Self::insert_assets(
            tx,
            final_account_state.vault_root(),
            updated_assets_values.iter().copied(),
        )?;

        let new_vault_root = smt_forest.update_asset_nodes(
            init_account_state.vault_root(),
            updated_assets_values.iter().copied(),
            removed_vault_keys.iter().copied(),
        )?;
        if new_vault_root != final_account_state.vault_root() {
            return Err(StoreError::MerkleStoreError(MerkleError::ConflictingRoots {
                expected_root: final_account_state.vault_root(),
                actual_root: new_vault_root,
            }));
        }

        Ok(())
    }
}
