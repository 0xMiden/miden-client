use miden_client::transaction::TransactionId as NativeTransactionId;

use super::word::Word;

#[napi]
#[derive(Clone)]
pub struct TransactionId(pub(crate) NativeTransactionId);

#[napi]
impl TransactionId {
    /// Returns the hex representation.
    #[napi(js_name = "toHex")]
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Returns the underlying word representation.
    #[napi]
    pub fn inner(&self) -> Word {
        Word::from(self.0.as_word())
    }
}

impl From<NativeTransactionId> for TransactionId {
    fn from(native: NativeTransactionId) -> Self {
        TransactionId(native)
    }
}

impl From<&NativeTransactionId> for TransactionId {
    fn from(native: &NativeTransactionId) -> Self {
        TransactionId(*native)
    }
}

impl From<TransactionId> for NativeTransactionId {
    fn from(id: TransactionId) -> Self {
        id.0
    }
}
