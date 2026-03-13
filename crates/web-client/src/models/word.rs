use js_export_macro::js_export;
use miden_client::{Felt as NativeFelt, Word as NativeWord};

use super::felt::Felt;
use crate::platform::{JsBytes, JsErr, JsU64, from_str_err};
use crate::utils::{deserialize_from_bytes, serialize_to_bytes};

#[derive(Clone)]
#[js_export]
pub struct Word(NativeWord);

#[js_export]
impl Word {
    /// Creates a word from four numeric values.
    #[js_export(constructor)]
    pub fn new(u64_vec: Vec<JsU64>) -> Word {
        let fixed_array_u64: [u64; 4] =
            u64_vec.iter().map(|v| *v as u64).collect::<Vec<u64>>().try_into().unwrap();
        let native_felt_vec: [NativeFelt; 4] = fixed_array_u64
            .iter()
            .map(|&v| NativeFelt::new(v))
            .collect::<Vec<NativeFelt>>()
            .try_into()
            .unwrap();
        Word(native_felt_vec.into())
    }

    /// Returns the word as an array of numeric values.
    #[js_export(js_name = "toU64s")]
    pub fn to_u64s(&self) -> Vec<JsU64> {
        self.0.iter().map(|f| NativeFelt::as_int(f) as JsU64).collect()
    }

    /// Creates a Word from a hex string.
    #[js_export(js_name = "fromHex")]
    pub fn from_hex(hex: String) -> Result<Word, JsErr> {
        let native_word = NativeWord::try_from(hex.as_str())
            .map_err(|err| from_str_err(&format!("Error instantiating Word from hex: {err}")))?;
        Ok(Word(native_word))
    }

    /// Creates a word from four field elements.
    #[js_export(js_name = "newFromFelts")]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new_from_felts(felt_vec: Vec<Felt>) -> Word {
        let native_felt_vec: [NativeFelt; 4] = felt_vec
            .iter()
            .map(|felt: &Felt| felt.into())
            .collect::<Vec<NativeFelt>>()
            .try_into()
            .unwrap();
        Word(native_felt_vec.into())
    }

    /// Returns the hex representation of the word.
    #[js_export(js_name = "toHex")]
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Serializes the word into bytes.
    pub fn serialize(&self) -> JsBytes {
        serialize_to_bytes(&self.0)
    }

    /// Deserializes a word from bytes.
    pub fn deserialize(bytes: JsBytes) -> Result<Word, JsErr> {
        let native_word = deserialize_from_bytes::<NativeWord>(&bytes)?;
        Ok(Word(native_word))
    }

    /// Returns the word as an array of field elements.
    #[js_export(js_name = "toFelts")]
    pub fn to_felts(&self) -> Vec<Felt> {
        self.0.iter().map(|felt| Felt::from(*felt)).collect::<Vec<Felt>>()
    }
}

impl Word {
    pub(crate) fn as_native(&self) -> &NativeWord {
        &self.0
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

impl_napi_from_value!(Word);
