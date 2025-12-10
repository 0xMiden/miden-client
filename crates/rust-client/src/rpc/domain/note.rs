use alloc::vec::Vec;

use miden_objects::block::{BlockHeader, BlockNumber};
use miden_objects::crypto::merkle::{MerklePath, SparseMerklePath};
use miden_objects::note::{
    Note,
    NoteDetails,
    NoteHeader,
    NoteId,
    NoteInclusionProof,
    NoteMetadata,
    NoteScript,
    NoteTag,
    NoteType,
};
use miden_objects::{Felt, MastForest, MastNodeId, Word};
use miden_tx::utils::Deserializable;

use super::{MissingFieldHelper, RpcConversionError};
use crate::rpc::{RpcError, generated as proto};

impl From<NoteId> for proto::note::NoteId {
    fn from(value: NoteId) -> Self {
        proto::note::NoteId { id: Some(value.into()) }
    }
}

impl TryFrom<proto::note::NoteId> for NoteId {
    type Error = RpcConversionError;

    fn try_from(value: proto::note::NoteId) -> Result<Self, Self::Error> {
        Word::try_from(value.id.ok_or(proto::note::NoteId::missing_field(stringify!(id)))?)
            .map(Into::into)
    }
}

impl TryFrom<proto::note::NoteMetadata> for NoteMetadata {
    type Error = RpcConversionError;

    fn try_from(value: proto::note::NoteMetadata) -> Result<Self, Self::Error> {
        let sender = value
            .sender
            .ok_or_else(|| proto::note::NoteMetadata::missing_field(stringify!(sender)))?
            .try_into()?;
        let note_type = NoteType::try_from(u64::from(value.note_type))?;
        let tag = NoteTag::from(value.tag);
        let execution_hint = value.execution_hint.try_into()?;

        let aux = Felt::try_from(value.aux).map_err(|_| RpcConversionError::NotAValidFelt)?;

        Ok(NoteMetadata::new(sender, note_type, tag, execution_hint, aux)?)
    }
}

impl From<NoteMetadata> for proto::note::NoteMetadata {
    fn from(value: NoteMetadata) -> Self {
        proto::note::NoteMetadata {
            sender: Some(value.sender().into()),
            note_type: value.note_type() as u32,
            tag: value.tag().into(),
            execution_hint: value.execution_hint().into(),
            aux: value.aux().into(),
        }
    }
}

impl TryFrom<proto::note::NoteInclusionInBlockProof> for NoteInclusionProof {
    type Error = RpcConversionError;

    fn try_from(value: proto::note::NoteInclusionInBlockProof) -> Result<Self, Self::Error> {
        Ok(NoteInclusionProof::new(
            value.block_num.into(),
            u16::try_from(value.note_index_in_block)
                .map_err(|_| RpcConversionError::InvalidField("NoteIndexInBlock".into()))?,
            value
                .inclusion_path
                .ok_or_else(|| {
                    proto::note::NoteInclusionInBlockProof::missing_field(stringify!(
                        inclusion_path
                    ))
                })?
                .try_into()?,
        )?)
    }
}

// SYNC NOTE
// ================================================================================================

/// Represents a `roto::rpc_store::SyncNotesResponse` with fields converted into domain types.
#[derive(Debug)]
pub struct NoteSyncInfo {
    /// Number of the latest block in the chain.
    pub chain_tip: BlockNumber,
    /// Block header of the block with the first note matching the specified criteria.
    pub block_header: BlockHeader,
    /// Proof for block header's MMR with respect to the chain tip.
    ///
    /// More specifically, the full proof consists of `forest`, `position` and `path` components.
    /// This value constitutes the `path`. The other two components can be obtained as follows:
    ///    - `position` is simply `response.block_header.block_num`.
    ///    - `forest` is the same as `response.chain_tip + 1`.
    pub mmr_path: MerklePath,
    /// List of all notes together with the Merkle paths from `response.block_header.note_root`.
    pub notes: Vec<CommittedNote>,
}

impl TryFrom<proto::rpc_store::SyncNotesResponse> for NoteSyncInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::SyncNotesResponse) -> Result<Self, Self::Error> {
        let chain_tip = value
            .pagination_info
            .ok_or(proto::rpc_store::SyncNotesResponse::missing_field(stringify!(pagination_info)))?
            .chain_tip;

        // Validate and convert block header
        let block_header = value
            .block_header
            .ok_or(proto::rpc_store::SyncNotesResponse::missing_field(stringify!(block_header)))?
            .try_into()?;

        let mmr_path = value
            .mmr_path
            .ok_or(proto::rpc_store::SyncNotesResponse::missing_field(stringify!(mmr_path)))?
            .try_into()?;

        // Validate and convert account note inclusions into an (AccountId, Word) tuple
        let mut notes = vec![];
        for note in value.notes {
            let note_id: NoteId = note
                .note_id
                .ok_or(proto::rpc_store::SyncNotesResponse::missing_field(stringify!(
                    notes.note_id
                )))?
                .try_into()?;

            let inclusion_path = note
                .inclusion_path
                .ok_or(proto::rpc_store::SyncNotesResponse::missing_field(stringify!(
                    notes.inclusion_path
                )))?
                .try_into()?;

            let metadata = note
                .metadata
                .ok_or(proto::rpc_store::SyncNotesResponse::missing_field(stringify!(
                    notes.metadata
                )))?
                .try_into()?;

            let committed_note = CommittedNote::new(
                note_id,
                u16::try_from(note.note_index_in_block).expect("note index out of range"),
                inclusion_path,
                metadata,
            );

            notes.push(committed_note);
        }

        Ok(NoteSyncInfo {
            chain_tip: chain_tip.into(),
            block_header,
            mmr_path,
            notes,
        })
    }
}

// COMMITTED NOTE
// ================================================================================================

/// Represents a committed note, returned as part of a `SyncStateResponse`.
#[derive(Debug, Clone)]
pub struct CommittedNote {
    /// Note ID of the committed note.
    note_id: NoteId,
    /// Note index for the note merkle tree.
    note_index: u16,
    /// Merkle path for the note merkle tree up to the block's note root.
    inclusion_path: SparseMerklePath,
    /// Note metadata.
    metadata: NoteMetadata,
}

impl CommittedNote {
    pub fn new(
        note_id: NoteId,
        note_index: u16,
        inclusion_path: SparseMerklePath,
        metadata: NoteMetadata,
    ) -> Self {
        Self {
            note_id,
            note_index,
            inclusion_path,
            metadata,
        }
    }

    pub fn note_id(&self) -> &NoteId {
        &self.note_id
    }

    pub fn note_index(&self) -> u16 {
        self.note_index
    }

    pub fn inclusion_path(&self) -> &SparseMerklePath {
        &self.inclusion_path
    }

    pub fn metadata(&self) -> NoteMetadata {
        self.metadata
    }
}

// FETCHED NOTE
// ================================================================================================

/// Describes the possible responses from the `GetNotesById` endpoint for a single note.
#[allow(clippy::large_enum_variant)]
pub enum FetchedNote {
    /// Details for a private note only include its [`NoteHeader`] and [`NoteInclusionProof`].
    /// Other details needed to consume the note are expected to be stored locally, off-chain.
    Private(NoteHeader, NoteInclusionProof),
    /// Contains the full [`Note`] object alongside its [`NoteInclusionProof`].
    Public(Note, NoteInclusionProof),
}

impl FetchedNote {
    /// Returns the note's inclusion details.
    pub fn inclusion_proof(&self) -> &NoteInclusionProof {
        match self {
            FetchedNote::Private(_, inclusion_proof) | FetchedNote::Public(_, inclusion_proof) => {
                inclusion_proof
            },
        }
    }

    /// Returns the note's metadata.
    pub fn metadata(&self) -> &NoteMetadata {
        match self {
            FetchedNote::Private(header, _) => header.metadata(),
            FetchedNote::Public(note, _) => note.metadata(),
        }
    }

    /// Returns the note's ID.
    pub fn id(&self) -> NoteId {
        match self {
            FetchedNote::Private(header, _) => header.id(),
            FetchedNote::Public(note, _) => note.id(),
        }
    }
}

impl TryFrom<proto::note::CommittedNote> for FetchedNote {
    type Error = RpcConversionError;

    fn try_from(value: proto::note::CommittedNote) -> Result<Self, Self::Error> {
        let inclusion_proof = value.inclusion_proof.ok_or_else(|| {
            proto::note::CommittedNote::missing_field(stringify!(inclusion_proof))
        })?;

        let note_id: NoteId = inclusion_proof
            .note_id
            .ok_or_else(|| {
                proto::note::CommittedNote::missing_field(stringify!(inclusion_proof.note_id))
            })?
            .try_into()?;

        let inclusion_proof = NoteInclusionProof::try_from(inclusion_proof)?;

        let note = value
            .note
            .ok_or_else(|| proto::note::CommittedNote::missing_field(stringify!(note)))?;

        let metadata = note
            .metadata
            .ok_or_else(|| proto::note::CommittedNote::missing_field(stringify!(note.metadata)))?
            .try_into()?;

        if let Some(detail_bytes) = note.details {
            let details = NoteDetails::read_from_bytes(&detail_bytes)?;
            let (assets, recipient) = details.into_parts();

            Ok(FetchedNote::Public(Note::new(assets, metadata, recipient), inclusion_proof))
        } else {
            let note_header = NoteHeader::new(note_id, metadata);
            Ok(FetchedNote::Private(note_header, inclusion_proof))
        }
    }
}

// NOTE SCRIPT
// ================================================================================================

impl TryFrom<proto::note::NoteScript> for NoteScript {
    type Error = RpcConversionError;

    fn try_from(note_script: proto::note::NoteScript) -> Result<Self, Self::Error> {
        let mast_forest = MastForest::read_from_bytes(&note_script.mast)?;
        let entrypoint = MastNodeId::from_u32_safe(note_script.entrypoint, &mast_forest)?;
        Ok(NoteScript::from_parts(alloc::sync::Arc::new(mast_forest), entrypoint))
    }
}
