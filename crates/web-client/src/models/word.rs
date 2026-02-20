use miden_client::{Felt as NativeFelt, Word as NativeWord};

use crate::prelude::*;

use super::felt::Felt;

#[bindings]
#[derive(Clone)]
pub struct Word(pub(crate) NativeWord);

#[bindings]
impl Word {
    /// Creates a word from four values.
    #[bindings(constructor)]
    pub fn new(values: Vec<i64>) -> JsResult<Word> {
        if values.len() != 4 {
            return Err(platform::error_from_string("Word requires exactly 4 elements"));
        }

        let native_felt_vec: [NativeFelt; 4] = [
            NativeFelt::new(values[0] as u64),
            NativeFelt::new(values[1] as u64),
            NativeFelt::new(values[2] as u64),
            NativeFelt::new(values[3] as u64),
        ];

        Ok(Word(native_felt_vec.into()))
    }

    /// Returns the hex representation of the word.
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Serializes the word into bytes.
    pub fn serialize(&self) -> JsBytes {
        platform::serialize_to_bytes(&self.0)
    }

    /// Returns the word as an array of i64 values.
    pub fn to_u64s(&self) -> Vec<i64> {
        self.0.iter().map(|f| NativeFelt::as_int(f) as i64).collect()
    }

    /// Returns the word as an array of field elements.
    pub fn to_felts(&self) -> Vec<Felt> {
        self.0.iter().map(|felt| Felt::from(*felt)).collect()
    }

    pub(crate) fn as_native(&self) -> &NativeWord {
        &self.0
    }

    /// Creates a Word from a hex string.
    #[bindings(factory)]
    pub fn from_hex(hex: String) -> JsResult<Word> {
        let native_word = NativeWord::try_from(hex.as_str())
            .map_err(|err| platform::error_from_string(&format!("Error creating Word from hex: {err}")))?;
        Ok(Word(native_word))
    }

    /// Deserializes a word from bytes.
    #[bindings(factory)]
    pub fn deserialize(bytes: &JsBytes) -> JsResult<Word> {
        platform::deserialize_from_bytes::<NativeWord>(bytes).map(Word)
    }
}

// WASM-only methods
#[cfg(feature = "wasm")]
impl Word {
    /// Creates a word from four field elements.
    
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
