use miden_client::Word as NativeWord;
use miden_client::transaction::TransactionId as NativeTransactionId;
use wasm_bindgen::prelude::*;

use super::felt::Felt;
use super::word::Word;
use crate::js_error_with_context;

/// A unique identifier of a transaction.
///
/// Transaction ID is computed as a hash of the initial and final account commitments together with
/// the commitments of the input and output notes.
///
/// This achieves the following properties:
/// - Transactions are identical if and only if they have the same ID.
/// - Computing transaction ID can be done solely from public transaction data.
#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionId(NativeTransactionId);

#[wasm_bindgen]
impl TransactionId {
    /// Creates a `TransactionId` from a hex string.
    ///
    /// Fails if the provided string is not a valid hex representation of a `TransactionId`.
    #[wasm_bindgen(js_name = "fromHex")]
    pub fn from_hex(hex: &str) -> Result<TransactionId, JsValue> {
        let native_word = NativeWord::try_from(hex).map_err(|err| {
            js_error_with_context(err, "error instantiating TransactionId from hex")
        })?;
        let native_tx_id = NativeTransactionId::from_raw(native_word);
        Ok(TransactionId(native_tx_id))
    }

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
