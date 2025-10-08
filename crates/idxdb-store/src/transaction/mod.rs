use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::account::Account;
use miden_client::note::NoteUpdateTracker;
use miden_client::store::input_note_states::ExpectedNoteState;
use miden_client::store::{InputNoteState, StoreError, TransactionFilter};
use miden_client::sync::NoteTagRecord;
use miden_client::transaction::{
    TransactionDetails,
    TransactionRecord,
    TransactionScript,
    TransactionStatus,
    TransactionStoreUpdate,
};
use miden_client::utils::Deserializable;

use super::WebStore;
use super::account::utils::update_account;
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

        let promise = idxdb_get_transactions(filter_as_str.to_string());
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

                Ok(TransactionRecord { id: id.into(), details, script, status })
            })
            .collect();

        transaction_records
    }

    pub async fn apply_transaction(
        &self,
        tx_update: TransactionStoreUpdate,
        note_updates: NoteUpdateTracker,
    ) -> Result<(), StoreError> {
        // Transaction Data
        insert_proven_transaction_data(
            tx_update.executed_transaction(),
            tx_update.submission_height(),
        )
        .await?;

        // Account Data
        // TODO: This should be refactored to avoid fetching the whole account state.
        let delta = tx_update.executed_transaction().account_delta();
        let mut account: Account = self
            .get_account(delta.id())
            .await?
            .ok_or(StoreError::AccountDataNotFound(delta.id()))?
            .into();

        account.apply_delta(delta)?;

        update_account(&account).await.map_err(|err| {
            StoreError::DatabaseError(format!("failed to update account: {err:?}"))
        })?;

        // Updates for notes
        apply_note_updates_tx(&note_updates).await?;

        // Updates for tags
        let note_tags = note_updates.updated_input_notes().filter_map(|note| {
            let note = note.inner();

            if let InputNoteState::Expected(ExpectedNoteState { tag: Some(tag), .. }) = note.state()
            {
                Some(NoteTagRecord::with_note_source(*tag, note.id()))
            } else {
                None
            }
        });

        for tag_record in note_tags {
            self.add_note_tag(tag_record).await?;
        }

        Ok(())
    }
}
