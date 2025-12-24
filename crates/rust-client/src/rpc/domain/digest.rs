use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Display, Formatter};

use hex::ToHex;
use miden_objects::note::NoteId;
use miden_objects::{Felt, StarkField, Word};

use crate::rpc::errors::RpcConversionError;
use crate::rpc::generated as proto;

// FORMATTING
// ================================================================================================

impl Display for proto::primitives::Digest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode_hex::<String>())
    }
}

impl Debug for proto::primitives::Digest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl ToHex for &proto::primitives::Digest {
    fn encode_hex<T: FromIterator<char>>(&self) -> T {
        (*self).encode_hex()
    }

    fn encode_hex_upper<T: FromIterator<char>>(&self) -> T {
        (*self).encode_hex_upper()
    }
}

impl ToHex for proto::primitives::Digest {
    fn encode_hex<T: FromIterator<char>>(&self) -> T {
        let mut data: Vec<char> = Vec::with_capacity(Word::SERIALIZED_SIZE * 2);
        data.extend(format!("{:016x}", self.d0).chars());
        data.extend(format!("{:016x}", self.d1).chars());
        data.extend(format!("{:016x}", self.d2).chars());
        data.extend(format!("{:016x}", self.d3).chars());
        data.into_iter().collect()
    }

    fn encode_hex_upper<T: FromIterator<char>>(&self) -> T {
        let mut data: Vec<char> = Vec::with_capacity(Word::SERIALIZED_SIZE * 2);
        data.extend(format!("{:016X}", self.d0).chars());
        data.extend(format!("{:016X}", self.d1).chars());
        data.extend(format!("{:016X}", self.d2).chars());
        data.extend(format!("{:016X}", self.d3).chars());
        data.into_iter().collect()
    }
}

// INTO
// ================================================================================================

impl From<Word> for proto::primitives::Digest {
    fn from(value: Word) -> Self {
        Self {
            d0: value[0].as_int(),
            d1: value[1].as_int(),
            d2: value[2].as_int(),
            d3: value[3].as_int(),
        }
    }
}

impl From<&Word> for proto::primitives::Digest {
    fn from(value: &Word) -> Self {
        (*value).into()
    }
}

impl From<&NoteId> for proto::primitives::Digest {
    fn from(value: &NoteId) -> Self {
        value.as_word().into()
    }
}

impl From<NoteId> for proto::primitives::Digest {
    fn from(value: NoteId) -> Self {
        value.as_word().into()
    }
}

// FROM DIGEST
// ================================================================================================

impl TryFrom<proto::primitives::Digest> for [Felt; 4] {
    type Error = RpcConversionError;

    fn try_from(value: proto::primitives::Digest) -> Result<Self, Self::Error> {
        if [value.d0, value.d1, value.d2, value.d3]
            .iter()
            .all(|v| *v < <Felt as StarkField>::MODULUS)
        {
            Ok([
                Felt::new(value.d0),
                Felt::new(value.d1),
                Felt::new(value.d2),
                Felt::new(value.d3),
            ])
        } else {
            Err(RpcConversionError::NotAValidFelt)
        }
    }
}

impl TryFrom<proto::primitives::Digest> for Word {
    type Error = RpcConversionError;

    fn try_from(value: proto::primitives::Digest) -> Result<Self, Self::Error> {
        Ok(Self::new(value.try_into()?))
    }
}

impl TryFrom<&proto::primitives::Digest> for [Felt; 4] {
    type Error = RpcConversionError;

    fn try_from(value: &proto::primitives::Digest) -> Result<Self, Self::Error> {
        (*value).try_into()
    }
}

impl TryFrom<&proto::primitives::Digest> for Word {
    type Error = RpcConversionError;

    fn try_from(value: &proto::primitives::Digest) -> Result<Self, Self::Error> {
        (*value).try_into()
    }
}
