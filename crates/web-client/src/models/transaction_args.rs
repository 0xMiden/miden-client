use miden_client::transaction::TransactionArgs as NativeTransactionArgs;

use crate::prelude::*;

#[cfg(feature = "wasm")]
use super::advice_inputs::AdviceInputs;
#[cfg(feature = "wasm")]
use super::note_id::NoteId;
use super::transaction_script::TransactionScript;
#[cfg(feature = "wasm")]
use super::word::Word;

/// Optional transaction arguments.
///
/// - Transaction script: a program that is executed in a transaction after all input notes scripts
///   have been executed.
/// - Note arguments: data put onto the stack right before a note script is executed. These are
///   different from note inputs, as the user executing the transaction can specify arbitrary note
///   args.
/// - Advice inputs: Provides data needed by the runtime, like the details of public output notes.
/// - Account inputs: Provides account data that will be accessed in the transaction.
#[derive(Clone)]
#[bindings]
pub struct TransactionArgs(NativeTransactionArgs);

#[bindings]
impl TransactionArgs {
    /// Returns the transaction script if provided.
    #[bindings]
    pub fn tx_script(&self) -> Option<TransactionScript> {
        self.0.tx_script().map(Into::into)
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl TransactionArgs {
    /// Returns note-specific arguments for the given note ID.
    
    pub fn get_note_args(&self, note_id: &NoteId) -> Option<Word> {
        self.0.get_note_args(note_id.into()).map(Into::into)
    }

    /// Returns advice inputs attached to the transaction.
    
    pub fn advice_inputs(&self) -> AdviceInputs {
        self.0.advice_inputs().into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeTransactionArgs> for TransactionArgs {
    fn from(native_args: NativeTransactionArgs) -> Self {
        TransactionArgs(native_args)
    }
}

impl From<&NativeTransactionArgs> for TransactionArgs {
    fn from(native_args: &NativeTransactionArgs) -> Self {
        TransactionArgs(native_args.clone())
    }
}
