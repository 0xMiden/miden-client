use js_export_macro::js_export;
use miden_client::account::AccountId as NativeAccountId;
use miden_client::transaction::ProvenTransaction as NativeProvenTransaction;

use crate::models::account_id::AccountId;
use crate::models::transaction_id::TransactionId;
use crate::models::word::Word;
use crate::platform::{JsBytes, JsErr};
use crate::utils::{deserialize_from_bytes, serialize_to_bytes};

/// Result of executing and proving a transaction. Contains all the data required to verify that a
/// transaction was executed correctly.
#[derive(Clone)]
#[js_export]
pub struct ProvenTransaction(NativeProvenTransaction);

#[js_export]
impl ProvenTransaction {
    /// Serializes the proven transaction into bytes.
    pub fn serialize(&self) -> JsBytes {
        serialize_to_bytes(&self.0)
    }

    /// Deserializes a proven transaction from bytes.
    pub fn deserialize(bytes: JsBytes) -> Result<ProvenTransaction, JsErr> {
        deserialize_from_bytes::<NativeProvenTransaction>(&bytes).map(ProvenTransaction)
    }

    /// Returns the transaction ID.
    pub fn id(&self) -> TransactionId {
        self.0.id().into()
    }

    /// Returns the account ID the transaction was executed against.
    #[js_export(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        let account_id: NativeAccountId = self.0.account_id();
        account_id.into()
    }

    /// Returns the reference block number used during execution.
    #[js_export(js_name = "refBlockNumber")]
    pub fn ref_block_number(&self) -> u32 {
        self.0.ref_block_num().as_u32()
    }

    /// Returns the block number at which the transaction expires.
    #[js_export(js_name = "expirationBlockNumber")]
    pub fn expiration_block_number(&self) -> u32 {
        self.0.expiration_block_num().as_u32()
    }

    // Note: proven output notes are not exposed in the web client yet.
    // The web client only exposes executed output notes via OutputNote/OutputNotes.

    /// Returns the commitment of the reference block.
    #[js_export(js_name = "refBlockCommitment")]
    pub fn ref_block_commitment(&self) -> Word {
        self.0.ref_block_commitment().into()
    }

    /// Returns the nullifiers of the consumed input notes.
    #[js_export(js_name = "nullifiers")]
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
