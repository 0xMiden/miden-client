use miden_client::transaction::TransactionKernel as NativeTransactionKernel;
use wasm_bindgen::prelude::*;

use crate::models::assembler::Assembler;

/// Access to the default transaction kernel assembler.
#[wasm_bindgen]
pub struct TransactionKernel(NativeTransactionKernel);

#[wasm_bindgen]
impl TransactionKernel {
    /// Returns an assembler preloaded with the transaction kernel libraries.
    pub fn assembler() -> Assembler {
        NativeTransactionKernel::assembler().into()
    }
}
