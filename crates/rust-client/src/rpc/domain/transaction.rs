use miden_objects::Word;
use miden_objects::account::AccountId;
use miden_objects::transaction::TransactionId;

use crate::rpc::errors::RpcConversionError;
use crate::rpc::generated as proto;

// INTO TRANSACTION ID
// ================================================================================================

impl TryFrom<proto::primitives::Digest> for TransactionId {
    type Error = RpcConversionError;

    fn try_from(value: proto::primitives::Digest) -> Result<Self, Self::Error> {
        let word: Word = value.try_into()?;
        Ok(word.into())
    }
}

impl TryFrom<proto::transaction::TransactionId> for TransactionId {
    type Error = RpcConversionError;

    fn try_from(value: proto::transaction::TransactionId) -> Result<Self, Self::Error> {
        value
            .id
            .ok_or(RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "TransactionId",
                field_name: "id",
            })?
            .try_into()
    }
}

impl From<TransactionId> for proto::transaction::TransactionId {
    fn from(value: TransactionId) -> Self {
        Self { id: Some(value.as_word().into()) }
    }
}

// TRANSACTION INCLUSION
// ================================================================================================

/// Represents a transaction that was included in the node at a certain block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionInclusion {
    /// The transaction identifier.
    pub transaction_id: TransactionId,
    /// The number of the block in which the transaction was included.
    pub block_num: u32,
    /// The account that the transaction was executed against.
    pub account_id: AccountId,
}
