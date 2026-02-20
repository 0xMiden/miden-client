use miden_client::transaction::TransactionRecord as NativeTransactionRecord;

use crate::prelude::*;

use super::account_id::AccountId;
#[cfg(feature = "wasm")]
use super::output_notes::OutputNotes;
use super::transaction_id::TransactionId;
use super::transaction_status::TransactionStatus;
use super::word::Word;

/// Describes a transaction that has been executed and is being tracked on the Client.
#[derive(Clone)]
#[bindings]
pub struct TransactionRecord(NativeTransactionRecord);

#[bindings]
impl TransactionRecord {
    /// Returns the transaction ID.
    pub fn id(&self) -> TransactionId {
        self.0.id.into()
    }

    /// Returns the account this transaction was executed against.
    pub fn account_id(&self) -> AccountId {
        self.0.details.account_id.into()
    }

    /// Returns the initial account state commitment before execution.
    #[bindings]
    pub fn init_account_state(&self) -> Word {
        self.0.details.init_account_state.into()
    }

    /// Returns the final account state commitment after execution.
    #[bindings]
    pub fn final_account_state(&self) -> Word {
        self.0.details.final_account_state.into()
    }

    /// Returns the nullifiers of the consumed input notes.
    #[bindings]
    pub fn input_note_nullifiers(&self) -> Vec<Word> {
        self.0.details.input_note_nullifiers.iter().map(Into::into).collect()
    }

    // pub fn transaction_script(&self) -> Option<TransactionScript> {
    //     self.0.transaction_script.map(|script| script.into())
    // }

    /// Returns the block height in which the transaction was included.
    pub fn block_num(&self) -> u32 {
        self.0.details.block_num.as_u32()
    }

    /// Returns the block height at which the transaction was submitted.
    #[bindings]
    pub fn submission_height(&self) -> u32 {
        self.0.details.submission_height.as_u32()
    }

    /// Returns the expiration block height for the transaction.
    #[bindings]
    pub fn expiration_block_num(&self) -> u32 {
        self.0.details.expiration_block_num.as_u32()
    }

    /// Returns the current status of the transaction.
    #[bindings]
    pub fn transaction_status(&self) -> TransactionStatus {
        self.0.status.clone().into()
    }

    /// Returns the timestamp when the record was created.
    #[bindings]
    pub fn creation_timestamp(&self) -> i64 {
        self.0.details.creation_timestamp as i64
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl TransactionRecord {
    /// Returns the output notes created by this transaction.
    
    pub fn output_notes(&self) -> OutputNotes {
        self.0.details.output_notes.clone().into()
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
