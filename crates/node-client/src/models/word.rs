use miden_client::utils::{Deserializable, Serializable};
use miden_client::{Felt as NativeFelt, Word as NativeWord};
use napi::bindgen_prelude::*;

use super::felt::Felt;

#[napi]
#[derive(Clone)]
pub struct Word(pub(crate) NativeWord);

#[napi]
impl Word {
    /// Creates a word from four u64 values.
    #[napi(constructor)]
    pub fn new(values: Vec<BigInt>) -> Result<Self> {
        if values.len() != 4 {
            return Err(napi::Error::from_reason("Word requires exactly 4 values"));
        }
        let felts: [NativeFelt; 4] = [
            NativeFelt::new(values[0].get_u64().1),
            NativeFelt::new(values[1].get_u64().1),
            NativeFelt::new(values[2].get_u64().1),
            NativeFelt::new(values[3].get_u64().1),
        ];
        Ok(Word(felts.into()))
    }

    /// Creates a Word from a hex string.
    #[napi(js_name = "fromHex")]
    pub fn from_hex(hex: String) -> Result<Self> {
        let native = NativeWord::try_from(hex.as_str()).map_err(|err| {
            napi::Error::from_reason(format!("Error parsing Word from hex: {err}"))
        })?;
        Ok(Word(native))
    }

    /// Returns the hex representation of the word.
    #[napi(js_name = "toHex")]
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Serializes the word into bytes.
    #[napi]
    pub fn serialize(&self) -> Buffer {
        self.0.to_bytes().into()
    }

    /// Deserializes a word from bytes.
    #[napi]
    pub fn deserialize(bytes: Buffer) -> Result<Word> {
        let native = NativeWord::read_from_bytes(&bytes).map_err(|err| {
            napi::Error::from_reason(format!("Failed to deserialize Word: {err}"))
        })?;
        Ok(Word(native))
    }

    /// Returns the word as an array of u64 values.
    #[napi(js_name = "toU64s")]
    pub fn to_u64s(&self) -> Vec<BigInt> {
        self.0.iter().map(|f| BigInt::from(f.as_int())).collect()
    }

    /// Returns the word as an array of field elements.
    #[napi(js_name = "toFelts")]
    pub fn to_felts(&self) -> Vec<Felt> {
        self.0.iter().map(|f| Felt::from(*f)).collect()
    }
}

impl From<NativeWord> for Word {
    fn from(native: NativeWord) -> Self {
        Word(native)
    }
}

impl From<&NativeWord> for Word {
    fn from(native: &NativeWord) -> Self {
        Word(*native)
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
