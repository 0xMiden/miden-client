use miden_objects::{
    Felt as NativeFelt, Word as NativeWord,
    transaction::TransactionScript as NativeTransactionScript,
};
use wasm_bindgen::prelude::*;

use super::rpo_digest::RpoDigest;
use crate::{
    js_error_with_context,
    models::{assembler::Assembler, transaction_script_inputs::TransactionScriptInputPairArray},
};

#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionScript(NativeTransactionScript);

#[wasm_bindgen]
impl TransactionScript {
    pub fn root(&self) -> RpoDigest {
        self.0.root().into()
    }

    pub fn compile(
        script_code: &str,
        inputs: TransactionScriptInputPairArray,
        assembler: &Assembler,
    ) -> Result<TransactionScript, JsValue> {
        let native_inputs: Vec<(NativeWord, Vec<NativeFelt>)> = inputs.into();

        let native_tx_script =
            NativeTransactionScript::compile(script_code, native_inputs, assembler.into())
                .map_err(|err| {
                    js_error_with_context(err, "failed to compile transaction script")
                })?;

        Ok(TransactionScript(native_tx_script))
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeTransactionScript> for TransactionScript {
    fn from(native_transaction_script: NativeTransactionScript) -> Self {
        TransactionScript(native_transaction_script)
    }
}

impl From<&NativeTransactionScript> for TransactionScript {
    fn from(native_transaction_script: &NativeTransactionScript) -> Self {
        TransactionScript(native_transaction_script.clone())
    }
}

impl From<TransactionScript> for NativeTransactionScript {
    fn from(transaction_script: TransactionScript) -> Self {
        transaction_script.0
    }
}

impl From<&TransactionScript> for NativeTransactionScript {
    fn from(transaction_script: &TransactionScript) -> Self {
        transaction_script.0.clone()
    }
}
