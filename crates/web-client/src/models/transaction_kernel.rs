use miden_client::transaction::TransactionKernel as NativeTransactionKernel;
use js_export_macro::js_export;

use crate::models::assembler::Assembler;

/// Access to the default transaction kernel assembler.
#[js_export]
pub struct TransactionKernel(NativeTransactionKernel);

#[js_export]
impl TransactionKernel {
    /// Returns an assembler preloaded with the transaction kernel libraries.
    pub fn assembler() -> Assembler {
        NativeTransactionKernel::assembler().into()
    }
}
