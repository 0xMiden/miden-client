#![allow(clippy::items_after_statements)]

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::rc::Rc;
use std::sync::Arc;

use miden_objects::Word;
use miden_objects::crypto::merkle::MerkleStore;
use miden_objects::crypto::utils::{Deserializable, Serializable};
use miden_objects::transaction::{ToInputNoteCommitments, TransactionScript};
use miden_tx::utils::sync::RwLock;
use rusqlite::types::Value;
use rusqlite::{Connection, Transaction, params};

use super::SqliteStore;
use super::note::apply_note_updates_tx;
use super::sync::add_note_tag_tx;
use crate::store::{StoreError, TransactionFilter};
use crate::transaction::{
    TransactionDetails,
    TransactionRecord,
    TransactionStatus,
    TransactionStatusVariant,
    TransactionStoreUpdate,
};
use crate::{insert_sql, subst};

pub(crate) const UPSERT_TRANSACTION_QUERY: &str = insert_sql!(
    transactions {
        id,
        details,
        script_root,
        block_num,
        status_variant,
        status
    } | REPLACE
);

pub(crate) const INSERT_TRANSACTION_SCRIPT_QUERY: &str =
    insert_sql!(transaction_scripts { script_root, script } | IGNORE);

// TRANSACTIONS FILTERS
// ================================================================================================

impl TransactionFilter {
    /// Returns a [String] containing the query for this Filter.
    pub fn to_query(&self) -> String {
        const QUERY: &str = "SELECT tx.id, script.script, tx.details, tx.status \
            FROM transactions AS tx LEFT JOIN transaction_scripts AS script ON tx.script_root = script.script_root";
        match self {
            TransactionFilter::All => QUERY.to_string(),
            TransactionFilter::Uncommitted => format!(
                "{QUERY} WHERE tx.status_variant != {}",
                TransactionStatusVariant::Committed as u8
            ),
            TransactionFilter::Ids(_) => {
                // Use SQLite's array parameter binding
                format!("{QUERY} WHERE tx.id IN rarray(?)")
            },
            TransactionFilter::ExpiredBefore(block_num) => {
                format!(
                    "{QUERY} WHERE tx.block_num < {} AND tx.status_variant != {} AND tx.status_variant != {}",
                    block_num.as_u32(),
                    TransactionStatusVariant::Discarded as u8,
                    TransactionStatusVariant::Committed as u8
                )
            },
        }
    }
}

// TRANSACTIONS
// ================================================================================================

struct SerializedTransactionData {
    /// Transaction ID
    id: String,
    /// Script root
    script_root: Option<Vec<u8>>,
    /// Transaction script
    tx_script: Option<Vec<u8>>,
    /// Transaction details
    details: Vec<u8>,
    /// Block number
    block_num: u32,
    /// Transaction status variant identifier
    status_variant: u8,
    /// Serialized transaction status
    status: Vec<u8>,
}

struct SerializedTransactionParts {
    /// Transaction ID
    id: String,
    /// Transaction script
    tx_script: Option<Vec<u8>>,
    /// Transaction details
    details: Vec<u8>,
    /// Serialized transaction status
    status: Vec<u8>,
}

impl SqliteStore {
    /// Retrieves tracked transactions, filtered by [`TransactionFilter`].
    pub fn get_transactions(
        conn: &mut Connection,
        filter: &TransactionFilter,
    ) -> Result<Vec<TransactionRecord>, StoreError> {
        match filter {
            TransactionFilter::Ids(ids) => {
                // Convert transaction IDs to strings for the array parameter
                let id_strings =
                    ids.iter().map(|id| Value::Text(id.to_string())).collect::<Vec<_>>();

                // Create a prepared statement and bind the array parameter
                conn.prepare(&filter.to_query())?
                    .query_map(params![Rc::new(id_strings)], parse_transaction_columns)?
                    .map(|result| Ok(result?).and_then(parse_transaction))
                    .collect::<Result<Vec<TransactionRecord>, _>>()
            },
            _ => {
                // For other filters, no parameters are needed
                conn.prepare(&filter.to_query())?
                    .query_map([], parse_transaction_columns)?
                    .map(|result| Ok(result?).and_then(parse_transaction))
                    .collect::<Result<Vec<TransactionRecord>, _>>()
            },
        }
    }

    /// Inserts a transaction and updates the current state based on the `tx_result` changes.
    pub fn apply_transaction(
        conn: &mut Connection,
        merkle_store: &Arc<RwLock<MerkleStore>>,
        tx_update: &TransactionStoreUpdate,
    ) -> Result<(), StoreError> {
        let executed_transaction = tx_update.executed_transaction();

        let updated_fungible_assets = Self::get_account_fungible_assets_for_delta(
            conn,
            &executed_transaction.initial_account().into(),
            executed_transaction.account_delta(),
        )?;

        let updated_storage_maps = Self::get_account_storage_maps_for_delta(
            conn,
            &executed_transaction.initial_account().into(),
            executed_transaction.account_delta(),
        )?;

        let tx = conn.transaction()?;

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
            submission_height: tx_update.submission_height(),
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

        // Insert transaction data
        upsert_transaction_record(&tx, &transaction_record)?;

        // Account Data
        let mut merkle_store = merkle_store.write();
        Self::apply_account_delta(
            &tx,
            &mut merkle_store,
            &executed_transaction.initial_account().into(),
            executed_transaction.final_account(),
            updated_fungible_assets,
            updated_storage_maps,
            executed_transaction.account_delta(),
        )?;
        drop(merkle_store);

        // Note Updates
        apply_note_updates_tx(&tx, tx_update.note_updates())?;

        for tag_record in tx_update.new_tags() {
            add_note_tag_tx(&tx, tag_record)?;
        }

        tx.commit().as_store_error()?;

        Ok(())
    }
}

/// Updates the transaction record in the database, inserting it if it doesn't exist.
pub(crate) fn upsert_transaction_record(
    tx: &Transaction<'_>,
    transaction: &TransactionRecord,
) -> Result<(), StoreError> {
    let SerializedTransactionData {
        id,
        script_root,
        tx_script,
        details,
        block_num,
        status_variant,
        status,
    } = serialize_transaction_data(transaction);

    if let Some(root) = script_root.clone() {
        tx.execute(INSERT_TRANSACTION_SCRIPT_QUERY, params![root, tx_script])?;
    }

    tx.execute(
        UPSERT_TRANSACTION_QUERY,
        params![id, details, script_root, block_num, status_variant, status],
    )?;

    Ok(())
}

/// Serializes the transaction record into a format suitable for storage in the database.
fn serialize_transaction_data(transaction_record: &TransactionRecord) -> SerializedTransactionData {
    let transaction_id: String = transaction_record.id.to_hex();

    let script_root = transaction_record.script.as_ref().map(|script| script.root().to_bytes());
    let tx_script = transaction_record.script.as_ref().map(TransactionScript::to_bytes);

    SerializedTransactionData {
        id: transaction_id,
        script_root,
        tx_script,
        details: transaction_record.details.to_bytes(),
        block_num: transaction_record.details.block_num.as_u32(),
        status_variant: transaction_record.status.variant() as u8,
        status: transaction_record.status.to_bytes(),
    }
}

fn parse_transaction_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedTransactionParts, rusqlite::Error> {
    let id: String = row.get(0)?;
    let tx_script: Option<Vec<u8>> = row.get(1)?;
    let details: Vec<u8> = row.get(2)?;
    let status: Vec<u8> = row.get(3)?;

    Ok(SerializedTransactionParts { id, tx_script, details, status })
}

/// Parse a transaction from the provided parts.
fn parse_transaction(
    serialized_transaction: SerializedTransactionParts,
) -> Result<TransactionRecord, StoreError> {
    let SerializedTransactionParts { id, tx_script, details, status } = serialized_transaction;

    let id: Word = id.as_str().try_into()?;

    let script: Option<TransactionScript> = tx_script
        .map(|script| TransactionScript::read_from_bytes(&script))
        .transpose()?;

    Ok(TransactionRecord {
        id: id.into(),
        details: TransactionDetails::read_from_bytes(&details)?,
        script,
        status: TransactionStatus::read_from_bytes(&status)?,
    })
}
