use miden_client::transaction::TransactionRecord as NativeTransactionRecord;
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::output_notes::OutputNotes;
use super::transaction_id::TransactionId;
use super::transaction_status::TransactionStatus;
use super::word::Word;

/// Describes a transaction that has been executed and is being tracked on the Client.
#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionRecord(NativeTransactionRecord);

#[wasm_bindgen]
impl TransactionRecord {
    /// Returns the transaction ID.
    pub fn id(&self) -> TransactionId {
        self.0.id.into()
    }

    /// Returns the account this transaction was executed against.
    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.0.details.account_id.into()
    }

    /// Returns the initial account state commitment before execution.
    #[wasm_bindgen(js_name = "initAccountState")]
    pub fn init_account_state(&self) -> Word {
        self.0.details.init_account_state.into()
    }

    /// Returns the final account state commitment after execution.
    #[wasm_bindgen(js_name = "finalAccountState")]
    pub fn final_account_state(&self) -> Word {
        self.0.details.final_account_state.into()
    }

    /// Returns the nullifiers of the consumed input notes.
    #[wasm_bindgen(js_name = "inputNoteNullifiers")]
    pub fn input_note_nullifiers(&self) -> Vec<Word> {
        self.0.details.input_note_nullifiers.iter().map(Into::into).collect()
    }

    /// Returns the output notes created by this transaction.
    #[wasm_bindgen(js_name = "outputNotes")]
    pub fn output_notes(&self) -> OutputNotes {
        self.0.details.output_notes.clone().into()
    }

    // pub fn transaction_script(&self) -> Option<TransactionScript> {
    //     self.0.transaction_script.map(|script| script.into())
    // }

    /// Returns the block height in which the transaction was included.
    #[wasm_bindgen(js_name = "blockNum")]
    pub fn block_num(&self) -> u32 {
        self.0.details.block_num.as_u32()
    }

    /// Returns the current status of the transaction.
    #[wasm_bindgen(js_name = "transactionStatus")]
    pub fn transaction_status(&self) -> TransactionStatus {
        self.0.status.clone().into()
    }

    /// Returns the timestamp when the record was created.
    #[wasm_bindgen(js_name = "creationTimestamp")]
    pub fn creation_timestamp(&self) -> u64 {
        self.0.details.creation_timestamp
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeTransactionRecord> for TransactionRecord {
    fn from(native_record: NativeTransactionRecord) -> Self {
        TransactionRecord(native_record)
    }
}

impl From<&NativeTransactionRecord> for TransactionRecord {
    fn from(native_record: &NativeTransactionRecord) -> Self {
        TransactionRecord(native_record.clone())
    }
}
