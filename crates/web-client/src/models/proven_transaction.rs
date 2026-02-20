use miden_client::account::AccountId as NativeAccountId;
use miden_client::transaction::ProvenTransaction as NativeProvenTransaction;

use crate::models::account_id::AccountId;
use crate::models::output_notes::OutputNotes;
use crate::models::transaction_id::TransactionId;
use crate::models::word::Word;
use crate::platform::{self, JsBytes, JsResult};
use crate::prelude::*;

/// Result of executing and proving a transaction. Contains all the data required to verify that a
/// transaction was executed correctly.
#[derive(Clone)]
#[bindings]
pub struct ProvenTransaction(NativeProvenTransaction);

// Shared methods (identical signatures)
#[bindings]
impl ProvenTransaction {
    /// Serializes the proven transaction into bytes.
    pub fn serialize(&self) -> JsBytes {
        platform::serialize_to_bytes(&self.0)
    }

    /// Returns the transaction ID.
    pub fn id(&self) -> TransactionId {
        self.0.id().into()
    }

    /// Returns the account ID the transaction was executed against.
    #[bindings(wasm)]
    pub fn account_id(&self) -> AccountId {
        let account_id: NativeAccountId = self.0.account_id();
        account_id.into()
    }

    /// Returns the reference block number used during execution.
    #[bindings(wasm(js_name = "refBlockNumber"))]
    pub fn ref_block_number(&self) -> u32 {
        self.0.ref_block_num().as_u32()
    }

    /// Returns the block number at which the transaction expires.
    #[bindings(wasm(js_name = "expirationBlockNumber"))]
    pub fn expiration_block_number(&self) -> u32 {
        self.0.expiration_block_num().as_u32()
    }

    /// Returns the commitment of the reference block.
    #[bindings(wasm(js_name = "refBlockCommitment"))]
    pub fn ref_block_commitment(&self) -> Word {
        self.0.ref_block_commitment().into()
    }

    /// Returns the nullifiers of the consumed input notes.
    pub fn nullifiers(&self) -> Vec<Word> {
        self.0.nullifiers().map(|nullifier| Word::from(nullifier.as_word())).collect()
    }

    /// Returns notes created by this transaction.
    #[bindings(wasm)]
    pub fn output_notes(&self) -> OutputNotes {
        self.0.output_notes().into()
    }
}

// wasm-specific: deserialize (takes &JsBytes)
#[cfg(feature = "wasm")]
impl ProvenTransaction {
    /// Deserializes a proven transaction from bytes.
    pub fn deserialize(bytes: &JsBytes) -> JsResult<ProvenTransaction> {
        platform::deserialize_from_bytes::<NativeProvenTransaction>(bytes).map(ProvenTransaction)
    }
}

// napi-specific: deserialize (takes owned JsBytes, needs #[napi(factory)])
#[cfg(feature = "napi")]
impl ProvenTransaction {
    /// Deserializes a proven transaction from bytes.
    #[bindings(napi(factory))]
    pub fn deserialize(bytes: JsBytes) -> JsResult<ProvenTransaction> {
        platform::deserialize_from_bytes::<NativeProvenTransaction>(&bytes).map(ProvenTransaction)
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
