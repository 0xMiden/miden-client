use miden_client::transaction::TransactionScript as NativeTransactionScript;
use miden_client::vm::Package as NativePackage;
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

    /// Builds a `TransactionScript` from a `Package`.
    #[wasm_bindgen(js_name = "fromPackage")]
    pub fn from_package(package: Package) -> Result<TransactionScript, JsValue> {
        let native_package: NativePackage = package.into();

        if !native_package.is_program() {
            return Err(JsValue::from_str("Package is not a program"));
        }

        let program = native_package.unwrap_program();
        let native_transaction_script = NativeTransactionScript::new((*program).clone());
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
