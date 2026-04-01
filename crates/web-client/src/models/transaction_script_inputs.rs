use js_export_macro::js_export;
use miden_client::{Felt as NativeFelt, Word as NativeWord};

use super::miden_arrays::FeltArray;
use super::word::Word;
use crate::models::miden_arrays::TransactionScriptInputPairArray;

/// A script argument represented as a word plus additional felts.
#[derive(Clone)]
#[js_export]
pub struct TransactionScriptInputPair {
    word: Word,
    felts: FeltArray,
}

#[js_export]
impl TransactionScriptInputPair {
    /// Creates a new script input pair.
    #[js_export(constructor)]
    pub fn new(word: Word, felts: FeltArray) -> TransactionScriptInputPair {
        TransactionScriptInputPair { word, felts }
    }

    /// Returns the word part of the input.
    pub fn word(&self) -> Word {
        self.word.clone()
    }

    /// Returns the remaining felts for the input.
    pub fn felts(&self) -> FeltArray {
        self.felts.clone()
    }
}

impl From<TransactionScriptInputPair> for (NativeWord, Vec<NativeFelt>) {
    fn from(transaction_script_input_pair: TransactionScriptInputPair) -> Self {
        let native_word: NativeWord = transaction_script_input_pair.word.into();
        let felts = transaction_script_input_pair.felts;
        let native_felts: Vec<NativeFelt> = Vec::from(felts).into_iter().map(Into::into).collect();
        (native_word, native_felts)
    }
}

impl From<&TransactionScriptInputPair> for (NativeWord, Vec<NativeFelt>) {
    fn from(transaction_script_input_pair: &TransactionScriptInputPair) -> Self {
        let native_word: NativeWord = transaction_script_input_pair.word.clone().into();
        let felts = &transaction_script_input_pair.felts;
        let native_felts: Vec<NativeFelt> = Vec::from(felts).into_iter().map(Into::into).collect();
        (native_word, native_felts)
    }
}

impl From<TransactionScriptInputPairArray> for Vec<(NativeWord, Vec<NativeFelt>)> {
    fn from(transaction_script_input_pair_array: TransactionScriptInputPairArray) -> Self {
        let items: Vec<TransactionScriptInputPair> = transaction_script_input_pair_array.into();
        items.into_iter().map(Into::into).collect()
    }
}

impl From<&TransactionScriptInputPairArray> for Vec<(NativeWord, Vec<NativeFelt>)> {
    fn from(transaction_script_input_pair_array: &TransactionScriptInputPairArray) -> Self {
        let items: Vec<TransactionScriptInputPair> = transaction_script_input_pair_array.into();
        items.into_iter().map(Into::into).collect()
    }
}

impl_napi_from_value!(TransactionScriptInputPair);
