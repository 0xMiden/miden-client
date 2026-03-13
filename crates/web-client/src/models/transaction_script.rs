use js_export_macro::js_export;
use miden_client::transaction::TransactionScript as NativeTransactionScript;

use crate::models::package::Package;
use crate::models::word::Word;
use crate::platform::JsErr;

/// A transaction script is a program that is executed in a transaction after all input notes have
/// been executed.
///
/// The `TransactionScript` object is composed of:
/// - An executable program defined by a MAST forest and an associated entrypoint.
/// - A set of transaction script inputs defined by a map of key-value inputs that are loaded into
///   the advice inputs' map such that the transaction script can access them.
#[derive(Clone)]
#[js_export]
pub struct TransactionScript(NativeTransactionScript);

#[js_export]
impl TransactionScript {
    /// Returns the MAST root commitment of the transaction script.
    pub fn root(&self) -> Word {
        self.0.root().into()
    }

    /// Creates a `NoteScript` from the given `Package`.
    /// Throws if the package is invalid.
    #[js_export(js_name = "fromPackage")]
    pub fn from_package(package: &Package) -> Result<TransactionScript, JsErr> {
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
