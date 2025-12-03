use miden_client::transaction::TransactionId as NativeTransactionId;
use wasm_bindgen::prelude::*;

use super::felt::Felt;
use super::word::Word;

/// Identifier of a transaction.
#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionId(NativeTransactionId);

#[wasm_bindgen]
impl TransactionId {
    /// Returns the transaction ID as field elements.
    #[wasm_bindgen(js_name = "asElements")]
    pub fn as_elements(&self) -> Vec<Felt> {
        self.0.as_elements().iter().map(Into::into).collect()
    }

    /// Returns the transaction ID as raw bytes.
    #[wasm_bindgen(js_name = "asBytes")]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    /// Returns the hexadecimal encoding of the transaction ID.
    #[wasm_bindgen(js_name = "toHex")]
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Returns the underlying word representation.
    pub fn inner(&self) -> Word {
        self.0.as_word().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeTransactionId> for TransactionId {
    fn from(native_id: NativeTransactionId) -> Self {
        TransactionId(native_id)
    }
}

impl From<&NativeTransactionId> for TransactionId {
    fn from(native_id: &NativeTransactionId) -> Self {
        TransactionId(*native_id)
    }
}

impl From<TransactionId> for NativeTransactionId {
    fn from(transaction_id: TransactionId) -> Self {
        transaction_id.0
    }
}

impl From<&TransactionId> for NativeTransactionId {
    fn from(id: &TransactionId) -> Self {
        id.0
    }
}
