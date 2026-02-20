use miden_client::transaction::TransactionId as NativeTransactionId;
use crate::prelude::*;

use super::felt::Felt;
use super::word::Word;

/// A unique identifier of a transaction.
///
/// Transaction ID is computed as a hash of the initial and final account commitments together with
/// the commitments of the input and output notes.
///
/// This achieves the following properties:
/// - Transactions are identical if and only if they have the same ID.
/// - Computing transaction ID can be done solely from public transaction data.
#[bindings]
#[derive(Clone)]
pub struct TransactionId(NativeTransactionId);

#[bindings]
impl TransactionId {
    /// Returns the transaction ID as field elements.
    #[bindings]
    pub fn as_elements(&self) -> Vec<Felt> {
        self.0.as_elements().iter().map(Into::into).collect()
    }

    /// Returns the transaction ID as raw bytes.
    #[bindings]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    /// Returns the hexadecimal encoding of the transaction ID.
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
