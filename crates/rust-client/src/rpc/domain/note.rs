use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use miden_protocol::account::AccountId;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::MerklePath;
use miden_protocol::note::{
    Note,
    NoteAttachment,
    NoteAttachmentKind,
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

/// Represents a single block's worth of note sync data from the `SyncNotesResponse`.
#[derive(Debug, Clone)]
pub struct NoteSyncBlock {
    /// Block header containing the matching notes.
    pub block_header: BlockHeader,
    /// MMR path for verifying the block's inclusion in the MMR at `block_to`.
    pub mmr_path: MerklePath,
    /// Notes matching the requested tags in this block, keyed by note ID.
    pub notes: BTreeMap<NoteId, CommittedNote>,
}

/// Represents a `SyncNotesResponse` with fields converted into domain types.
///
/// The response may contain multiple blocks with matching notes. When `blocks` is empty,
/// no notes matched in the scanned range.
#[derive(Debug)]
pub struct NoteSyncInfo {
    /// The last block the node checked. Used as a cursor for pagination: if less than the
    /// requested range end (or chain tip), the client should continue from this block.
    pub block_to: BlockNumber,
    /// Blocks containing matching notes, ordered by block number ascending.
    /// May be empty if no notes matched in the range.
    pub blocks: Vec<NoteSyncBlock>,
}

impl TryFrom<proto::rpc::SyncNotesResponse> for NoteSyncInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc::SyncNotesResponse) -> Result<Self, Self::Error> {
        let pagination_info = value
            .pagination_info
            .ok_or(proto::rpc::SyncNotesResponse::missing_field(stringify!(pagination_info)))?;

        let block_to = BlockNumber::from(pagination_info.block_num);

        let blocks = value
            .blocks
            .into_iter()
            .map(|block| {
                let block_header = block
                    .block_header
                    .ok_or(proto::rpc::SyncNotesResponse::missing_field(stringify!(
                        blocks.block_header
                    )))?
                    .try_into()?;

                let mmr_path = block
                    .mmr_path
                    .ok_or(proto::rpc::SyncNotesResponse::missing_field(stringify!(
                        blocks.mmr_path
                    )))?
                    .try_into()?;

                let notes: BTreeMap<NoteId, CommittedNote> = block
                    .notes
                    .into_iter()
                    .map(|n| {
                        let note = CommittedNote::try_from(n)?;
                        Ok((*note.note_id(), note))
                    })
                    .collect::<Result<_, RpcConversionError>>()?;

                Ok(NoteSyncBlock { block_header, mmr_path, notes })
            })
            .collect::<Result<Vec<_>, RpcError>>()?;

        Ok(NoteSyncInfo { block_to, blocks })
    }
}

// COMMITTED NOTE
// ================================================================================================

/// The metadata state of a committed note.
///
/// The sync response provides header fields (sender, type, tag, attachment kind) but not the
/// actual attachment data. For notes without attachments, full [`NoteMetadata`] can be
/// constructed directly. For notes with attachments, only the header fields are available
/// until the full metadata is fetched via `GetNotesById`.
#[derive(Debug, Clone)]
pub enum CommittedNoteMetadata {
    /// Full metadata is available (note has no attachment, or attachment was already fetched).
    Full(NoteMetadata),
    /// Only the header fields are available; the attachment data has not been fetched yet.
    Header {
        sender: AccountId,
        note_type: NoteType,
        tag: NoteTag,
    },
}

impl CommittedNoteMetadata {
    /// Returns the note type, available in both variants.
    pub fn note_type(&self) -> NoteType {
        match self {
            Self::Full(m) => m.note_type(),
            Self::Header { note_type, .. } => *note_type,
        }
    }

    /// Returns the note tag, available in both variants.
    pub fn tag(&self) -> NoteTag {
        match self {
            Self::Full(m) => m.tag(),
            Self::Header { tag, .. } => *tag,
        }
    }

    /// Returns the sender, available in both variants.
    pub fn sender(&self) -> AccountId {
        match self {
            Self::Full(m) => m.sender(),
            Self::Header { sender, .. } => *sender,
        }
    }

    /// Returns the full metadata if available.
    pub fn metadata(&self) -> Option<&NoteMetadata> {
        match self {
            Self::Full(m) => Some(m),
            Self::Header { .. } => None,
        }
    }
}

/// Represents a committed note, returned as part of a `SyncNotesResponse`.
///
/// The sync response provides a [`NoteMetadataHeader`](crate::note::NoteMetadataHeader) but not the
/// actual attachment data. For notes without attachments, full [`NoteMetadata`] is available
/// immediately. For notes with attachments, the metadata starts as
/// [`CommittedNoteMetadata::Header`] until the full data is fetched via `GetNotesById`.
#[derive(Debug, Clone)]
pub struct CommittedNote {
    /// Note ID of the committed note.
    note_id: NoteId,
    /// Note metadata — either full or header-only depending on whether the note has an
    /// attachment that hasn't been fetched yet.
    metadata: CommittedNoteMetadata,
    /// Inclusion proof for the note in the block.
    inclusion_proof: NoteInclusionProof,
}

impl CommittedNote {
    pub fn new(
        note_id: NoteId,
        metadata: CommittedNoteMetadata,
        inclusion_proof: NoteInclusionProof,
    ) -> Self {
        Self { note_id, metadata, inclusion_proof }
    }

    pub fn note_id(&self) -> &NoteId {
        &self.note_id
    }

    pub fn note_type(&self) -> NoteType {
        self.metadata.note_type()
    }

    pub fn tag(&self) -> NoteTag {
        self.metadata.tag()
    }

    /// Returns the full note metadata, or `None` if only the header is available.
    pub fn metadata(&self) -> Option<&NoteMetadata> {
        self.metadata.metadata()
    }

    /// Returns the committed note metadata enum.
    pub fn committed_metadata(&self) -> &CommittedNoteMetadata {
        &self.metadata
    }

    /// Sets the full metadata, promoting from `Header` to `Full`.
    ///
    /// Used after fetching attachment data via `GetNotesById` for notes whose sync
    /// response only included header fields.
    pub fn set_metadata(&mut self, metadata: NoteMetadata) {
        self.metadata = CommittedNoteMetadata::Full(metadata);
    }

    pub fn inclusion_proof(&self) -> &NoteInclusionProof {
        &self.inclusion_proof
    }
}

impl TryFrom<proto::note::NoteSyncRecord> for CommittedNote {
    type Error = RpcConversionError;

    fn try_from(note: proto::note::NoteSyncRecord) -> Result<Self, Self::Error> {
        let proto_header = note.metadata_header.ok_or(
            proto::rpc::SyncNotesResponse::missing_field(stringify!(notes.metadata_header)),
        )?;

        let sender = proto_header
            .sender
            .ok_or(proto::rpc::SyncNotesResponse::missing_field(stringify!(
                notes.metadata_header.sender
            )))?
            .try_into()?;
        let note_type =
            NoteType::try_from(u64::try_from(proto_header.note_type).expect("invalid note type"))?;
        let tag = NoteTag::new(proto_header.tag);
        let attachment_kind = u8::try_from(proto_header.attachment_kind)
            .ok()
            .and_then(|kind| NoteAttachmentKind::try_from(kind).ok())
            .unwrap_or_default();

        let metadata = if attachment_kind == NoteAttachmentKind::None {
            CommittedNoteMetadata::Full(NoteMetadata::new(sender, note_type).with_tag(tag))
        } else {
            CommittedNoteMetadata::Header { sender, note_type, tag }
        };

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

        Ok(CommittedNote::new(note_id, metadata, inclusion_proof))
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
