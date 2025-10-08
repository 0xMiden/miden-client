use alloc::string::ToString;

use miden_objects::Word;
use miden_objects::crypto::merkle::{LeafIndex, SMT_DEPTH, SmtLeaf, SmtProof};

use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::errors::RpcConversionError;
use crate::rpc::generated as proto;

// SMT LEAF ENTRY
// ================================================================================================

impl From<&(Word, Word)> for proto::primitives::SmtLeafEntry {
    fn from(value: &(Word, Word)) -> Self {
        proto::primitives::SmtLeafEntry {
            key: Some(value.0.into()),
            value: Some(value.1.into()),
        }
    }
}

impl TryFrom<&proto::primitives::SmtLeafEntry> for (Word, Word) {
    type Error = RpcConversionError;

    fn try_from(value: &proto::primitives::SmtLeafEntry) -> Result<Self, Self::Error> {
        let key = match value.key {
            Some(key) => key.try_into()?,
            None => return Err(proto::primitives::SmtLeafEntry::missing_field(stringify!(key))),
        };

        let value: Word = match value.value {
            Some(value) => value.try_into()?,
            None => return Err(proto::primitives::SmtLeafEntry::missing_field(stringify!(value))),
        };

        Ok((key, value))
    }
}

// SMT LEAF
// ================================================================================================

impl From<SmtLeaf> for proto::primitives::SmtLeaf {
    fn from(value: SmtLeaf) -> Self {
        (&value).into()
    }
}

impl From<&SmtLeaf> for proto::primitives::SmtLeaf {
    fn from(value: &SmtLeaf) -> Self {
        match value {
            SmtLeaf::Empty(index) => proto::primitives::SmtLeaf {
                leaf: Some(proto::primitives::smt_leaf::Leaf::EmptyLeafIndex(index.value())),
            },
            SmtLeaf::Single(entry) => proto::primitives::SmtLeaf {
                leaf: Some(proto::primitives::smt_leaf::Leaf::Single(entry.into())),
            },
            SmtLeaf::Multiple(entries) => proto::primitives::SmtLeaf {
                leaf: Some(proto::primitives::smt_leaf::Leaf::Multiple(
                    proto::primitives::SmtLeafEntryList {
                        entries: entries.iter().map(Into::into).collect(),
                    },
                )),
            },
        }
    }
}

impl TryFrom<&proto::primitives::SmtLeaf> for SmtLeaf {
    type Error = RpcConversionError;

    fn try_from(value: &proto::primitives::SmtLeaf) -> Result<Self, Self::Error> {
        match &value.leaf {
            Some(proto::primitives::smt_leaf::Leaf::EmptyLeafIndex(index)) => Ok(SmtLeaf::Empty(
                LeafIndex::<SMT_DEPTH>::new(*index)
                    .map_err(|err| RpcConversionError::InvalidField(err.to_string()))?,
            )),
            Some(proto::primitives::smt_leaf::Leaf::Single(entry)) => {
                Ok(SmtLeaf::Single(entry.try_into()?))
            },
            Some(proto::primitives::smt_leaf::Leaf::Multiple(entries)) => {
                let entries =
                    entries.entries.iter().map(TryInto::try_into).collect::<Result<_, _>>()?;
                Ok(SmtLeaf::Multiple(entries))
            },
            None => Err(proto::primitives::SmtLeaf::missing_field(stringify!(leaf))),
        }
    }
}

// SMT PROOF
// ================================================================================================

impl From<SmtProof> for proto::primitives::SmtOpening {
    fn from(value: SmtProof) -> Self {
        (&value).into()
    }
}

impl From<&SmtProof> for proto::primitives::SmtOpening {
    fn from(value: &SmtProof) -> Self {
        proto::primitives::SmtOpening {
            leaf: Some(value.leaf().into()),
            path: Some(value.path().clone().into()),
        }
    }
}

impl TryFrom<&proto::primitives::SmtOpening> for SmtProof {
    type Error = RpcConversionError;

    fn try_from(value: &proto::primitives::SmtOpening) -> Result<Self, Self::Error> {
        let leaf = match &value.leaf {
            Some(leaf) => leaf.try_into()?,
            None => return Err(proto::primitives::SmtOpening::missing_field(stringify!(leaf))),
        };

        let path = match value.path.clone() {
            Some(path) => path.try_into()?,
            None => return Err(proto::primitives::SmtOpening::missing_field(stringify!(path))),
        };

        SmtProof::new(path, leaf).map_err(|err| RpcConversionError::InvalidField(err.to_string()))
    }
}
