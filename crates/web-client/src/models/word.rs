use miden_objects::{Felt as NativeFelt, Word as NativeWord};
use wasm_bindgen::prelude::*;

use super::felt::Felt;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Word(NativeWord);

#[wasm_bindgen]
impl Word {
    #[wasm_bindgen(constructor)]
    pub fn new(u64_vec: Vec<u64>) -> Word {
        let fixed_array_u64: [u64; 4] = u64_vec.try_into().unwrap();

        let native_felt_vec: [NativeFelt; 4] = fixed_array_u64
            .iter()
            .map(|&v| NativeFelt::new(v))
            .collect::<Vec<NativeFelt>>()
            .try_into()
            .unwrap();

        let native_word: NativeWord = native_felt_vec.into();

        Word(native_word)
    }

    #[wasm_bindgen(js_name = "newFromFelts")]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new_from_felts(felt_vec: Vec<Felt>) -> Word {
        let native_felt_vec: [NativeFelt; 4] = felt_vec
            .iter()
            .map(|felt: &Felt| felt.into())
            .collect::<Vec<NativeFelt>>()
            .try_into()
            .unwrap();

        let native_word: NativeWord = native_felt_vec.into();

        Word(native_word)
    }

    #[wasm_bindgen(js_name = "toHex")]
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    #[wasm_bindgen(js_name = "toU64s")]
    pub fn to_u64s(&self) -> Vec<u64> {
        self.0.iter().map(NativeFelt::as_int).collect::<Vec<u64>>()
    }

    #[wasm_bindgen(js_name = "toFelts")]
    pub fn to_felts(&self) -> Vec<Felt> {
        self.0.iter().map(|felt| Felt::from(*felt)).collect::<Vec<Felt>>()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeWord> for Word {
    fn from(native_word: NativeWord) -> Self {
        Word(native_word)
    }
}

impl From<&NativeWord> for Word {
    fn from(native_word: &NativeWord) -> Self {
        Word(*native_word)
    }
}

impl From<Word> for NativeWord {
    fn from(word: Word) -> Self {
        word.0
    }
}

impl From<&Word> for NativeWord {
    fn from(word: &Word) -> Self {
        word.0
    }
}
