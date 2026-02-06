use alloc::string::String;
use core::fmt::{self, Debug, Display, Formatter};

use hex::ToHex;
use miden_protocol::note::NoteId;
use miden_protocol::{Felt, StarkField, Word};

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
        const HEX_LOWER: [char; 16] =
            ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
        [self.d0, self.d1, self.d2, self.d3]
            .into_iter()
            .flat_map(u64::to_be_bytes)
            .flat_map(|b| [HEX_LOWER[(b >> 4) as usize], HEX_LOWER[(b & 0xf) as usize]])
            .collect()
    }

    fn encode_hex_upper<T: FromIterator<char>>(&self) -> T {
        const HEX_UPPER: [char; 16] =
            ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
        [self.d0, self.d1, self.d2, self.d3]
            .into_iter()
            .flat_map(u64::to_be_bytes)
            .flat_map(|b| [HEX_UPPER[(b >> 4) as usize], HEX_UPPER[(b & 0xf) as usize]])
            .collect()
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
