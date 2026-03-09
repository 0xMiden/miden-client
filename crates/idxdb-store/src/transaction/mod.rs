use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::account::{Account, StorageMap, StorageSlotType};
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

use super::IdxdbStore;
use super::account::utils::{
    apply_full_account_state,
    apply_transaction_delta,
    compute_storage_delta,
    compute_vault_delta,
};
use super::note::utils::apply_note_updates_tx;
use crate::promise::await_js;

mod js_bindings;
use js_bindings::idxdb_get_transactions;

mod models;
use models::TransactionIdxdbObject;

pub mod utils;
use utils::insert_proven_transaction_data;

impl IdxdbStore {
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
            // Full-state path: the delta contains the complete account state.
            let account: Account =
                delta.try_into().expect("casting account from full state delta should not fail");
            apply_full_account_state(self.db_id(), &account).await.map_err(|err| {
                StoreError::DatabaseError(format!("failed to apply full account state: {err:?}"))
            })?;

            let mut smt_forest = self.smt_forest.write();
            smt_forest.insert_and_stage_account_state(
                account.id(),
                account.vault(),
                account.storage(),
            )?;
        } else {
            // Delta path: load only targeted data, avoid loading full Account.
            let faucet_id_prefixes: Vec<String> = delta
                .vault()
                .fungible()
                .iter()
                .map(|(faucet_id, _)| faucet_id.prefix().to_hex())
                .collect();
            let old_vault_assets = self
                .get_vault_assets(account_id, Some(faucet_id_prefixes))
                .await?;
            let old_map_roots = self.get_storage_map_roots(account_id).await?;

            let final_header = executed_tx.final_account();

            // Compute storage and vault changes using SMT forest, then stage new roots.
            let (updated_storage_slots, updated_assets, removed_vault_keys) = {
                let mut smt_forest = self.smt_forest.write();

                // Get current tracked roots to build the final roots from
                let mut final_roots = smt_forest
                    .get_roots(&account_id)
                    .cloned()
                    .ok_or(StoreError::AccountDataNotFound(account_id))?;

                // Storage: compute new map roots via SMT forest
                let updated_storage_slots =
                    compute_storage_delta(&mut smt_forest, &old_map_roots, delta)?;

                // Update map roots in final_roots with new values from the delta
                let default_map_root = StorageMap::default().root();
                for (slot_name, (new_root, slot_type)) in &updated_storage_slots {
                    if *slot_type == StorageSlotType::Map {
                        let old_root =
                            old_map_roots.get(slot_name).copied().unwrap_or(default_map_root);
                        if let Some(root) = final_roots.iter_mut().find(|r| **r == old_root) {
                            *root = *new_root;
                        } else {
                            // New map slot not in the old roots — append it
                            final_roots.push(*new_root);
                        }
                    }
                }

                // Vault: compute new asset values and update SMT forest
                let old_vault_root = final_roots[0];
                let (updated_assets, removed_vault_keys) =
                    compute_vault_delta(&old_vault_assets, delta)?;
                let new_vault_root = smt_forest.update_asset_nodes(
                    old_vault_root,
                    updated_assets.iter().copied(),
                    removed_vault_keys.iter().copied(),
                )?;

                if new_vault_root != final_header.vault_root() {
                    return Err(StoreError::DatabaseError(format!(
                        "computed vault root {} does not match final account header {}",
                        new_vault_root.to_hex(),
                        final_header.vault_root().to_hex(),
                    )));
                }

                // Update vault root in final_roots (first element is always vault root)
                final_roots[0] = new_vault_root;

                // Stage the new roots for later commit/discard during sync
                smt_forest.stage_roots(account_id, final_roots);

                (updated_storage_slots, updated_assets, removed_vault_keys)
            };

            apply_transaction_delta(
                self.db_id(),
                account_id,
                final_header,
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
