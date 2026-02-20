use miden_client::{Felt as NativeFelt, Word as NativeWord};

use crate::prelude::*;

#[cfg(feature = "napi")]
use super::felt::Felt;
#[cfg(feature = "wasm")]
use crate::models::miden_arrays::FeltArray;
#[cfg(feature = "wasm")]
use crate::models::miden_arrays::TransactionScriptInputPairArray;
use super::word::Word;

/// A script argument represented as a word plus additional felts.
#[derive(Clone)]
#[bindings]
pub struct TransactionScriptInputPair {
    word: Word,
    #[cfg(feature = "wasm")]
    felts: FeltArray,
    #[cfg(feature = "napi")]
    felts: Vec<Felt>,
}

#[bindings]
impl TransactionScriptInputPair {
    /// Returns the word part of the input.
    pub fn word(&self) -> Word {
        self.word.clone()
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl TransactionScriptInputPair {
    /// Creates a new script input pair.
    #[wasm_bindgen(constructor)]
    pub fn new(word: Word, felts: &FeltArray) -> TransactionScriptInputPair {
        TransactionScriptInputPair { word, felts: felts.clone() }
    }

    /// Returns the remaining felts for the input.
    pub fn felts(&self) -> FeltArray {
        self.felts.clone()
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl TransactionScriptInputPair {
    /// Creates a new script input pair.
    #[napi(constructor)]
    pub fn new(word: &Word, felts: Vec<&Felt>) -> TransactionScriptInputPair {
        TransactionScriptInputPair {
            word: word.clone(),
            felts: felts.into_iter().map(|f| *f).collect(),
        }
    }

    /// Returns the remaining felts for the input.
    pub fn felts(&self) -> Vec<Felt> {
        self.felts.clone()
    }
}

impl From<TransactionScriptInputPair> for (NativeWord, Vec<NativeFelt>) {
    fn from(pair: TransactionScriptInputPair) -> Self {
        let native_word: NativeWord = pair.word.into();
        #[cfg(feature = "wasm")]
        let native_felts: Vec<NativeFelt> =
            pair.felts.__inner.into_iter().map(Into::into).collect();
        #[cfg(feature = "napi")]
        let native_felts: Vec<NativeFelt> =
            pair.felts.into_iter().map(Into::into).collect();
        (native_word, native_felts)
    }
}

impl From<&TransactionScriptInputPair> for (NativeWord, Vec<NativeFelt>) {
    fn from(pair: &TransactionScriptInputPair) -> Self {
        let native_word: NativeWord = pair.word.clone().into();
        #[cfg(feature = "wasm")]
        let native_felts: Vec<NativeFelt> = pair
            .felts
            .__inner
            .iter()
            .map(|felt| (*felt).into())
            .collect();
        #[cfg(feature = "napi")]
        let native_felts: Vec<NativeFelt> =
            pair.felts.iter().map(|felt| (*felt).into()).collect();
        (native_word, native_felts)
    }
}

#[cfg(feature = "wasm")]
impl From<TransactionScriptInputPairArray> for Vec<(NativeWord, Vec<NativeFelt>)> {
    fn from(transaction_script_input_pair_array: TransactionScriptInputPairArray) -> Self {
        transaction_script_input_pair_array
            .__inner
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

#[cfg(feature = "wasm")]
impl From<&TransactionScriptInputPairArray> for Vec<(NativeWord, Vec<NativeFelt>)> {
    fn from(transaction_script_input_pair_array: &TransactionScriptInputPairArray) -> Self {
        transaction_script_input_pair_array.__inner.iter().map(Into::into).collect()
    }
}
