use alloc::string::ToString;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::asset::FungibleAsset;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::{NoteHeader, Nullifier};
use miden_protocol::transaction::{
    InputNoteCommitment,
    InputNotes,
    TransactionHeader,
    TransactionId,
};

use super::note::CommittedNote;
use crate::rpc::{RpcConversionError, RpcError, generated as proto};

// TODO: Remove this when we turn on fees and the node informs the correct asset account ID

/// A native asset faucet ID for use in testing scenarios.
pub const ACCOUNT_ID_NATIVE_ASSET_FAUCET: u128 = 0xab00_0000_0000_cd20_0000_ac00_0000_de00_u128;

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
    /// The initial account state commitment before the transaction was executed.
    pub initial_state_commitment: Word,
}

// TRANSACTIONS INFO
// ================================================================================================

/// Represent a list of transaction records that were included in a range of blocks.
#[derive(Debug, Clone)]
pub struct TransactionsInfo {
    /// Current chain tip
    pub chain_tip: BlockNumber,
    /// The block number of the last check included in this response.
    pub block_num: BlockNumber,
    /// List of transaction records.
    pub transaction_records: Vec<TransactionRecord>,
}

impl TryFrom<proto::rpc::SyncTransactionsResponse> for TransactionsInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc::SyncTransactionsResponse) -> Result<Self, Self::Error> {
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
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    /// Block number in which the transaction was included.
    pub block_num: BlockNumber,
    /// A transaction header.
    pub transaction_header: TransactionHeader,
    /// Output notes with inclusion proofs, as returned by the node's `SyncTransactions`
    /// response.
    pub output_notes: Vec<CommittedNote>,
}

impl TryFrom<proto::rpc::TransactionRecord> for TransactionRecord {
    type Error = RpcError;

    fn try_from(value: proto::rpc::TransactionRecord) -> Result<Self, Self::Error> {
        let block_num = value.block_num.into();
        let proto_header =
            value.header.ok_or(RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "TransactionRecord",
                field_name: "transaction_header",
            })?;

        let (transaction_header, output_notes) = convert_transaction_header(proto_header)?;

        Ok(Self {
            block_num,
            transaction_header,
            output_notes,
        })
    }
}

/// Converts a proto `TransactionHeader` into the domain `TransactionHeader` and extracts
/// committed output notes with their inclusion proofs.
///
/// The proto `output_notes` field contains `NoteSyncRecord`s (metadata header + inclusion
/// proof). We parse each into a `CommittedNote` for output note state transitions, and
/// also construct `NoteHeader`s for the `TransactionHeader` (which needs them for
/// identification purposes).
fn convert_transaction_header(
    value: proto::transaction::TransactionHeader,
) -> Result<(TransactionHeader, Vec<CommittedNote>), RpcError> {
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
        .input_notes
        .into_iter()
        .map(|d| {
            let word: Word = d
                .nullifier
                .ok_or(RpcError::ExpectedDataMissing("nullifier".into()))?
                .try_into()
                .map_err(|e: RpcConversionError| RpcError::InvalidResponse(e.to_string()))?;
            Ok(InputNoteCommitment::from(Nullifier::from_raw(word)))
        })
        .collect::<Result<Vec<_>, RpcError>>()?;
    let input_notes = InputNotes::new_unchecked(note_commitments);

    // Parse output notes as CommittedNotes (with inclusion proofs) and build NoteHeaders
    // for the TransactionHeader in a single pass. Notes with attachments may lack full
    // metadata; they are omitted from the TransactionHeader but still carried as
    // CommittedNotes for output note state transitions.
    let mut committed_output_notes = Vec::with_capacity(value.output_notes.len());
    let mut output_note_headers = Vec::with_capacity(value.output_notes.len());

    for record in value.output_notes {
        let note = CommittedNote::try_from(record).map_err(RpcError::from)?;
        if let Some(metadata) = note.metadata() {
            output_note_headers.push(NoteHeader::new(*note.note_id(), metadata.clone()));
        }
        committed_output_notes.push(note);
    }

    let transaction_header = TransactionHeader::new(
        account_id.try_into()?,
        initial_state_commitment.try_into()?,
        final_state_commitment.try_into()?,
        input_notes,
        output_note_headers,
        // TODO: handle this; should we open an issue in miden-node?
        FungibleAsset::new(ACCOUNT_ID_NATIVE_ASSET_FAUCET.try_into().expect("is valid"), 0u64)
            .unwrap(),
    );
    Ok((transaction_header, committed_output_notes))
}
