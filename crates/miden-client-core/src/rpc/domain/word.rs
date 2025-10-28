use alloc::{string::String, vec::Vec};
use core::fmt::{self, Display, Formatter};

use hex::ToHex;
use miden_objects::{Felt, StarkField, Word, note::NoteId};

use crate::rpc::{errors::RpcConversionError, generated::word};

// CONSTANTS
// ================================================================================================

pub const WORD_DATA_SIZE: usize = 32;

// FORMATTING
// ================================================================================================

impl Display for word::Word {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode_hex::<String>())
    }
}

impl ToHex for &word::Word {
    fn encode_hex<T: FromIterator<char>>(&self) -> T {
        (*self).encode_hex()
    }

    fn encode_hex_upper<T: FromIterator<char>>(&self) -> T {
        (*self).encode_hex_upper()
    }
}

impl ToHex for word::Word {
    fn encode_hex<T: FromIterator<char>>(&self) -> T {
        let mut data: Vec<char> = Vec::with_capacity(WORD_DATA_SIZE);
        data.extend(format!("{:016x}", self.w0).chars());
        data.extend(format!("{:016x}", self.w1).chars());
        data.extend(format!("{:016x}", self.w2).chars());
        data.extend(format!("{:016x}", self.w3).chars());
        data.into_iter().collect()
    }

    fn encode_hex_upper<T: FromIterator<char>>(&self) -> T {
        let mut data: Vec<char> = Vec::with_capacity(WORD_DATA_SIZE);
        data.extend(format!("{:016X}", self.w0).chars());
        data.extend(format!("{:016X}", self.w1).chars());
        data.extend(format!("{:016X}", self.w2).chars());
        data.extend(format!("{:016X}", self.w3).chars());
        data.into_iter().collect()
    }
}

// INTO
// ================================================================================================

impl From<Word> for word::Word {
    fn from(value: Word) -> Self {
        Self {
            w0: value[0].as_int(),
            w1: value[1].as_int(),
            w2: value[2].as_int(),
            w3: value[3].as_int(),
        }
    }
}

impl From<&Word> for word::Word {
    fn from(value: &Word) -> Self {
        (*value).into()
    }
}

impl From<&NoteId> for word::Word {
    fn from(value: &NoteId) -> Self {
        (*value).as_word().into()
    }
}

impl From<NoteId> for word::Word {
    fn from(value: NoteId) -> Self {
        value.as_word().into()
    }
}

// FROM WORD
// ================================================================================================

impl TryFrom<word::Word> for [Felt; 4] {
    type Error = RpcConversionError;

    fn try_from(value: word::Word) -> Result<Self, Self::Error> {
        if [value.w0, value.w1, value.w2, value.w3]
            .iter()
            .all(|v| *v < <Felt as StarkField>::MODULUS)
        {
            Ok([
                Felt::new(value.w0),
                Felt::new(value.w1),
                Felt::new(value.w2),
                Felt::new(value.w3),
            ])
        } else {
            Err(RpcConversionError::NotAValidFelt)
        }
    }
}

impl TryFrom<word::Word> for Word {
    type Error = RpcConversionError;

    fn try_from(value: word::Word) -> Result<Self, Self::Error> {
        Ok(Self::new(value.try_into()?))
    }
}

impl TryFrom<&word::Word> for [Felt; 4] {
    type Error = RpcConversionError;

    fn try_from(value: &word::Word) -> Result<Self, Self::Error> {
        (*value).try_into()
    }
}

impl TryFrom<&word::Word> for Word {
    type Error = RpcConversionError;

    fn try_from(value: &word::Word) -> Result<Self, Self::Error> {
        (*value).try_into()
    }
}
