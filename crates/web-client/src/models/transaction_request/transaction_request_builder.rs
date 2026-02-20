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
use crate::prelude::*;

use crate::models::advice_map::AdviceMap;
#[cfg(feature = "wasm")]
use crate::models::miden_arrays::{
    ForeignAccountArray,
    NoteAndArgsArray,
    NoteDetailsAndTagArray,
    NoteRecipientArray,
    OutputNoteArray,
};
#[cfg(feature = "napi")]
use crate::models::foreign_account::ForeignAccount;
#[cfg(feature = "napi")]
use crate::models::note_recipient::NoteRecipient;
#[cfg(feature = "napi")]
use crate::models::output_note::OutputNote;
#[cfg(feature = "napi")]
use crate::models::transaction_request::note_and_args::NoteAndArgs;
#[cfg(feature = "napi")]
use crate::models::transaction_request::note_details_and_tag::NoteDetailsAndTag;
use crate::models::transaction_request::TransactionRequest;
use crate::models::transaction_script::TransactionScript;
use crate::models::word::Word;

/// A builder for a `TransactionRequest`.
///
/// Use this builder to construct a `TransactionRequest` by adding input notes, specifying
/// scripts, and setting other transaction parameters.
#[bindings]
#[derive(Clone)]
pub struct TransactionRequestBuilder(Option<NativeTransactionRequestBuilder>);

impl TransactionRequestBuilder {
    fn take_inner(&mut self) -> NativeTransactionRequestBuilder {
        self.0
            .take()
            .expect("TransactionRequestBuilder has already been consumed by build()")
    }
}

// Shared methods (same signatures on both platforms)
#[bindings]
impl TransactionRequestBuilder {
    /// Creates a new empty transaction request builder.
    #[bindings(constructor)]
    pub fn new() -> TransactionRequestBuilder {
        TransactionRequestBuilder(Some(NativeTransactionRequestBuilder::new()))
    }

    /// Attaches a custom transaction script.
    pub fn with_custom_script(&mut self, script: &TransactionScript) {
        let native_script: NativeTransactionScript = script.into();
        let inner = self.take_inner();
        self.0 = Some(inner.custom_script(native_script));
    }

    /// Merges an advice map to be available during script execution.
    pub fn extend_advice_map(&mut self, advice_map: &AdviceMap) {
        let native_advice_map: NativeAdviceMap = advice_map.into();
        let inner = self.take_inner();
        self.0 = Some(inner.extend_advice_map(native_advice_map));
    }

    /// Adds a transaction script argument.
    pub fn with_script_arg(&mut self, script_arg: &Word) {
        let native_word: NativeWord = script_arg.into();
        let inner = self.take_inner();
        self.0 = Some(inner.script_arg(native_word));
    }

    /// Adds an authentication argument.
    pub fn with_auth_arg(&mut self, auth_arg: &Word) {
        let native_word: NativeWord = auth_arg.into();
        let inner = self.take_inner();
        self.0 = Some(inner.auth_arg(native_word));
    }

    /// Finalizes the builder into a `TransactionRequest`.
    pub fn build(&mut self) -> TransactionRequest {
        let inner = self.take_inner();
        TransactionRequest::from(inner.build().unwrap())
    }
}

// wasm-specific methods (uses wasm array types)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl TransactionRequestBuilder {
    /// Adds input notes with optional arguments.
    pub fn with_input_notes(&mut self, notes: &NoteAndArgsArray) {
        let wrapper_notes: Vec<crate::models::transaction_request::note_and_args::NoteAndArgs> = notes.into();
        let native_note_and_note_args: Vec<(NativeNote, Option<NativeNoteArgs>)> =
            wrapper_notes.into_iter().map(Into::into).collect();
        let inner = self.take_inner();
        self.0 = Some(inner.input_notes(native_note_and_note_args));
    }

    /// Adds notes created by the sender that should be emitted by the transaction.
    pub fn with_own_output_notes(&mut self, notes: &OutputNoteArray) {
        let wrapper_notes: Vec<crate::models::output_note::OutputNote> = notes.into();
        let native_output_notes: Vec<NativeOutputNote> =
            wrapper_notes.into_iter().map(Into::into).collect();
        let inner = self.take_inner();
        self.0 = Some(inner.own_output_notes(native_output_notes));
    }

    /// Declares expected output recipients (used for verification).
    #[wasm_bindgen(js_name = "withExpectedOutputRecipients")]
    pub fn with_expected_output_notes(&mut self, recipients: &NoteRecipientArray) {
        let native_recipients: Vec<NativeNoteRecipient> = recipients.into();
        let inner = self.take_inner();
        self.0 = Some(inner.expected_output_recipients(native_recipients));
    }

    /// Declares notes expected to be created in follow-up executions.
    pub fn with_expected_future_notes(&mut self, note_details_and_tag: &NoteDetailsAndTagArray) {
        let wrapper_notes: Vec<crate::models::transaction_request::note_details_and_tag::NoteDetailsAndTag> =
            note_details_and_tag.into();
        let native_note_details_and_tag: Vec<(NativeNoteDetails, NativeNoteTag)> =
            wrapper_notes.into_iter().map(Into::into).collect();
        let inner = self.take_inner();
        self.0 = Some(inner.expected_future_notes(native_note_details_and_tag));
    }

    /// Registers foreign accounts referenced by the transaction.
    pub fn with_foreign_accounts(&mut self, foreign_accounts: &ForeignAccountArray) {
        let native_foreign_accounts: Vec<NativeForeignAccount> =
            foreign_accounts.__inner.iter().map(|account| account.clone().into()).collect();
        let inner = self.take_inner();
        self.0 = Some(inner.foreign_accounts(native_foreign_accounts));
    }
}

// napi-specific methods (uses Vec<&T> parameter types)
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl TransactionRequestBuilder {
    /// Adds input notes with optional arguments.
    pub fn with_input_notes(&mut self, notes: Vec<&NoteAndArgs>) {
        let native_note_and_note_args: Vec<(NativeNote, Option<NativeNoteArgs>)> =
            notes.into_iter().map(Into::into).collect();
        let inner = self.take_inner();
        self.0 = Some(inner.input_notes(native_note_and_note_args));
    }

    /// Adds notes created by the sender that should be emitted by the transaction.
    pub fn with_own_output_notes(&mut self, notes: Vec<&OutputNote>) {
        let native_output_notes: Vec<NativeOutputNote> =
            notes.into_iter().map(|n| n.note().clone()).collect();
        let inner = self.take_inner();
        self.0 = Some(inner.own_output_notes(native_output_notes));
    }

    /// Declares expected output recipients (used for verification).
    #[napi(js_name = "withExpectedOutputRecipients")]
    pub fn with_expected_output_notes(&mut self, recipients: Vec<&NoteRecipient>) {
        let native_recipients: Vec<NativeNoteRecipient> =
            recipients.into_iter().map(Into::into).collect();
        let inner = self.take_inner();
        self.0 = Some(inner.expected_output_recipients(native_recipients));
    }

    /// Declares notes expected to be created in follow-up executions.
    pub fn with_expected_future_notes(&mut self, note_details_and_tag: Vec<&NoteDetailsAndTag>) {
        let native_note_details_and_tag: Vec<(NativeNoteDetails, NativeNoteTag)> =
            note_details_and_tag.into_iter().map(Into::into).collect();
        let inner = self.take_inner();
        self.0 = Some(inner.expected_future_notes(native_note_details_and_tag));
    }

    /// Registers foreign accounts referenced by the transaction.
    pub fn with_foreign_accounts(&mut self, foreign_accounts: Vec<&ForeignAccount>) {
        let native_foreign_accounts: Vec<NativeForeignAccount> =
            foreign_accounts.into_iter().map(|a| a.into()).collect();
        let inner = self.take_inner();
        self.0 = Some(inner.foreign_accounts(native_foreign_accounts));
    }
}

// CONVERSIONS
// ================================================================================================

#[cfg(feature = "wasm")]
impl From<TransactionRequestBuilder> for NativeTransactionRequestBuilder {
    fn from(mut transaction_request: TransactionRequestBuilder) -> Self {
        transaction_request.take_inner()
    }
}

#[cfg(feature = "wasm")]
impl From<&TransactionRequestBuilder> for NativeTransactionRequestBuilder {
    fn from(transaction_request: &TransactionRequestBuilder) -> Self {
        transaction_request.0.clone().expect("TransactionRequestBuilder has been consumed")
    }
}

impl Default for TransactionRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}
