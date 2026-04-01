use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::account::Account;
use miden_client::store::{StoreError, TransactionFilter, apply_account_delta_to_forest};
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
use super::account::utils::{apply_full_account_state, apply_transaction_delta};
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
            let fungible_faucet_prefixes: Vec<String> = delta
                .vault()
                .fungible()
                .iter()
                .map(|(faucet_id, _)| faucet_id.prefix().to_hex())
                .collect();
            let old_vault_assets =
                self.get_vault_assets(account_id, fungible_faucet_prefixes).await?;
            let map_slot_names: Vec<String> =
                delta.storage().maps().map(|(slot_name, _)| slot_name.to_string()).collect();
            let old_map_roots = self.get_storage_map_roots(account_id, map_slot_names).await?;

            let final_header = executed_tx.final_account();

            let applied_delta = {
                let mut smt_forest = self.smt_forest.write();
                apply_account_delta_to_forest(
                    &mut smt_forest,
                    account_id,
                    &old_vault_assets,
                    &old_map_roots,
                    final_header.vault_root(),
                    delta,
                )?
            };

            apply_transaction_delta(
                self.db_id(),
                account_id,
                final_header,
                &applied_delta.updated_storage_slots,
                &applied_delta.updated_assets,
                &applied_delta.removed_vault_keys,
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
