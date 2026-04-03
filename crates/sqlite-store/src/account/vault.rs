//! Vault/asset-related database operations for accounts.

use std::rc::Rc;
use std::vec::Vec;

use miden_client::Word;
use miden_client::account::{AccountDelta, AccountId};
use miden_client::asset::Asset;
use miden_client::store::StoreError;
use miden_protocol::asset::AssetVaultKey;
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::sql_error::SqlResultExt;
use crate::{SqliteStore, insert_sql, subst};

impl SqliteStore {
    // READER METHODS
    // --------------------------------------------------------------------------------------------

    /// Fetches the relevant fungible assets of an account that will be updated by the account
    /// delta.
    pub(crate) fn get_account_fungible_assets_for_delta(
        conn: &Connection,
        account_id: AccountId,
        delta: &AccountDelta,
    ) -> Result<Vec<Asset>, StoreError> {
        let fungible_faucet_prefixes = delta
            .vault()
            .fungible()
            .iter()
            .map(|(faucet_id, _)| Value::Text(faucet_id.prefix().to_hex()))
            .collect::<Vec<Value>>();

        const QUERY: &str = "SELECT asset FROM latest_account_assets WHERE account_id = ? AND faucet_id_prefix IN rarray(?)";

        conn.prepare(QUERY)
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
            .collect()
    }

    // MUTATOR/WRITER METHODS
    // --------------------------------------------------------------------------------------------

    /// Inserts assets into the latest tables only.
    ///
    /// Historical archival is handled separately by the caller when needed.
    pub(crate) fn insert_assets(
        tx: &Transaction<'_>,
        account_id: AccountId,
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

        let mut latest_stmt = tx.prepare_cached(LATEST_QUERY).into_store_error()?;
        let account_id_hex = account_id.to_hex();

        for asset in assets {
            let vault_key_word: Word = asset.vault_key().into();
            let vault_key_hex = vault_key_word.to_hex();
            let faucet_prefix_hex = asset.faucet_id_prefix().to_hex();
            let asset_hex = Word::from(asset).to_hex();

            latest_stmt
                .execute(params![&account_id_hex, &vault_key_hex, &faucet_prefix_hex, &asset_hex])
                .into_store_error()?;
        }

        Ok(())
    }

    /// Persists vault delta changes: archives old values from latest to historical,
    /// then updates latest (deletes removed assets, inserts/updates changed assets).
    pub(crate) fn persist_vault_delta(
        tx: &Transaction<'_>,
        account_id_hex: &str,
        nonce_val: &rusqlite::types::Value,
        removed_vault_keys: &[AssetVaultKey],
        updated_assets: &[Asset],
    ) -> Result<(), StoreError> {
        const READ_OLD_ASSET: &str =
            "SELECT asset FROM latest_account_assets WHERE account_id = ? AND vault_key = ?";
        const HISTORICAL_INSERT: &str = insert_sql!(
            historical_account_assets {
                account_id,
                replaced_at_nonce,
                vault_key,
                faucet_id_prefix,
                old_asset
            } | REPLACE
        );
        const LATEST_INSERT: &str = insert_sql!(
            latest_account_assets {
                account_id,
                vault_key,
                faucet_id_prefix,
                asset
            } | REPLACE
        );

        let mut hist_stmt = tx.prepare_cached(HISTORICAL_INSERT).into_store_error()?;
        let mut latest_stmt = tx.prepare_cached(LATEST_INSERT).into_store_error()?;

        // Archive and delete removed assets
        for vault_key in removed_vault_keys {
            let vault_key_word: Word = (*vault_key).into();
            let vault_key_hex = vault_key_word.to_hex();
            let faucet_prefix_hex = vault_key.faucet_id_prefix().to_hex();

            // Read old asset value from latest (should exist since we're removing it)
            let old_asset: Option<String> = tx
                .query_row(READ_OLD_ASSET, params![account_id_hex, &vault_key_hex], |row| {
                    row.get(0)
                })
                .optional()
                .into_store_error()?
                .flatten();

            // Archive old value to historical
            hist_stmt
                .execute(params![
                    account_id_hex,
                    nonce_val,
                    &vault_key_hex,
                    &faucet_prefix_hex,
                    old_asset,
                ])
                .into_store_error()?;
        }

        // Batch delete removed assets from latest
        if !removed_vault_keys.is_empty() {
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
        }

        // Archive old values and insert updated assets
        for asset in updated_assets {
            let vault_key_word: Word = asset.vault_key().into();
            let vault_key_hex = vault_key_word.to_hex();
            let faucet_prefix_hex = asset.faucet_id_prefix().to_hex();
            let asset_hex = Word::from(*asset).to_hex();

            // Read old asset value from latest (NULL if asset is new)
            let old_asset: Option<String> = tx
                .query_row(READ_OLD_ASSET, params![account_id_hex, &vault_key_hex], |row| {
                    row.get(0)
                })
                .optional()
                .into_store_error()?
                .flatten();

            // Archive old value to historical (NULL old_asset = asset was new)
            hist_stmt
                .execute(params![
                    account_id_hex,
                    nonce_val,
                    &vault_key_hex,
                    &faucet_prefix_hex,
                    old_asset,
                ])
                .into_store_error()?;

            // Insert/update in latest
            latest_stmt
                .execute(params![account_id_hex, &vault_key_hex, &faucet_prefix_hex, &asset_hex])
                .into_store_error()?;
        }

        Ok(())
    }
}
