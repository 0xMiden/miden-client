use miden_client::{
    note::BlockNumber,
    transaction::{
        DiscardCause, TransactionDetails, TransactionId as NativeTransactionId,
        TransactionRecord as NativeTransactionRecord, TransactionScript,
        TransactionStatus as NativeTransactionStatus,
    },
};
use miden_objects::utils::serde::{
    ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use super::{
    account_id::AccountId, output_notes::OutputNotes, transaction_id::TransactionId,
    transaction_status::TransactionStatus, word::Word,
};
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionRecord(NativeTransactionRecord);

#[wasm_bindgen]
impl TransactionRecord {
    pub fn id(&self) -> TransactionId {
        self.0.id.into()
    }

    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.0.details.account_id.into()
    }

    #[wasm_bindgen(js_name = "initAccountState")]
    pub fn init_account_state(&self) -> Word {
        self.0.details.init_account_state.into()
    }

    #[wasm_bindgen(js_name = "finalAccountState")]
    pub fn final_account_state(&self) -> Word {
        self.0.details.final_account_state.into()
    }

    #[wasm_bindgen(js_name = "inputNoteNullifiers")]
    pub fn input_note_nullifiers(&self) -> Vec<Word> {
        self.0.details.input_note_nullifiers.iter().map(Into::into).collect()
    }

    #[wasm_bindgen(js_name = "outputNotes")]
    pub fn output_notes(&self) -> OutputNotes {
        self.0.details.output_notes.clone().into()
    }

    // pub fn transaction_script(&self) -> Option<TransactionScript> {
    //     self.0.transaction_script.map(|script| script.into())
    // }

    #[wasm_bindgen(js_name = "blockNum")]
    pub fn block_num(&self) -> u32 {
        self.0.details.block_num.as_u32()
    }

    #[wasm_bindgen(js_name = "transactionStatus")]
    pub fn transaction_status(&self) -> TransactionStatus {
        self.0.status.clone().into()
    }

    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<TransactionRecord, JsValue> {
        deserialize_from_uint8array::<TransactionRecord>(bytes)
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

impl From<&TransactionRecord> for NativeTransactionRecord {
    fn from(transaction_record: &TransactionRecord) -> Self {
        transaction_record.0.clone()
    }
}

// SERIALIZATION
// ================================================================================================

impl Serializable for TransactionRecord {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        let NativeTransactionRecord { id, details, script, status } = self.into();

        id.write_into(target);
        details.write_into(target);
        script.write_into(target);
        match status {
            NativeTransactionStatus::Pending => target.write_u8(0),
            NativeTransactionStatus::Committed(block_num) => {
                target.write_u8(1);
                block_num.write_into(target);
            },
            NativeTransactionStatus::Discarded(discard_cause) => {
                target.write_u8(2);
                discard_cause.write_into(target);
            },
        }
    }

    fn get_size_hint(&self) -> usize {
        let NativeTransactionRecord { id, details, script, status } = self.into();
        let status_size_hint = match status {
            NativeTransactionStatus::Pending => 1,
            NativeTransactionStatus::Committed(block_num) => 1 + block_num.get_size_hint(),
            NativeTransactionStatus::Discarded(discard_cause) => 1 + discard_cause.get_size_hint(),
        };
        id.get_size_hint() + details.get_size_hint() + script.get_size_hint() + status_size_hint
    }
}

impl Deserializable for TransactionRecord {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let id = NativeTransactionId::read_from(source)?;
        let details = TransactionDetails::read_from(source)?;
        let script = Option::<TransactionScript>::read_from(source)?;
        let status = match source.read_u8()? {
            0 => Ok(NativeTransactionStatus::Pending),
            1 => Ok(NativeTransactionStatus::Committed(BlockNumber::read_from(source)?)),
            2 => Ok(NativeTransactionStatus::Discarded(DiscardCause::read_from(source)?)),
            _ => Err(DeserializationError::InvalidValue("Invalid transaction status".to_string())),
        }?;

        Ok(NativeTransactionRecord::new(id, details, script, status).into())
    }
}
