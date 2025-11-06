use miden_client::{Felt as NativeFelt, Word as NativeWord};
use wasm_bindgen::prelude::*;

use super::miden_arrays::FeltArray;
use super::word::Word;
use crate::models::miden_arrays::TransactionScriptInputPairArray;

#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionScriptInputPair {
    word: Word,
    felts: FeltArray,
}

#[wasm_bindgen]
impl TransactionScriptInputPair {
    #[wasm_bindgen(constructor)]
    pub fn new(word: Word, felts: &FeltArray) -> TransactionScriptInputPair {
        TransactionScriptInputPair { word, felts: felts.clone() }
    }

    pub fn word(&self) -> Word {
        self.word.clone()
    }

    pub fn felts(&self) -> FeltArray {
        self.felts.clone()
    }
}

impl From<TransactionScriptInputPair> for (NativeWord, Vec<NativeFelt>) {
    fn from(transaction_script_input_pair: TransactionScriptInputPair) -> Self {
        let native_word: NativeWord = transaction_script_input_pair.word.into();
        let native_felts: Vec<NativeFelt> = transaction_script_input_pair
            .felts
            .__inner
            .into_iter()
            .map(Into::into)
            .collect();
        (native_word, native_felts)
    }
}

impl From<&TransactionScriptInputPair> for (NativeWord, Vec<NativeFelt>) {
    fn from(transaction_script_input_pair: &TransactionScriptInputPair) -> Self {
        let native_word: NativeWord = transaction_script_input_pair.word.clone().into();
        let native_felts: Vec<NativeFelt> = transaction_script_input_pair
            .felts
            .__inner
            .iter()
            .map(|felt| (*felt).into())
            .collect();
        (native_word, native_felts)
    }
}

impl From<TransactionScriptInputPairArray> for Vec<(NativeWord, Vec<NativeFelt>)> {
    fn from(transaction_script_input_pair_array: TransactionScriptInputPairArray) -> Self {
        transaction_script_input_pair_array
            .__inner
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

impl From<&TransactionScriptInputPairArray> for Vec<(NativeWord, Vec<NativeFelt>)> {
    fn from(transaction_script_input_pair_array: &TransactionScriptInputPairArray) -> Self {
        transaction_script_input_pair_array.__inner.iter().map(Into::into).collect()
    }
}
