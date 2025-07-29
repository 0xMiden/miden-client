use alloc::vec::Vec;

use miden_objects::{
    Felt, Word,
    block::BlockHeader,
    crypto::merkle::{MerklePath, SparseMerklePath},
    note::{Note, NoteDetails, NoteId, NoteInclusionProof, NoteMetadata, NoteTag, NoteType},
};
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
        Word::try_from(value.id.ok_or(proto::note::NoteId::missing_field("Notes.Id"))?)
            .map(Into::into)
    }
}

impl TryFrom<proto::note::NoteMetadata> for NoteMetadata {
    type Error = RpcConversionError;

    fn try_from(value: proto::note::NoteMetadata) -> Result<Self, Self::Error> {
        let sender = value
            .sender
            .ok_or_else(|| proto::note::NoteMetadata::missing_field("Sender"))?
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
                .ok_or_else(|| proto::note::NoteInclusionInBlockProof::missing_field("MerklePath"))?
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
    pub chain_tip: u32,
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
        let chain_tip = value.chain_tip;

        // Validate and convert block header
        let block_header = value
            .block_header
            .ok_or(RpcError::ExpectedDataMissing("BlockHeader".into()))?
            .try_into()?;

        let mmr_path = value
            .mmr_path
            .ok_or(RpcError::ExpectedDataMissing("MmrPath".into()))?
            .try_into()?;

        // Validate and convert account note inclusions into an (AccountId, Word) tuple
        let mut notes = vec![];
        for note in value.notes {
            let note_id: NoteId = note
                .note_id
                .ok_or(RpcError::ExpectedDataMissing("Notes.Id".into()))?
                .try_into()?;

            let inclusion_path = note
                .inclusion_path
                .ok_or(RpcError::ExpectedDataMissing("Notes.SparseMerklePath".into()))?
                .try_into()?;

            let metadata = note
                .metadata
                .ok_or(RpcError::ExpectedDataMissing("Metadata".into()))?
                .try_into()?;

            let committed_note = CommittedNote::new(
                note_id,
                u16::try_from(note.note_index_in_block).expect("note index out of range"),
                inclusion_path,
                metadata,
            );

            notes.push(committed_note);
        }

        Ok(NoteSyncInfo { chain_tip, block_header, mmr_path, notes })
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

/// Describes the possible responses from  the `GetNotesById` endpoint for a single note.
#[allow(clippy::large_enum_variant)]
pub enum FetchedNote {
    /// Details for a private note only include its [`NoteMetadata`] and [`NoteInclusionProof`].
    /// Other details needed to consume the note are expected to be stored locally, off-chain.
    Private(NoteId, NoteMetadata, NoteInclusionProof),
    /// Contains the full [`Note`] object alongside its [`NoteInclusionProof`].
    Public(Note, NoteInclusionProof),
}

impl FetchedNote {
    /// Returns the note's inclusion details.
    pub fn inclusion_proof(&self) -> &NoteInclusionProof {
        match self {
            FetchedNote::Private(_, _, inclusion_proof)
            | FetchedNote::Public(_, inclusion_proof) => inclusion_proof,
        }
    }

    /// Returns the note's metadata.
    pub fn metadata(&self) -> &NoteMetadata {
        match self {
            FetchedNote::Private(_, metadata, _) => metadata,
            FetchedNote::Public(note, _) => note.metadata(),
        }
    }

    /// Returns the note's ID.
    pub fn id(&self) -> NoteId {
        match self {
            FetchedNote::Private(id, ..) => *id,
            FetchedNote::Public(note, _) => note.id(),
        }
    }
}

impl TryFrom<proto::note::CommittedNote> for FetchedNote {
    type Error = RpcConversionError;

    fn try_from(value: proto::note::CommittedNote) -> Result<Self, Self::Error> {
        let inclusion_proof = value
            .inclusion_proof
            .ok_or_else(|| proto::note::CommittedNote::missing_field("InclusionProof"))?;

        let note_id: NoteId = inclusion_proof
            .note_id
            .ok_or_else(|| proto::note::CommittedNote::missing_field("InclusionProof.NoteId"))?
            .try_into()?;

        let inclusion_proof = NoteInclusionProof::try_from(inclusion_proof)?;

        let note = value.note.ok_or_else(|| proto::note::CommittedNote::missing_field("Note"))?;

        let metadata = note
            .metadata
            .ok_or_else(|| proto::note::CommittedNote::missing_field("Note.Metadata"))?
            .try_into()?;

        if let Some(detail_bytes) = note.details {
            let details = NoteDetails::read_from_bytes(&detail_bytes)?;
            let (assets, recipient) = details.into_parts();

            Ok(FetchedNote::Public(Note::new(assets, metadata, recipient), inclusion_proof))
        } else {
            Ok(FetchedNote::Private(note_id, metadata, inclusion_proof))
        }
    }
}
