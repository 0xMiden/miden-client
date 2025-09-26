use miden_client::transaction::TransactionScript as NativeTransactionScript;
use wasm_bindgen::prelude::*;

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

    // TODO: Replace this for ScriptBuilder
    pub fn compile(script_code: &str, assembler: Assembler) -> Result<TransactionScript, JsValue> {
        let tx_script: TransactionScript = assembler.compile_transaction_script(script_code)?;

        Ok(tx_script)
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
