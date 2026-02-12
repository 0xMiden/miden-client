//! Vault/asset-related database operations for accounts.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::vec::Vec;

use miden_client::Word;
use miden_client::account::{AccountDelta, AccountHeader, AccountId, AccountIdPrefix};
use miden_client::asset::{Asset, FungibleAsset, NonFungibleDeltaAction};
use miden_client::store::StoreError;
use miden_protocol::asset::AssetVaultKey;
use miden_protocol::crypto::merkle::MerkleError;
use rusqlite::types::Value;
use rusqlite::{Connection, Transaction, params};

use crate::smt_forest::AccountSmtForest;
use crate::sql_error::SqlResultExt;
use crate::{SqliteStore, insert_sql, subst, u64_to_value};

impl SqliteStore {
    // READER METHODS
    // --------------------------------------------------------------------------------------------

    /// Fetches the relevant fungible assets of an account that will be updated by the account
    /// delta.
    pub(crate) fn get_account_fungible_assets_for_delta(
        conn: &Connection,
        account_id: AccountId,
        delta: &AccountDelta,
    ) -> Result<BTreeMap<AccountIdPrefix, FungibleAsset>, StoreError> {
        let fungible_faucet_prefixes = delta
            .vault()
            .fungible()
            .iter()
            .map(|(faucet_id, _)| Value::Text(faucet_id.prefix().to_hex()))
            .collect::<Vec<Value>>();

        const QUERY: &str = "SELECT asset FROM latest_account_assets WHERE account_id = ? AND faucet_id_prefix IN rarray(?)";

        Ok(conn
            .prepare(QUERY)
            .into_store_error()?
            .query_map(params![account_id.to_hex(), Rc::new(fungible_faucet_prefixes)], |row| {
                let asset: String = row.get(0)?;
                Ok(asset)
            })
            .into_store_error()?
            .map(|result| {
                let asset_str: String = result.into_store_error()?;
                let word = Word::try_from(asset_str)?;
                Ok(Asset::try_from(word)?)
            })
            .collect::<Result<Vec<Asset>, StoreError>>()?
            .into_iter()
            // SAFETY: all retrieved assets should be fungible
            .map(|asset| (asset.faucet_id_prefix(), asset.unwrap_fungible()))
            .collect())
    }

    // MUTATOR/WRITER METHODS
    // --------------------------------------------------------------------------------------------

    /// Inserts assets into both latest and historical tables for a specific
    /// (`account_id`, `nonce`).
    pub(crate) fn insert_assets(
        tx: &Transaction<'_>,
        account_id: AccountId,
        nonce: u64,
        assets: impl Iterator<Item = Asset>,
    ) -> Result<(), StoreError> {
        const LATEST_QUERY: &str = insert_sql!(
            latest_account_assets {
                account_id,
                vault_key,
                faucet_id_prefix,
                asset
            } | REPLACE
        );
        const HISTORICAL_QUERY: &str = insert_sql!(
            historical_account_assets {
                account_id,
                nonce,
                vault_key,
                faucet_id_prefix,
                asset
            } | REPLACE
        );

        let mut latest_stmt = tx.prepare_cached(LATEST_QUERY).into_store_error()?;
        let mut hist_stmt = tx.prepare_cached(HISTORICAL_QUERY).into_store_error()?;
        let account_id_hex = account_id.to_hex();
        let nonce_val = u64_to_value(nonce);

        for asset in assets {
            let vault_key_word: Word = asset.vault_key().into();
            let vault_key_hex = vault_key_word.to_hex();
            let faucet_prefix_hex = asset.faucet_id_prefix().to_hex();
            let asset_hex = Word::from(asset).to_hex();

            latest_stmt
                .execute(params![&account_id_hex, &vault_key_hex, &faucet_prefix_hex, &asset_hex])
                .into_store_error()?;

            hist_stmt
                .execute(params![
                    &account_id_hex,
                    &nonce_val,
                    &vault_key_hex,
                    &faucet_prefix_hex,
                    &asset_hex,
                ])
                .into_store_error()?;
        }

        Ok(())
    }

    /// Applies vault delta changes to the account state, updating fungible and non-fungible assets.
    ///
    /// The function updates the SMT forest with all asset changes and verifies that the resulting
    /// vault root matches the expected final state. It deletes removed assets from latest and
    /// writes tombstones to historical, then inserts updated assets.
    pub(crate) fn apply_account_vault_delta(
        tx: &Transaction<'_>,
        smt_forest: &mut AccountSmtForest,
        account_id: AccountId,
        init_account_state: &AccountHeader,
        final_account_state: &AccountHeader,
        mut updated_fungible_assets: BTreeMap<AccountIdPrefix, FungibleAsset>,
        delta: &AccountDelta,
    ) -> Result<(), StoreError> {
        let nonce = final_account_state.nonce().as_int();
        let account_id_hex = account_id.to_hex();
        let nonce_val = u64_to_value(nonce);

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

        let updated_assets_values: Vec<Asset> = updated_assets.values().copied().collect();
        Self::persist_vault_delta(
            tx,
            &account_id_hex,
            &nonce_val,
            &removed_vault_keys,
            &updated_assets_values,
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

    /// Persists vault delta changes to the database: deletes removed assets from latest,
    /// writes tombstones to historical, and inserts updated assets into both tables.
    fn persist_vault_delta(
        tx: &Transaction<'_>,
        account_id_hex: &str,
        nonce_val: &rusqlite::types::Value,
        removed_vault_keys: &[AssetVaultKey],
        updated_assets: &[Asset],
    ) -> Result<(), StoreError> {
        // Delete removed assets from latest
        const DELETE_LATEST_QUERY: &str =
            "DELETE FROM latest_account_assets WHERE account_id = ? AND vault_key IN rarray(?)";

        tx.execute(
            DELETE_LATEST_QUERY,
            params![
                account_id_hex,
                Rc::new(
                    removed_vault_keys
                        .iter()
                        .map(|k| Value::from(Word::from(*k).to_hex()))
                        .collect::<Vec<Value>>(),
                ),
            ],
        )
        .into_store_error()?;

        // Write tombstones to historical for removed assets
        const HISTORICAL_TOMBSTONE_QUERY: &str = "INSERT OR REPLACE INTO historical_account_assets \
             (account_id, nonce, vault_key, faucet_id_prefix, asset) VALUES (?, ?, ?, ?, NULL)";
        let mut tombstone_stmt =
            tx.prepare_cached(HISTORICAL_TOMBSTONE_QUERY).into_store_error()?;
        for vault_key in removed_vault_keys {
            let vault_key_word: Word = (*vault_key).into();
            let faucet_prefix_hex = vault_key.faucet_id_prefix().to_hex();
            tombstone_stmt
                .execute(params![account_id_hex, nonce_val, vault_key_word.to_hex(), faucet_prefix_hex])
                .into_store_error()?;
        }

        // Insert updated assets into latest and historical
        const LATEST_INSERT: &str = insert_sql!(
            latest_account_assets {
                account_id,
                vault_key,
                faucet_id_prefix,
                asset
            } | REPLACE
        );
        const HISTORICAL_INSERT: &str = insert_sql!(
            historical_account_assets {
                account_id,
                nonce,
                vault_key,
                faucet_id_prefix,
                asset
            } | REPLACE
        );

        let mut latest_stmt = tx.prepare_cached(LATEST_INSERT).into_store_error()?;
        let mut hist_stmt = tx.prepare_cached(HISTORICAL_INSERT).into_store_error()?;

        for asset in updated_assets {
            let vault_key_word: Word = asset.vault_key().into();
            let vault_key_hex = vault_key_word.to_hex();
            let faucet_prefix_hex = asset.faucet_id_prefix().to_hex();
            let asset_hex = Word::from(*asset).to_hex();

            latest_stmt
                .execute(params![account_id_hex, &vault_key_hex, &faucet_prefix_hex, &asset_hex])
                .into_store_error()?;

            hist_stmt
                .execute(params![
                    account_id_hex,
                    nonce_val,
                    &vault_key_hex,
                    &faucet_prefix_hex,
                    &asset_hex,
                ])
                .into_store_error()?;
        }

        Ok(())
    }
}
