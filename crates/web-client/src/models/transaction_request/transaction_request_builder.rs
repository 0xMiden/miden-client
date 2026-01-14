use miden_client::Word as NativeWord;
use miden_client::note::{
    Note as NativeNote,
    NoteDetails as NativeNoteDetails,
    NoteRecipient as NativeNoteRecipient,
    NoteTag as NativeNoteTag,
};
use miden_client::transaction::{
    ForeignAccount as NativeForeignAccount,
    NoteArgs as NativeNoteArgs,
    OutputNote as NativeOutputNote,
    TransactionRequestBuilder as NativeTransactionRequestBuilder,
    TransactionScript as NativeTransactionScript,
};
use miden_client::vm::AdviceMap as NativeAdviceMap;
use wasm_bindgen::prelude::*;

use crate::models::advice_map::AdviceMap;
use crate::models::miden_arrays::{
    ForeignAccountArray,
    NoteAndArgsArray,
    NoteDetailsAndTagArray,
    NoteRecipientArray,
    OutputNoteArray,
};
use crate::models::transaction_request::TransactionRequest;
use crate::models::transaction_script::TransactionScript;
use crate::models::word::Word;

/// A builder for a `TransactionRequest`.
///
/// Use this builder to construct a `TransactionRequest` by adding input notes, specifying
/// scripts, and setting other transaction parameters.
#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionRequestBuilder(NativeTransactionRequestBuilder);

#[wasm_bindgen]
impl TransactionRequestBuilder {
    /// Creates a new empty transaction request builder.
    #[wasm_bindgen(constructor)]
    pub fn new() -> TransactionRequestBuilder {
        let native_transaction_request = NativeTransactionRequestBuilder::new();
        TransactionRequestBuilder(native_transaction_request)
    }

    /// Adds input notes with optional arguments.
    #[wasm_bindgen(js_name = "withInputNotes")]
    pub fn with_input_notes(mut self, notes: &NoteAndArgsArray) -> Self {
        let native_note_and_note_args: Vec<(NativeNote, Option<NativeNoteArgs>)> = notes.into();
        self.0 = self.0.input_notes(native_note_and_note_args);
        self
    }

    /// Adds notes created by the sender that should be emitted by the transaction.
    #[wasm_bindgen(js_name = "withOwnOutputNotes")]
    pub fn with_own_output_notes(mut self, notes: &OutputNoteArray) -> Self {
        let native_output_notes: Vec<NativeOutputNote> = notes.into();
        self.0 = self.0.own_output_notes(native_output_notes);
        self
    }

    /// Attaches a custom transaction script.
    #[wasm_bindgen(js_name = "withCustomScript")]
    pub fn with_custom_script(mut self, script: &TransactionScript) -> Self {
        let native_script: NativeTransactionScript = script.into();
        self.0 = self.0.custom_script(native_script);
        self
    }

    /// Declares expected output recipients (used for verification).
    #[wasm_bindgen(js_name = "withExpectedOutputRecipients")]
    pub fn with_expected_output_notes(mut self, recipients: &NoteRecipientArray) -> Self {
        let native_recipients: Vec<NativeNoteRecipient> = recipients.into();
        self.0 = self.0.expected_output_recipients(native_recipients);
        self
    }

    /// Declares notes expected to be created in follow-up executions.
    #[wasm_bindgen(js_name = "withExpectedFutureNotes")]
    pub fn with_expected_future_notes(
        mut self,
        note_details_and_tag: &NoteDetailsAndTagArray,
    ) -> Self {
        let native_note_details_and_tag: Vec<(NativeNoteDetails, NativeNoteTag)> =
            note_details_and_tag.into();
        self.0 = self.0.expected_future_notes(native_note_details_and_tag);
        self
    }

    /// Merges an advice map to be available during script execution.
    #[wasm_bindgen(js_name = "extendAdviceMap")]
    pub fn extend_advice_map(mut self, advice_map: &AdviceMap) -> Self {
        let native_advice_map: NativeAdviceMap = advice_map.into();
        self.0 = self.0.extend_advice_map(native_advice_map);
        self
    }

    /// Registers foreign accounts referenced by the transaction.
    #[wasm_bindgen(js_name = "withForeignAccounts")]
    pub fn with_foreign_accounts(mut self, foreign_accounts: &ForeignAccountArray) -> Self {
        let native_foreign_accounts: Vec<NativeForeignAccount> =
            foreign_accounts.__inner.iter().map(|account| account.clone().into()).collect();
        self.0 = self.0.foreign_accounts(native_foreign_accounts);
        self
    }

    /// Adds a transaction script argument.
    #[wasm_bindgen(js_name = "withScriptArg")]
    pub fn with_script_arg(mut self, script_arg: &Word) -> Self {
        let native_word: NativeWord = script_arg.into();
        self.0 = self.0.script_arg(native_word);
        self
    }

    /// Adds an authentication argument.
    #[wasm_bindgen(js_name = "withAuthArg")]
    pub fn with_auth_arg(mut self, auth_arg: &Word) -> Self {
        let native_word: NativeWord = auth_arg.into();
        self.0 = self.0.auth_arg(native_word);
        self
    }

    /// Finalizes the builder into a `TransactionRequest`.
    pub fn build(self) -> TransactionRequest {
        TransactionRequest(self.0.build().unwrap())
    }
}

// CONVERSIONS
// ================================================================================================

impl From<TransactionRequestBuilder> for NativeTransactionRequestBuilder {
    fn from(transaction_request: TransactionRequestBuilder) -> Self {
        transaction_request.0
    }
}

impl From<&TransactionRequestBuilder> for NativeTransactionRequestBuilder {
    fn from(transaction_request: &TransactionRequestBuilder) -> Self {
        transaction_request.0.clone()
    }
}

impl Default for TransactionRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}
