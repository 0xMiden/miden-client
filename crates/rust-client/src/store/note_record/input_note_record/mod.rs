use alloc::string::ToString;

use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::note::{
    Note,
    NoteAssets,
    NoteDetails,
    NoteHeader,
    NoteId,
    NoteInclusionProof,
    NoteMetadata,
    Nullifier,
};
use miden_protocol::transaction::{InputNote, TransactionId};
use miden_protocol::utils::serde::{
    ByteReader,
    ByteWriter,
    Deserializable,
    DeserializationError,
    Serializable,
};

use super::NoteRecordError;

mod states;
pub use states::{
    CommittedNoteState,
    ConsumedAuthenticatedLocalNoteState,
    ConsumedExternalNoteState,
    ConsumedUnauthenticatedLocalNoteState,
    ExpectedNoteState,
    InputNoteState,
    InvalidNoteState,
    NoteSubmissionData,
    ProcessingAuthenticatedNoteState,
    ProcessingUnauthenticatedNoteState,
    UnverifiedNoteState,
};

// INPUT NOTE RECORD
// ================================================================================================

/// Represents a Note of which the Store can keep track and retrieve.
///
/// An [`InputNoteRecord`] always carries the note id and the note state. Full [`NoteDetails`]
/// are optional — when present, the record can be transformed into an [`InputNote`] and used
/// as input for transactions. Header-only records (no details) are used to track erased notes
/// that the client discovered but never had the full data for. In that case, the metadata is
/// carried by the state (see [`ConsumedExternalNoteState::metadata`]).
#[derive(Clone, Debug, PartialEq)]
pub struct InputNoteRecord {
    /// The note identifier. Always available.
    id: NoteId,
    /// Full note details (assets + recipient), if known. `None` for header-only records.
    details: Option<NoteDetails>,
    /// The timestamp at which the note was created. If it's not known, it will be None.
    created_at: Option<u64>,
    /// The state of the note, with specific fields for each one. Carries the note metadata.
    state: InputNoteState,
}

impl InputNoteRecord {
    /// Creates a new input note record with full details.
    pub fn new(
        details: NoteDetails,
        created_at: Option<u64>,
        state: InputNoteState,
    ) -> InputNoteRecord {
        InputNoteRecord {
            id: details.id(),
            details: Some(details),
            created_at,
            state,
        }
    }

    /// Creates a new input note record from only a note header. This is used for erased
    /// notes where the client doesn't have the full note data. The caller must ensure the
    /// provided `state` carries the note metadata (e.g. [`ConsumedExternalNoteState::metadata`]
    /// populated with `header.metadata()`).
    pub fn from_header(header: NoteHeader, state: InputNoteState) -> InputNoteRecord {
        InputNoteRecord {
            id: header.id(),
            details: None,
            created_at: None,
            state,
        }
    }

    // PUBLIC ACCESSORS
    // ================================================================================================

    /// Returns the input note ID.
    pub fn id(&self) -> NoteId {
        self.id
    }

    /// Returns the note's recipient digest, if full details are available.
    pub fn recipient(&self) -> Option<Word> {
        self.details.as_ref().map(|d| d.recipient().digest())
    }

    /// Returns the note's commitment, if the record contains the [`NoteMetadata`].
    pub fn commitment(&self) -> Option<Word> {
        self.metadata().map(|m| NoteHeader::new(self.id, m.clone()).to_commitment())
    }

    /// Returns the note's assets, if full details are available.
    pub fn assets(&self) -> Option<&NoteAssets> {
        self.details.as_ref().map(NoteDetails::assets)
    }

    /// Returns the timestamp in which the note record was created, if available.
    pub fn created_at(&self) -> Option<u64> {
        self.created_at
    }

    /// Returns the current note state.
    pub fn state(&self) -> &InputNoteState {
        &self.state
    }

    /// Returns the note metadata, if available. Single source of truth: delegates to
    /// [`InputNoteState::metadata`].
    pub fn metadata(&self) -> Option<&NoteMetadata> {
        self.state.metadata()
    }

    /// Returns the note nullifier, if full details are available.
    pub fn nullifier(&self) -> Option<Nullifier> {
        self.details.as_ref().map(NoteDetails::nullifier)
    }

    /// Returns the inclusion proof for the note.
    pub fn inclusion_proof(&self) -> Option<&NoteInclusionProof> {
        self.state.inclusion_proof()
    }

    /// Returns the note's full details, if available.
    pub fn details(&self) -> Option<&NoteDetails> {
        self.details.as_ref()
    }

    /// Returns true if this record has full note details.
    pub fn has_details(&self) -> bool {
        self.details.is_some()
    }

    /// If the note was consumed locally, it returns the corresponding transaction ID.
    /// Otherwise, returns `None`.
    pub fn consumer_transaction_id(&self) -> Option<&TransactionId> {
        self.state.consumer_transaction_id()
    }

    /// Returns the account ID that consumed this note, if available.
    ///
    /// This is available for notes in processing, consumed-local, or consumed-external
    /// states. For externally consumed notes, the account is only known when it is tracked
    /// by this client. Returns `None` for notes that haven't been submitted for consumption,
    /// invalid notes, or externally consumed notes where the consuming account is unknown.
    pub fn consumer_account(&self) -> Option<AccountId> {
        match &self.state {
            InputNoteState::ProcessingAuthenticated(s) => Some(s.submission_data.consumer_account),
            InputNoteState::ProcessingUnauthenticated(s) => {
                Some(s.submission_data.consumer_account)
            },
            InputNoteState::ConsumedAuthenticatedLocal(s) => {
                Some(s.submission_data.consumer_account)
            },
            InputNoteState::ConsumedUnauthenticatedLocal(s) => {
                Some(s.submission_data.consumer_account)
            },
            InputNoteState::ConsumedExternal(s) => s.consumer_account,
            _ => None,
        }
    }

    /// Returns true if the note is authenticated, meaning that it has the necessary inclusion
    /// proof and block header information to be considered valid.
    pub fn is_authenticated(&self) -> bool {
        matches!(
            self.state,
            InputNoteState::Committed { .. }
                | InputNoteState::ProcessingAuthenticated { .. }
                | InputNoteState::ConsumedAuthenticatedLocal { .. }
        )
    }

    /// Returns true if the note has been nullified on chain.
    pub fn is_consumed(&self) -> bool {
        matches!(
            self.state,
            InputNoteState::ConsumedExternal { .. }
                | InputNoteState::ConsumedAuthenticatedLocal { .. }
                | InputNoteState::ConsumedUnauthenticatedLocal { .. }
        )
    }

    /// Returns true if the note is currently being processed by a local transaction.
    pub fn is_processing(&self) -> bool {
        matches!(
            self.state,
            InputNoteState::ProcessingAuthenticated { .. }
                | InputNoteState::ProcessingUnauthenticated { .. }
        )
    }

    /// Returns true if the note is in a committed state (i.e. it has a valid inclusion proof but
    /// isn't consumed or being processed).
    pub fn is_committed(&self) -> bool {
        matches!(self.state, InputNoteState::Committed { .. })
    }

    /// Sets the consumed transaction order on the inner note state. No-op if the note is not in
    /// a consumed state.
    pub fn set_consumed_tx_order(&mut self, order: Option<u32>) {
        self.state.set_consumed_tx_order(order);
    }

    // TRANSITIONS
    // ================================================================================================

    /// Modifies the state of the note record to reflect that the it has received an inclusion
    /// proof. It is assumed to be unverified until the block header information is received.
    /// Returns `true` if the state was changed.
    pub(crate) fn inclusion_proof_received(
        &mut self,
        inclusion_proof: NoteInclusionProof,
        metadata: NoteMetadata,
    ) -> Result<bool, NoteRecordError> {
        let new_state = self.state.inclusion_proof_received(inclusion_proof, metadata)?;
        if let Some(new_state) = new_state {
            self.state = new_state;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Modifies the state of the note record to reflect that the it has received a block header.
    /// This will mark the note as verified or invalid, depending on the block header
    /// information and inclusion proof. Returns `true` if the state was changed.
    pub(crate) fn block_header_received(
        &mut self,
        block_header: &BlockHeader,
    ) -> Result<bool, NoteRecordError> {
        let new_state = self.state.block_header_received(self.id(), block_header)?;
        if let Some(new_state) = new_state {
            self.state = new_state;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Modifies the state of the note record to reflect that the note has been consumed by a
    /// transaction not submitted by this client. Returns `true` if the state was changed.
    ///
    /// `consumer_account` is `Some` when the consuming account is tracked by this client
    /// (derived from `sync_transactions` data). It is `None` for untracked accounts.
    ///
    /// Errors:
    /// - If the nullifier doesn't match the expected value.
    pub(crate) fn consumed_externally(
        &mut self,
        nullifier: Nullifier,
        nullifier_block_height: BlockNumber,
        consumer_account: Option<AccountId>,
    ) -> Result<bool, NoteRecordError> {
        // For header-only records we skip nullifier validation since we don't have the data
        // to compute it.
        if let Some(expected_nullifier) = self.nullifier() {
            if expected_nullifier != nullifier {
                return Err(NoteRecordError::StateTransitionError(
                    "Nullifier does not match the expected value".to_string(),
                ));
            }
        }

        let new_state = self.state.consumed_externally(nullifier_block_height, consumer_account)?;
        if let Some(new_state) = new_state {
            self.state = new_state;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Modifies the state of the note record to reflect that the client began processing the note
    /// to be consumed. Returns `true` if the state was changed.
    pub(crate) fn consumed_locally(
        &mut self,
        consumer_account: AccountId,
        consumer_transaction: TransactionId,
        current_timestamp: Option<u64>,
    ) -> Result<bool, NoteRecordError> {
        let new_state = self.state.consumed_locally(
            consumer_account,
            consumer_transaction,
            current_timestamp,
        )?;
        if let Some(new_state) = new_state {
            self.state = new_state;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Modifies the state of the note record to reflect that the transaction currently consuming
    /// the note was committed. Returns `true` if the state was changed.
    pub(crate) fn transaction_committed(
        &mut self,
        transaction_id: TransactionId,
        block_height: BlockNumber,
    ) -> Result<bool, NoteRecordError> {
        let new_state = self.state.transaction_committed(transaction_id, block_height)?;
        if let Some(new_state) = new_state {
            self.state = new_state;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// SERIALIZATION
// ================================================================================================

impl Serializable for InputNoteRecord {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.id.write_into(target);
        self.details.write_into(target);
        self.created_at.write_into(target);
        self.state.write_into(target);
    }
}

impl Deserializable for InputNoteRecord {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let id = NoteId::read_from(source)?;
        let details = Option::<NoteDetails>::read_from(source)?;
        let created_at = Option::<u64>::read_from(source)?;
        let state = InputNoteState::read_from(source)?;

        Ok(InputNoteRecord { id, details, created_at, state })
    }
}

// CONVERSION
// ================================================================================================

impl From<Note> for InputNoteRecord {
    fn from(value: Note) -> Self {
        let metadata = value.metadata().clone();
        let details: NoteDetails = value.into();
        Self {
            id: details.id(),
            details: Some(details),
            created_at: None,
            state: ExpectedNoteState {
                metadata: Some(metadata.clone()),
                after_block_num: BlockNumber::from(0),
                tag: Some(metadata.tag()),
            }
            .into(),
        }
    }
}

impl From<InputNote> for InputNoteRecord {
    fn from(value: InputNote) -> Self {
        match value {
            InputNote::Authenticated { note, proof } => {
                let metadata = note.metadata().clone();
                let details: NoteDetails = note.into();
                Self {
                    id: details.id(),
                    details: Some(details),
                    created_at: None,
                    state: UnverifiedNoteState { metadata, inclusion_proof: proof }.into(),
                }
            },
            InputNote::Unauthenticated { note } => note.into(),
        }
    }
}

impl TryInto<InputNote> for InputNoteRecord {
    type Error = NoteRecordError;

    fn try_into(self) -> Result<InputNote, Self::Error> {
        let details = self.details().ok_or_else(|| {
            NoteRecordError::ConversionError(
                "Input Note Record does not contain full details".to_string(),
            )
        })?;
        match (self.metadata(), self.inclusion_proof()) {
            (Some(metadata), Some(inclusion_proof)) => Ok(InputNote::authenticated(
                Note::new(details.assets().clone(), metadata.clone(), details.recipient().clone()),
                inclusion_proof.clone(),
            )),
            (Some(metadata), None) => Ok(InputNote::unauthenticated(Note::new(
                details.assets().clone(),
                metadata.clone(),
                details.recipient().clone(),
            ))),
            _ => Err(NoteRecordError::ConversionError(
                "Input Note Record does not contain metadata".to_string(),
            )),
        }
    }
}

impl TryInto<Note> for InputNoteRecord {
    type Error = NoteRecordError;

    fn try_into(self) -> Result<Note, Self::Error> {
        let details = self.details().ok_or_else(|| {
            NoteRecordError::ConversionError(
                "Input Note Record does not contain full details".to_string(),
            )
        })?;
        match self.metadata().cloned() {
            Some(metadata) => {
                Ok(Note::new(details.assets().clone(), metadata, details.recipient().clone()))
            },
            None => Err(NoteRecordError::ConversionError(
                "Input Note Record does not contain metadata".to_string(),
            )),
        }
    }
}

impl TryInto<Note> for &InputNoteRecord {
    type Error = NoteRecordError;

    fn try_into(self) -> Result<Note, Self::Error> {
        let details = self.details().ok_or_else(|| {
            NoteRecordError::ConversionError(
                "Input Note Record does not contain full details".to_string(),
            )
        })?;
        match self.metadata().cloned() {
            Some(metadata) => {
                Ok(Note::new(details.assets().clone(), metadata, details.recipient().clone()))
            },
            None => Err(NoteRecordError::ConversionError(
                "Input Note Record does not contain metadata".to_string(),
            )),
        }
    }
}

impl TryFrom<InputNoteRecord> for NoteDetails {
    type Error = NoteRecordError;

    fn try_from(value: InputNoteRecord) -> Result<Self, Self::Error> {
        value.details.ok_or_else(|| {
            NoteRecordError::ConversionError(
                "Input Note Record does not contain full details".to_string(),
            )
        })
    }
}
