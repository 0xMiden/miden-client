use miden_client::transaction::TransactionKernel as NativeTransactionKernel;
use wasm_bindgen::prelude::*;

use crate::models::assembler::Assembler;

#[wasm_bindgen]
pub struct TransactionKernel(NativeTransactionKernel);

#[wasm_bindgen]
impl TransactionKernel {
    pub fn assembler() -> Assembler {
        NativeTransactionKernel::assembler().into()
    }
}
