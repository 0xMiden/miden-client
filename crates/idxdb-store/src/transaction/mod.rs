use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::account::Account;
use miden_client::store::{StoreError, TransactionFilter};
use miden_client::transaction::{
    TransactionDetails,
    TransactionId,
    TransactionRecord,
    TransactionScript,
    TransactionStatus,
    TransactionStoreUpdate,
};
use miden_client::utils::Deserializable;

use super::WebStore;
use super::account::utils::{
    apply_full_account_state, apply_transaction_delta, compute_storage_delta, compute_vault_delta,
};
use super::note::utils::apply_note_updates_tx;
use crate::promise::await_js;

mod js_bindings;
use js_bindings::idxdb_get_transactions;

mod models;
use models::TransactionIdxdbObject;

pub mod utils;
use utils::insert_proven_transaction_data;

impl WebStore {
    pub async fn get_transactions(
        &self,
        filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, StoreError> {
        let filter_as_str = match filter {
            TransactionFilter::All => "All",
            TransactionFilter::Uncommitted => "Uncommitted",
            TransactionFilter::Ids(ids) => &{
                let ids_str =
                    ids.iter().map(ToString::to_string).collect::<Vec<String>>().join(",");
                format!("Ids:{ids_str}")
            },
            TransactionFilter::ExpiredBefore(block_number) => {
                &format!("ExpiredPending:{block_number}")
            },
        };

        let promise = idxdb_get_transactions(self.db_id(), filter_as_str.to_string());
        let transactions_idxdb: Vec<TransactionIdxdbObject> =
            await_js(promise, "failed to get transactions").await?;

        let transaction_records: Result<Vec<TransactionRecord>, StoreError> = transactions_idxdb
            .into_iter()
            .map(|tx_idxdb| {
                let id: Word = tx_idxdb.id.try_into()?;

                let details = TransactionDetails::read_from_bytes(&tx_idxdb.details)?;

                let script: Option<TransactionScript> = if tx_idxdb.script_root.is_some() {
                    let tx_script = tx_idxdb
                        .tx_script
                        .map(|script| TransactionScript::read_from_bytes(&script))
                        .transpose()?
                        .expect("Transaction script should be included in the row");

                    Some(tx_script)
                } else {
                    None
                };

                let status = TransactionStatus::read_from_bytes(&tx_idxdb.status)?;

                Ok(TransactionRecord {
                    id: TransactionId::from_raw(id),
                    details,
                    script,
                    status,
                })
            })
            .collect();

        transaction_records
    }

    pub async fn apply_transaction(
        &self,
        tx_update: TransactionStoreUpdate,
    ) -> Result<(), StoreError> {
        let executed_tx = tx_update.executed_transaction();

        // Transaction Data
        insert_proven_transaction_data(self.db_id(), executed_tx, tx_update.submission_height())
            .await?;

        let delta = executed_tx.account_delta();
        let account_id = executed_tx.account_id();

        if delta.is_full_state() {
            // Full-state path: writes all entries and tombstones for removed ones
            let old_roots =
                self.collect_account_smt_roots(core::iter::once(account_id)).await?;

            let account: Account =
                delta.try_into().expect("casting account from full state delta should not fail");
            apply_full_account_state(self.db_id(), &account).await.map_err(|err| {
                StoreError::DatabaseError(format!("failed to apply full account state: {err:?}"))
            })?;

            // Update SMT forest: insert new state and release old roots
            let mut smt_forest = self.smt_forest.write().expect("smt_forest write lock");
            smt_forest.insert_account_state(account.vault(), account.storage())?;
            smt_forest.pop_roots(old_roots);
        } else {
            // Delta path: write only changed entries (single Dexie transaction)
            let (header, status) = self
                .get_account_header(account_id)
                .await?
                .ok_or(StoreError::AccountDataNotFound(account_id))?;
            let old_vault_root = header.vault_root();
            let seed = status.seed().copied();

            let old_vault_assets = self.get_vault_assets(account_id).await?;
            let old_map_roots = self.get_storage_map_roots(account_id).await?;

            let final_header = executed_tx.final_account();

            // Compute storage and vault changes using SMT forest
            let updated_storage_slots;
            let updated_assets;
            let removed_vault_keys;
            {
                let mut smt_forest =
                    self.smt_forest.write().expect("smt_forest write lock");

                // Storage: compute new map roots via SMT forest
                updated_storage_slots =
                    compute_storage_delta(&mut smt_forest, &old_map_roots, delta)?;

                // Vault: compute new asset values and update SMT forest
                let (assets, removed_keys) =
                    compute_vault_delta(&old_vault_assets, delta)?;
                let new_vault_root = smt_forest.update_asset_nodes(
                    old_vault_root,
                    assets.iter().copied(),
                    removed_keys.iter().copied(),
                )?;
                if new_vault_root != final_header.vault_root() {
                    return Err(StoreError::DatabaseError(format!(
                        "computed vault root {} does not match final account header {}",
                        new_vault_root.to_hex(),
                        final_header.vault_root().to_hex(),
                    )));
                }
                updated_assets = assets;
                removed_vault_keys = removed_keys;

                // Don't pop old roots here — matching SqliteStore's behavior.
                // Old roots stay in the forest so undo can restore pre-transaction
                // state. They get released during sync (full-state path) or undo.
                //
                // Note: the forest is mutated above (update_storage_map_nodes,
                // update_asset_nodes) before the async DB write below. If the DB
                // write fails, the forest will have extra nodes for the new roots.
                // This is a memory issue, not a correctness issue — old roots are
                // still valid and match the DB state.
            }

            // Write to DB atomically
            apply_transaction_delta(
                self.db_id(),
                account_id,
                final_header,
                seed,
                &updated_storage_slots,
                &updated_assets,
                &removed_vault_keys,
                delta,
            )
            .await
            .map_err(|err| {
                StoreError::DatabaseError(format!("failed to apply transaction delta: {err:?}"))
            })?;
        }

        // Updates for notes
        apply_note_updates_tx(self.db_id(), tx_update.note_updates()).await?;

        for tag_record in tx_update.new_tags() {
            self.add_note_tag(*tag_record).await?;
        }

        Ok(())
    }
}
