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
use super::account::utils::{apply_full_account_state, apply_transaction_delta};
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
        // Transaction Data
        insert_proven_transaction_data(
            self.db_id(),
            tx_update.executed_transaction(),
            tx_update.submission_height(),
        )
        .await?;

        // Account Data
        let delta = tx_update.executed_transaction().account_delta();
        let mut account: Account = self
            .get_account(delta.id())
            .await?
            .ok_or(StoreError::AccountDataNotFound(delta.id()))?
            .try_into()
            .map_err(|_| StoreError::AccountDataNotFound(delta.id()))?;

        if delta.is_full_state() {
            account =
                delta.try_into().expect("casting account from full state delta should not fail");
            // Full-state path: writes all entries and tombstones for removed ones
            apply_full_account_state(self.db_id(), &account).await.map_err(|err| {
                StoreError::DatabaseError(format!("failed to apply full account state: {err:?}"))
            })?;
        } else {
            account.apply_delta(delta)?;
            // Delta path: write only changed entries (single Dexie transaction)
            apply_transaction_delta(self.db_id(), &account, delta).await.map_err(|err| {
                StoreError::DatabaseError(format!("failed to apply transaction delta: {err:?}"))
            })?;
        }

        // Update SMT forest with the new account state
        {
            let mut smt_forest = self.smt_forest.write().expect("smt_forest write lock");
            smt_forest.insert_account_state(account.vault(), account.storage())?;
        }

        // Updates for notes
        apply_note_updates_tx(self.db_id(), tx_update.note_updates()).await?;

        for tag_record in tx_update.new_tags() {
            self.add_note_tag(*tag_record).await?;
        }

        Ok(())
    }
}
