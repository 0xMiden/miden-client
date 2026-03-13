use js_export_macro::js_export;
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

use crate::models::advice_map::AdviceMap;
use crate::models::foreign_account::ForeignAccount;
use crate::models::miden_arrays::{
    ForeignAccountArray,
    NoteAndArgsArray,
    NoteDetailsAndTagArray,
    NoteRecipientArray,
    OutputNoteArray,
};
use crate::models::note_recipient::NoteRecipient;
use crate::models::output_note::OutputNote;
use crate::models::transaction_request::TransactionRequest;
use crate::models::transaction_request::note_and_args::NoteAndArgs;
use crate::models::transaction_request::note_details_and_tag::NoteDetailsAndTag;
use crate::models::transaction_script::TransactionScript;
use crate::models::word::Word;

/// A builder for a `TransactionRequest`.
///
/// Use this builder to construct a `TransactionRequest` by adding input notes, specifying
/// scripts, and setting other transaction parameters.
#[derive(Clone)]
#[js_export]
pub struct TransactionRequestBuilder(NativeTransactionRequestBuilder);

// Internal methods accessible from Rust code (not processed by napi/wasm_bindgen).
impl TransactionRequestBuilder {
    /// Creates a new empty transaction request builder (internal Rust access).
    pub(crate) fn new() -> TransactionRequestBuilder {
        let native_transaction_request = NativeTransactionRequestBuilder::new();
        TransactionRequestBuilder(native_transaction_request)
    }
}

#[js_export]
impl TransactionRequestBuilder {
    /// Creates a new empty transaction request builder.
    #[js_export(constructor)]
    pub fn js_new() -> TransactionRequestBuilder {
        TransactionRequestBuilder::new()
    }

    /// Adds input notes with optional arguments.
    #[js_export(js_name = "withInputNotes")]
    pub fn with_input_notes(&mut self, notes: NoteAndArgsArray) -> Self {
        let items: Vec<NoteAndArgs> = notes.into();
        let native_note_and_note_args: Vec<(NativeNote, Option<NativeNoteArgs>)> =
            items.into_iter().map(Into::into).collect();
        self.0 = self.0.clone().input_notes(native_note_and_note_args);
        self.clone()
    }

    /// Adds notes created by the sender that should be emitted by the transaction.
    #[js_export(js_name = "withOwnOutputNotes")]
    pub fn with_own_output_notes(&mut self, notes: OutputNoteArray) -> Self {
        let items: Vec<OutputNote> = notes.into();
        let native_output_notes: Vec<NativeOutputNote> =
            items.into_iter().map(Into::into).collect();
        self.0 = self.0.clone().own_output_notes(native_output_notes);
        self.clone()
    }

    /// Attaches a custom transaction script.
    #[js_export(js_name = "withCustomScript")]
    pub fn with_custom_script(&mut self, script: &TransactionScript) -> Self {
        let native_script: NativeTransactionScript = script.into();
        self.0 = self.0.clone().custom_script(native_script);
        self.clone()
    }

    /// Declares expected output recipients (used for verification).
    #[js_export(js_name = "withExpectedOutputRecipients")]
    pub fn with_expected_output_notes(&mut self, recipients: NoteRecipientArray) -> Self {
        let items: Vec<NoteRecipient> = recipients.into();
        let native_recipients: Vec<NativeNoteRecipient> =
            items.into_iter().map(NativeNoteRecipient::from).collect();
        self.0 = self.0.clone().expected_output_recipients(native_recipients);
        self.clone()
    }

    /// Declares notes expected to be created in follow-up executions.
    #[js_export(js_name = "withExpectedFutureNotes")]
    pub fn with_expected_future_notes(
        &mut self,
        note_details_and_tag: NoteDetailsAndTagArray,
    ) -> Self {
        let items: Vec<NoteDetailsAndTag> = note_details_and_tag.into();
        let native_note_details_and_tag: Vec<(NativeNoteDetails, NativeNoteTag)> =
            items.into_iter().map(Into::into).collect();
        self.0 = self.0.clone().expected_future_notes(native_note_details_and_tag);
        self.clone()
    }

    /// Merges an advice map to be available during script execution.
    #[js_export(js_name = "extendAdviceMap")]
    pub fn extend_advice_map(&mut self, advice_map: &AdviceMap) -> Self {
        let native_advice_map: NativeAdviceMap = advice_map.into();
        self.0 = self.0.clone().extend_advice_map(native_advice_map);
        self.clone()
    }

    /// Registers foreign accounts referenced by the transaction.
    #[js_export(js_name = "withForeignAccounts")]
    pub fn with_foreign_accounts(&mut self, foreign_accounts: ForeignAccountArray) -> Self {
        let items: Vec<ForeignAccount> = foreign_accounts.into();
        let native_foreign_accounts: Vec<NativeForeignAccount> =
            items.into_iter().map(Into::into).collect();
        self.0 = self.0.clone().foreign_accounts(native_foreign_accounts);
        self.clone()
    }

    /// Adds a transaction script argument.
    #[js_export(js_name = "withScriptArg")]
    pub fn with_script_arg(&mut self, script_arg: &Word) -> Self {
        let native_word: NativeWord = script_arg.into();
        self.0 = self.0.clone().script_arg(native_word);
        self.clone()
    }

    /// Adds an authentication argument.
    #[js_export(js_name = "withAuthArg")]
    pub fn with_auth_arg(&mut self, auth_arg: &Word) -> Self {
        let native_word: NativeWord = auth_arg.into();
        self.0 = self.0.clone().auth_arg(native_word);
        self.clone()
    }

    /// Finalizes the builder into a `TransactionRequest`.
    pub fn build(&self) -> TransactionRequest {
        TransactionRequest(self.0.clone().build().unwrap())
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
