use miden_objects::{Felt as NativeFelt, Word as NativeWord, vm::AdviceMap as NativeAdviceMap};
use wasm_bindgen::prelude::*;

use super::felt::{Felt, FeltArray};
use crate::models::word::Word;

#[derive(Clone)]
#[wasm_bindgen]
pub struct AdviceMap(NativeAdviceMap);

#[wasm_bindgen]
impl AdviceMap {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AdviceMap {
        AdviceMap(NativeAdviceMap::new())
    }

    pub fn insert(&mut self, key: &Word, value: &FeltArray) -> Option<Vec<Felt>> {
        let native_word: NativeWord = key.into();
        let native_felts: Vec<NativeFelt> = value.into();
        let insert_result: Option<Vec<NativeFelt>> = self.0.insert(native_word, native_felts);
        insert_result.map(|native_felts_vec| native_felts_vec.into_iter().map(Into::into).collect())
    }
}

impl Default for AdviceMap {
    fn default() -> Self {
        Self::new()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAdviceMap> for AdviceMap {
    fn from(native_advice_map: NativeAdviceMap) -> Self {
        AdviceMap(native_advice_map)
    }
}

impl From<&NativeAdviceMap> for AdviceMap {
    fn from(native_advice_map: &NativeAdviceMap) -> Self {
        AdviceMap(native_advice_map.clone())
    }
}

impl From<AdviceMap> for NativeAdviceMap {
    fn from(advice_map: AdviceMap) -> Self {
        advice_map.0
    }
}

impl From<&AdviceMap> for NativeAdviceMap {
    fn from(advice_map: &AdviceMap) -> Self {
        advice_map.0.clone()
    }
}
