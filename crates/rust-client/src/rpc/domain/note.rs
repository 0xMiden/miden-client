use alloc::vec::Vec;

use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::MerklePath;
use miden_protocol::note::{
    Note,
    NoteAttachment,
    NoteDetails,
    NoteHeader,
    NoteId,
    NoteInclusionProof,
    NoteMetadata,
    NoteScript,
    NoteTag,
    NoteType,
};
use miden_protocol::{MastForest, MastNodeId, Word};
use miden_tx::utils::serde::Deserializable;

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
        let word =
            Word::try_from(value.id.ok_or(proto::note::NoteId::missing_field(stringify!(id)))?)?;
        Ok(Self::from_raw(word))
    }
}

impl TryFrom<proto::note::NoteMetadata> for NoteMetadata {
    type Error = RpcConversionError;

    fn try_from(value: proto::note::NoteMetadata) -> Result<Self, Self::Error> {
        let sender = value
            .sender
            .ok_or_else(|| proto::note::NoteMetadata::missing_field(stringify!(sender)))?
            .try_into()?;
        let note_type =
            NoteType::try_from(u64::try_from(value.note_type).expect("invalid note type"))?;
        let tag = NoteTag::new(value.tag);

        // Deserialize attachment if present
        let attachment = if value.attachment.is_empty() {
            NoteAttachment::default()
        } else {
            NoteAttachment::read_from_bytes(&value.attachment)
                .map_err(RpcConversionError::DeserializationError)?
        };

        Ok(NoteMetadata::new(sender, note_type).with_tag(tag).with_attachment(attachment))
    }
}

impl From<NoteMetadata> for proto::note::NoteMetadata {
    fn from(value: NoteMetadata) -> Self {
        use miden_tx::utils::serde::Serializable;
        proto::note::NoteMetadata {
            sender: Some(value.sender().into()),
            note_type: value.note_type() as i32,
            tag: value.tag().as_u32(),
            attachment: value.attachment().to_bytes(),
        }
    }
}

impl TryFrom<proto::note::NoteHeader> for NoteHeader {
    type Error = RpcConversionError;

    fn try_from(value: proto::note::NoteHeader) -> Result<Self, Self::Error> {
        let note_id = value
            .note_id
            .ok_or(proto::note::NoteHeader::missing_field(stringify!(note_id)))?
            .try_into()?;
        let metadata = value
            .metadata
            .ok_or(proto::note::NoteHeader::missing_field(stringify!(metadata)))?
            .try_into()?;
        Ok(NoteHeader::new(note_id, metadata))
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

impl TryFrom<proto::rpc::SyncNotesResponse> for NoteSyncInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc::SyncNotesResponse) -> Result<Self, Self::Error> {
        let chain_tip = value
            .pagination_info
            .ok_or(proto::rpc::SyncNotesResponse::missing_field(stringify!(pagination_info)))?
            .chain_tip;

        // Validate and convert block header
        let block_header = value
            .block_header
            .ok_or(proto::rpc::SyncNotesResponse::missing_field(stringify!(block_header)))?
            .try_into()?;

        let mmr_path = value
            .mmr_path
            .ok_or(proto::rpc::SyncNotesResponse::missing_field(stringify!(mmr_path)))?
            .try_into()?;

        let mut notes = vec![];
        for note in value.notes {
            let metadata_header = note.metadata_header.ok_or(
                proto::rpc::SyncNotesResponse::missing_field(stringify!(notes.metadata_header)),
            )?;

            let note_type = NoteType::try_from(
                u64::try_from(metadata_header.note_type).expect("invalid note type"),
            )?;
            let tag = NoteTag::new(metadata_header.tag);

            let proto_inclusion_proof = note.inclusion_proof.ok_or(
                proto::rpc::SyncNotesResponse::missing_field(stringify!(notes.inclusion_proof)),
            )?;

            let note_id: NoteId = proto_inclusion_proof
                .note_id
                .ok_or(proto::rpc::SyncNotesResponse::missing_field(stringify!(
                    notes.inclusion_proof.note_id
                )))?
                .try_into()?;

            let inclusion_proof: NoteInclusionProof = proto_inclusion_proof.try_into()?;

            notes.push(CommittedNote::new(note_id, note_type, tag, inclusion_proof));
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

/// Represents a committed note, returned as part of a `SyncNotesResponse`.
///
/// Contains only the note type and tag from the metadata header (fixed-size), rather than full
/// [`NoteMetadata`], since the sync response no longer includes attachment data. Clients needing
/// full metadata should source it from the local store (for tracked notes) or call `GetNotesById`
/// (for public notes).
#[derive(Debug, Clone)]
pub struct CommittedNote {
    /// Note ID of the committed note.
    note_id: NoteId,
    /// The note type (public, private, etc.).
    note_type: NoteType,
    /// The note tag used for filtering.
    tag: NoteTag,
    /// Inclusion proof for the note in the block.
    inclusion_proof: NoteInclusionProof,
}

impl CommittedNote {
    pub fn new(
        note_id: NoteId,
        note_type: NoteType,
        tag: NoteTag,
        inclusion_proof: NoteInclusionProof,
    ) -> Self {
        Self { note_id, note_type, tag, inclusion_proof }
    }

    pub fn note_id(&self) -> &NoteId {
        &self.note_id
    }

    pub fn note_type(&self) -> NoteType {
        self.note_type
    }

    pub fn tag(&self) -> NoteTag {
        self.tag
    }

    pub fn inclusion_proof(&self) -> &NoteInclusionProof {
        &self.inclusion_proof
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
