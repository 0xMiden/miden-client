use miden_objects::transaction::TransactionScript as NativeTransactionScript;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::assembler::Assembler;
use crate::models::word::Word;

#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionScript(NativeTransactionScript);

#[wasm_bindgen]
impl TransactionScript {
    pub fn root(&self) -> Word {
        self.0.root().into()
    }

    pub fn compile(script_code: &str, assembler: &Assembler) -> Result<TransactionScript, JsValue> {
        let native_tx_script = NativeTransactionScript::compile(script_code, assembler.into())
            .map_err(|err| js_error_with_context(err, "failed to compile transaction script"))?;

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
