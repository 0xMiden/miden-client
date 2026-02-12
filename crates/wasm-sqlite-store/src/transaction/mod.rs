use alloc::string::String;
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::account::Account;
use miden_client::note::ToInputNoteCommitments;
use miden_client::store::{StoreError, TransactionFilter};
use miden_client::transaction::{
    TransactionDetails,
    TransactionId,
    TransactionRecord,
    TransactionScript,
    TransactionStatus,
    TransactionStoreUpdate,
};
use miden_client::utils::{Deserializable, Serializable};

use super::WasmSqliteStore;
use crate::account::utils::update_account;
use crate::note::apply_note_updates;

mod js_bindings;
use js_bindings::{
    js_get_transactions,
    js_insert_transaction_script,
    js_upsert_transaction_record,
};

mod models;
use models::TransactionObject;

impl WasmSqliteStore {
    #[allow(clippy::unused_async)]
    pub(crate) async fn get_transactions(
        &self,
        filter: TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, StoreError> {
        let query = filter.to_query();
        let js_value = js_get_transactions(self.db_id(), query);
        let transactions: Vec<TransactionObject> = serde_wasm_bindgen::from_value(js_value)
            .map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize transactions: {err:?}"))
            })?;

        transactions
            .into_iter()
            .map(parse_transaction_object)
            .collect::<Result<Vec<_>, _>>()
    }

    pub(crate) async fn apply_transaction(
        &self,
        tx_update: TransactionStoreUpdate,
    ) -> Result<(), StoreError> {
        let executed_tx = tx_update.executed_transaction();

        // Build transaction details
        let nullifiers: Vec<Word> =
            executed_tx.input_notes().iter().map(|x| x.nullifier().as_word()).collect();

        let output_notes = executed_tx.output_notes();

        let details = TransactionDetails {
            account_id: executed_tx.account_id(),
            init_account_state: executed_tx.initial_account().commitment(),
            final_account_state: executed_tx.final_account().commitment(),
            input_note_nullifiers: nullifiers,
            output_notes: output_notes.clone(),
            block_num: executed_tx.block_header().block_num(),
            submission_height: tx_update.submission_height(),
            expiration_block_num: executed_tx.expiration_block_num(),
            creation_timestamp: crate::current_timestamp_u64(),
        };

        let transaction_record = TransactionRecord::new(
            executed_tx.id(),
            details,
            executed_tx.tx_args().tx_script().cloned(),
            TransactionStatus::Pending,
        );

        // Upsert the transaction script if present
        if let Some(script) = &transaction_record.script {
            let script_root = script.root().to_bytes();
            let tx_script_bytes = Some(script.to_bytes());
            js_insert_transaction_script(self.db_id(), script_root, tx_script_bytes);
        }

        // Upsert the transaction record
        let serialized = serialize_transaction_data(&transaction_record);
        js_upsert_transaction_record(
            self.db_id(),
            serialized.id,
            serialized.details,
            serialized.block_num,
            serialized.status_variant,
            serialized.status,
            serialized.script_root,
        );

        // Update account state by fetching the current account and applying the delta
        let delta = executed_tx.account_delta();
        let mut account: Account = self
            .get_account(delta.id())
            .await?
            .ok_or(StoreError::AccountDataNotFound(delta.id()))?
            .try_into()
            .map_err(|_| StoreError::AccountDataNotFound(delta.id()))?;

        if delta.is_full_state() {
            account =
                delta.try_into().expect("casting account from full state delta should not fail");
        } else {
            account.apply_delta(delta)?;
        }

        update_account(self.db_id(), &account);

        // Update notes
        apply_note_updates(self.db_id(), tx_update.note_updates());

        // Add new tags
        for tag_record in tx_update.new_tags() {
            self.add_note_tag(*tag_record).await?;
        }

        Ok(())
    }
}

struct SerializedTransactionData {
    id: String,
    script_root: Option<Vec<u8>>,
    details: Vec<u8>,
    block_num: u32,
    status_variant: u8,
    status: Vec<u8>,
}

fn serialize_transaction_data(transaction_record: &TransactionRecord) -> SerializedTransactionData {
    let transaction_id = transaction_record.id.to_hex();
    let script_root = transaction_record.script.as_ref().map(|script| script.root().to_bytes());
    let details = transaction_record.details.to_bytes();
    let block_num = transaction_record.details.block_num.as_u32();
    let status_variant = transaction_record.status.variant() as u8;
    let status = transaction_record.status.to_bytes();

    SerializedTransactionData {
        id: transaction_id,
        script_root,
        details,
        block_num,
        status_variant,
        status,
    }
}

fn parse_transaction_object(tx_obj: TransactionObject) -> Result<TransactionRecord, StoreError> {
    let id: Word = tx_obj.id.as_str().try_into()?;
    let details = TransactionDetails::read_from_bytes(&tx_obj.details)?;
    let status = TransactionStatus::read_from_bytes(&tx_obj.status)?;

    let script = tx_obj
        .tx_script
        .map(|script_bytes| TransactionScript::read_from_bytes(&script_bytes))
        .transpose()?;

    Ok(TransactionRecord {
        id: TransactionId::from_raw(id),
        details,
        script,
        status,
    })
}
