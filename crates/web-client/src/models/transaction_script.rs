use miden_client::transaction::TransactionScript as NativeTransactionScript;
use wasm_bindgen::prelude::*;

use crate::models::package::Package;
use crate::models::word::Word;

#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionScript(NativeTransactionScript);

#[wasm_bindgen]
impl TransactionScript {
    pub fn root(&self) -> Word {
        self.0.root().into()
    }

    /// Creates a `NoteScript` from the given `Package`.
    /// Throws if the package is invalid.
    #[wasm_bindgen(js_name = "fromPackage")]
    pub fn from_package(package: &Package) -> Result<TransactionScript, JsValue> {
        let program = package.as_program()?;
        let native_transaction_script = NativeTransactionScript::new(program.into());
        Ok(native_transaction_script.into())
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
