use miden_client::account::AccountId as NativeAccountId;
use miden_client::transaction::ProvenTransaction as NativeProvenTransaction;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::account_id::AccountId;
use crate::models::output_notes::OutputNotes;
use crate::models::transaction_id::TransactionId;
use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Result of executing and proving a transaction. Contains all the data required to verify that a
/// transaction was executed correctly.
#[derive(Clone)]
#[wasm_bindgen]
pub struct ProvenTransaction(NativeProvenTransaction);

#[wasm_bindgen]
impl ProvenTransaction {
    /// Serializes the proven transaction into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a proven transaction from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<ProvenTransaction, JsValue> {
        deserialize_from_uint8array::<NativeProvenTransaction>(bytes).map(ProvenTransaction)
    }

    /// Returns the transaction ID.
    pub fn id(&self) -> TransactionId {
        self.0.id().into()
    }

    /// Returns the account ID the transaction was executed against.
    #[wasm_bindgen(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        let account_id: NativeAccountId = self.0.account_id();
        account_id.into()
    }

    /// Returns the reference block number used during execution.
    #[wasm_bindgen(js_name = "refBlockNumber")]
    pub fn ref_block_number(&self) -> u32 {
        self.0.ref_block_num().as_u32()
    }

    /// Returns the block number at which the transaction expires.
    #[wasm_bindgen(js_name = "expirationBlockNumber")]
    pub fn expiration_block_number(&self) -> u32 {
        self.0.expiration_block_num().as_u32()
    }

    /// Returns notes created by this transaction.
    #[wasm_bindgen(js_name = "outputNotes")]
    pub fn output_notes(&self) -> OutputNotes {
        self.0.output_notes().into()
    }

    /// Returns the commitment of the reference block.
    #[wasm_bindgen(js_name = "refBlockCommitment")]
    pub fn ref_block_commitment(&self) -> Word {
        self.0.ref_block_commitment().into()
    }

    /// Returns the nullifiers of the consumed input notes.
    #[wasm_bindgen(js_name = "nullifiers")]
    pub fn nullifiers(&self) -> Vec<Word> {
        self.0.nullifiers().map(|nullifier| Word::from(nullifier.as_word())).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<ProvenTransaction> for NativeProvenTransaction {
    fn from(proven: ProvenTransaction) -> Self {
        proven.0
    }
}

impl From<&ProvenTransaction> for NativeProvenTransaction {
    fn from(proven: &ProvenTransaction) -> Self {
        proven.0.clone()
    }
}

impl From<NativeProvenTransaction> for ProvenTransaction {
    fn from(proven: NativeProvenTransaction) -> Self {
        ProvenTransaction(proven)
    }
}

impl From<&NativeProvenTransaction> for ProvenTransaction {
    fn from(proven: &NativeProvenTransaction) -> Self {
        ProvenTransaction(proven.clone())
    }
}
