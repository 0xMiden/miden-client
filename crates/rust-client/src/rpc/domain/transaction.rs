use alloc::string::ToString;
use alloc::vec::Vec;

use miden_objects::Word;
use miden_objects::account::AccountId;
use miden_objects::asset::FungibleAsset;
use miden_objects::block::BlockNumber;
use miden_objects::note::{NoteHeader, Nullifier};
use miden_objects::testing::account_id::ACCOUNT_ID_NATIVE_ASSET_FAUCET;
use miden_objects::transaction::{
    InputNoteCommitment,
    InputNotes,
    TransactionHeader,
    TransactionId,
};

use crate::rpc::{RpcConversionError, RpcError, generated as proto};

// INTO TRANSACTION ID
// ================================================================================================

impl TryFrom<proto::primitives::Digest> for TransactionId {
    type Error = RpcConversionError;

    fn try_from(value: proto::primitives::Digest) -> Result<Self, Self::Error> {
        let word: Word = value.try_into()?;
        Ok(Self::from_raw(word))
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
    pub block_num: BlockNumber,
    /// The account that the transaction was executed against.
    pub account_id: AccountId,
}

// TRANSACTIONS INFO
// ================================================================================================

/// Represent a list of transaction records that were included in a range of blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionsInfo {
    /// Current chain tip
    pub chain_tip: BlockNumber,
    /// The block number of the last check included in this response.
    pub block_num: BlockNumber,
    /// List of transaction records.
    pub transaction_records: Vec<TransactionRecord>,
}

impl TryFrom<proto::rpc_store::SyncTransactionsResponse> for TransactionsInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::SyncTransactionsResponse) -> Result<Self, Self::Error> {
        let pagination_info = value.pagination_info.ok_or(
            RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "SyncTransactionsResponse",
                field_name: "pagination_info",
            },
        )?;

        let chain_tip = pagination_info.chain_tip.into();
        let block_num = pagination_info.block_num.into();

        let transaction_records = value
            .transactions
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<TransactionRecord>, RpcError>>()?;

        Ok(Self {
            chain_tip,
            block_num,
            transaction_records,
        })
    }
}

// TRANSACTION RECORD
// ================================================================================================

/// Contains information about a transaction that got included in the chain at a specific block
/// number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionRecord {
    /// Block number in which the transaction was included.
    pub block_num: BlockNumber,
    /// A transaction header.
    pub transaction_header: TransactionHeader,
}

impl TryFrom<proto::rpc_store::TransactionRecord> for TransactionRecord {
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::TransactionRecord) -> Result<Self, Self::Error> {
        let block_num = value.block_num.into();
        let transaction_header =
            value.header.ok_or(RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "TransactionRecord",
                field_name: "transaction_header",
            })?;

        Ok(Self {
            block_num,
            transaction_header: transaction_header.try_into()?,
        })
    }
}

impl TryFrom<proto::transaction::TransactionHeader> for TransactionHeader {
    type Error = RpcError;

    fn try_from(value: proto::transaction::TransactionHeader) -> Result<Self, Self::Error> {
        let account_id =
            value
                .account_id
                .ok_or(RpcConversionError::MissingFieldInProtobufRepresentation {
                    entity: "TransactionHeader",
                    field_name: "account_id",
                })?;

        let initial_state_commitment = value.initial_state_commitment.ok_or(
            RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "TransactionHeader",
                field_name: "initial_state_commitment",
            },
        )?;

        let final_state_commitment = value.final_state_commitment.ok_or(
            RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "TransactionHeader",
                field_name: "final_state_commitment",
            },
        )?;

        let note_commitments = value
            .nullifiers
            .into_iter()
            .map(|d| {
                Nullifier::from_hex(&d.to_string())
                    .map(InputNoteCommitment::from)
                    .map_err(|e| RpcError::InvalidResponse(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let input_notes = InputNotes::new_unchecked(note_commitments);

        let output_notes = value
            .output_notes
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<NoteHeader>, RpcError>>()?;

        let transaction_header = TransactionHeader::new(
            account_id.try_into()?,
            initial_state_commitment.try_into()?,
            final_state_commitment.try_into()?,
            input_notes,
            output_notes,
            // TODO: handle this; should we open an issue in miden-node?
            FungibleAsset::new(ACCOUNT_ID_NATIVE_ASSET_FAUCET.try_into().unwrap(), 0u64).unwrap(),
        );
        Ok(transaction_header)
    }
}

impl TryFrom<proto::note::NoteSyncRecord> for NoteHeader {
    type Error = RpcError;

    fn try_from(value: proto::note::NoteSyncRecord) -> Result<Self, Self::Error> {
        let note_id = value
            .note_id
            .ok_or(RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "NoteSyncRecord",
                field_name: "note_id",
            })?
            .try_into()?;

        let note_metadata = value
            .metadata
            .ok_or(RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "NoteSyncRecord",
                field_name: "metadata",
            })?
            .try_into()?;

        let note_header = Self::new(note_id, note_metadata);
        Ok(note_header)
    }
}
