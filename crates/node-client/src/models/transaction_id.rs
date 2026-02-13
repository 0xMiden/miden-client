use miden_client::transaction::TransactionId as NativeTransactionId;

use super::napi_wrap;
use super::word::Word;

napi_wrap!(copy TransactionId wraps NativeTransactionId);

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
