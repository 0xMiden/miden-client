use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_objects::Word;
use miden_objects::block::BlockNumber;
use miden_objects::transaction::{ExecutedTransaction, ToInputNoteCommitments, TransactionScript};
use miden_tx::utils::Serializable;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::JsFuture;

use super::js_bindings::{idxdb_insert_transaction_script, idxdb_upsert_transaction_record};
use crate::store::StoreError;
use crate::transaction::{TransactionDetails, TransactionRecord, TransactionStatus};

// TYPES
// ================================================================================================

#[derive(Debug, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct SerializedTransactionData {
    pub id: String,
    pub details: Vec<u8>,
    #[wasm_bindgen(js_name = "scriptRoot")]
    pub script_root: Option<Vec<u8>>,
    #[wasm_bindgen(js_name = "txScript")]
    pub tx_script: Option<Vec<u8>>,
    #[wasm_bindgen(js_name = "blockNum")]
    pub block_num: String,
    pub committed: bool,
    pub discarded: bool,
    pub status: Vec<u8>,
}

// ================================================================================================

/// Converts an `ExecutedTransaction` into a `TransactionRecord` and inserts it into the store.
/// `submission_height` is the block number at which the transaction was submitted to the network.
pub async fn insert_proven_transaction_data(
    executed_transaction: &ExecutedTransaction,
    submission_height: BlockNumber,
) -> Result<(), StoreError> {
    // Build transaction record
    let nullifiers: Vec<Word> = executed_transaction
        .input_notes()
        .iter()
        .map(|x| x.nullifier().as_word())
        .collect();

    let output_notes = executed_transaction.output_notes();

    let details = TransactionDetails {
        account_id: executed_transaction.account_id(),
        init_account_state: executed_transaction.initial_account().commitment(),
        final_account_state: executed_transaction.final_account().commitment(),
        input_note_nullifiers: nullifiers,
        output_notes: output_notes.clone(),
        block_num: executed_transaction.block_header().block_num(),
        submission_height,
        expiration_block_num: executed_transaction.expiration_block_num(),
        creation_timestamp: u64::try_from(chrono::Utc::now().timestamp())
            .expect("timestamp is always after epoch"),
    };

    let transaction_record = TransactionRecord::new(
        executed_transaction.id(),
        details,
        executed_transaction.tx_args().tx_script().cloned(),
        TransactionStatus::Pending,
    );

    upsert_transaction_record(&transaction_record).await?;

    Ok(())
}

/// Serializes the transaction record into a format suitable for storage in the database.
pub(crate) fn serialize_transaction_record(
    transaction_record: &TransactionRecord,
) -> SerializedTransactionData {
    let transaction_id: String = transaction_record.id.as_word().to_hex();

    let script_root = transaction_record.script.as_ref().map(|script| script.root().to_bytes());
    let tx_script = transaction_record.script.as_ref().map(TransactionScript::to_bytes);

    let (committed, discarded) = match &transaction_record.status {
        TransactionStatus::Pending => (false, false),
        TransactionStatus::Committed { .. } => (true, false),
        TransactionStatus::Discarded(_) => (false, true),
    };

    SerializedTransactionData {
        id: transaction_id,
        script_root,
        tx_script,
        details: transaction_record.details.to_bytes(),
        block_num: transaction_record.details.block_num.as_u32().to_string(),
        committed,
        discarded,
        status: transaction_record.status.to_bytes(),
    }
}

/// Updates the transaction record in the database, inserting it if it doesn't exist.
pub(crate) async fn upsert_transaction_record(
    transaction: &TransactionRecord,
) -> Result<(), StoreError> {
    let serialized_data = serialize_transaction_record(transaction);

    if let Some(root) = serialized_data.script_root.clone() {
        let promise = idxdb_insert_transaction_script(root, serialized_data.tx_script);
        JsFuture::from(promise).await.map_err(|js_error| {
            StoreError::DatabaseError(format!("failed to insert script: {js_error:?}"))
        })?;
    }

    let promise = idxdb_upsert_transaction_record(
        serialized_data.id,
        serialized_data.details,
        serialized_data.script_root.clone(),
        serialized_data.block_num,
        serialized_data.committed.into(),
        serialized_data.discarded.into(),
        serialized_data.status,
    );
    JsFuture::from(promise).await.map_err(|js_error| {
        StoreError::DatabaseError(format!("failed to insert transaction data: {js_error:?}"))
    })?;

    Ok(())
}
