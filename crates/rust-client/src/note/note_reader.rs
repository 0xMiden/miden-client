//! Provides lazy readers over input and output notes.

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;

use crate::ClientError;
use crate::store::{
    InputNoteRecord, InputNoteState, NoteFilter, OutputNoteRecord, OutputNoteState, Store,
};

/// Selects which kind of notes a [`NoteReader`] iterates over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteReaderSource {
    Input,
    Output,
}

/// A note returned by a [`NoteReader`].
#[derive(Clone, Debug, PartialEq)]
pub enum NoteRecord {
    Input(InputNoteRecord),
    Output(OutputNoteRecord),
}

impl NoteRecord {
    /// Returns the note ID regardless of whether it is an input or output note.
    pub fn id(&self) -> crate::note::NoteId {
        match self {
            Self::Input(note) => note.id(),
            Self::Output(note) => note.id(),
        }
    }
}

/// A lazy reader over notes managed by the client's store.
///
/// `NoteReader` can iterate over either input notes or output notes. For consumed input notes it
/// uses store-level offset queries so records are streamed one by one in consumption order.
/// Other input filters and all output-note queries are materialized once and then iterated in
/// memory.
pub struct NoteReader {
    store: Arc<dyn Store>,
    source: NoteReaderSource,
    filter: NoteFilter,
    consumer: Option<AccountId>,
    block_range: Option<(BlockNumber, BlockNumber)>,
    offset: u32,
    buffered_notes: Option<Vec<NoteRecord>>,
}

impl NoteReader {
    /// Creates a new reader for the requested note source.
    pub fn new(store: Arc<dyn Store>, source: NoteReaderSource) -> Self {
        let filter = match source {
            NoteReaderSource::Input => NoteFilter::Consumed,
            NoteReaderSource::Output => NoteFilter::All,
        };

        Self {
            store,
            source,
            filter,
            consumer: None,
            block_range: None,
            offset: 0,
            buffered_notes: None,
        }
    }

    /// Filters the reader by note state.
    #[must_use]
    pub fn with_filter(mut self, filter: NoteFilter) -> Self {
        self.filter = filter;
        self.reset();
        self
    }

    /// Filters input-note readers by consumer account ID.
    ///
    /// This filter is ignored for output-note readers.
    #[must_use]
    pub fn for_consumer(mut self, account_id: AccountId) -> Self {
        self.consumer = Some(account_id);
        self.reset();
        self
    }

    /// Restricts iteration to notes whose relevant block number falls within the given range.
    ///
    /// For consumed input notes and consumed output notes this uses the nullifier block height.
    /// For committed notes it uses the inclusion-proof block height. For expected output notes it
    /// uses `expected_height`. For expected or processing unauthenticated input notes it uses
    /// `after_block_num`. Notes without an applicable block number are excluded when a block range
    /// is set.
    #[must_use]
    pub fn in_block_range(mut self, from: BlockNumber, to: BlockNumber) -> Self {
        self.block_range = Some((from, to));
        self.reset();
        self
    }

    /// Resets the reader to the beginning.
    pub fn reset(&mut self) {
        self.offset = 0;
        self.buffered_notes = None;
    }

    /// Returns the next matching note, or `None` when all matching notes have been returned.
    pub async fn next(&mut self) -> Result<Option<NoteRecord>, ClientError> {
        if self.should_stream_input_notes() {
            return self.next_streamed_input_note().await;
        }

        if self.buffered_notes.is_none() {
            self.buffered_notes = Some(self.load_buffered_notes().await?);
        }

        let note = self
            .buffered_notes
            .as_ref()
            .and_then(|notes| notes.get(self.offset as usize))
            .cloned();

        if note.is_some() {
            self.offset += 1;
        }

        Ok(note)
    }

    fn should_stream_input_notes(&self) -> bool {
        self.source == NoteReaderSource::Input && matches!(self.filter, NoteFilter::Consumed)
    }

    async fn next_streamed_input_note(&mut self) -> Result<Option<NoteRecord>, ClientError> {
        let (block_start, block_end) = match self.block_range {
            Some((from, to)) => (Some(from), Some(to)),
            None => (None, None),
        };

        let note = self
            .store
            .get_input_note_by_offset(
                self.filter.clone(),
                self.consumer,
                block_start,
                block_end,
                self.offset,
            )
            .await
            .map_err(ClientError::StoreError)?;

        if note.is_some() {
            self.offset += 1;
        }

        Ok(note.map(NoteRecord::Input))
    }

    async fn load_buffered_notes(&self) -> Result<Vec<NoteRecord>, ClientError> {
        match self.source {
            NoteReaderSource::Input => self.load_input_notes().await,
            NoteReaderSource::Output => self.load_output_notes().await,
        }
    }

    async fn load_input_notes(&self) -> Result<Vec<NoteRecord>, ClientError> {
        let mut notes = self
            .store
            .get_input_notes(self.filter.clone())
            .await
            .map_err(ClientError::StoreError)?;

        notes.retain(|note| self.matches_input_note(note));

        if !matches!(self.filter, NoteFilter::Consumed) {
            notes.sort_by_key(|note| input_note_sort_key(note));
        }

        Ok(notes.into_iter().map(NoteRecord::Input).collect())
    }

    async fn load_output_notes(&self) -> Result<Vec<NoteRecord>, ClientError> {
        let mut notes = self
            .store
            .get_output_notes(self.filter.clone())
            .await
            .map_err(ClientError::StoreError)?;

        notes.retain(|note| self.matches_output_note(note));
        notes.sort_by_key(|note| output_note_sort_key(note));

        Ok(notes.into_iter().map(NoteRecord::Output).collect())
    }

    fn matches_input_note(&self, note: &InputNoteRecord) -> bool {
        if let Some(consumer) = self.consumer
            && note.consumer_account() != Some(consumer)
        {
            return false;
        }

        block_matches_range(input_note_block_num(note), self.block_range)
    }

    fn matches_output_note(&self, note: &OutputNoteRecord) -> bool {
        block_matches_range(output_note_block_num(note), self.block_range)
    }
}

/// Backwards-compatible reader for input notes.
pub struct InputNoteReader(NoteReader);

impl InputNoteReader {
    /// Creates a new `InputNoteReader` that iterates over consumed input notes.
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self(NoteReader::new(store, NoteReaderSource::Input))
    }

    /// Filters the reader by input-note state.
    #[must_use]
    pub fn with_filter(mut self, filter: NoteFilter) -> Self {
        self.0 = self.0.with_filter(filter);
        self
    }

    /// Filters notes by consumer account ID.
    #[must_use]
    pub fn for_consumer(mut self, account_id: AccountId) -> Self {
        self.0 = self.0.for_consumer(account_id);
        self
    }

    /// Restricts iteration to notes whose relevant block number falls within the given range.
    #[must_use]
    pub fn in_block_range(mut self, from: BlockNumber, to: BlockNumber) -> Self {
        self.0 = self.0.in_block_range(from, to);
        self
    }

    /// Resets the iterator to the beginning.
    pub fn reset(&mut self) {
        self.0.reset();
    }

    /// Returns the next matching input note.
    pub async fn next(&mut self) -> Result<Option<InputNoteRecord>, ClientError> {
        match self.0.next().await? {
            Some(NoteRecord::Input(note)) => Ok(Some(note)),
            Some(NoteRecord::Output(_)) => unreachable!("input readers never yield output notes"),
            None => Ok(None),
        }
    }
}

/// Reader for output notes.
pub struct OutputNoteReader(NoteReader);

impl OutputNoteReader {
    /// Creates a new `OutputNoteReader` that iterates over all output notes.
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self(NoteReader::new(store, NoteReaderSource::Output))
    }

    /// Filters the reader by output-note state.
    #[must_use]
    pub fn with_filter(mut self, filter: NoteFilter) -> Self {
        self.0 = self.0.with_filter(filter);
        self
    }

    /// Restricts iteration to notes whose relevant block number falls within the given range.
    #[must_use]
    pub fn in_block_range(mut self, from: BlockNumber, to: BlockNumber) -> Self {
        self.0 = self.0.in_block_range(from, to);
        self
    }

    /// Resets the iterator to the beginning.
    pub fn reset(&mut self) {
        self.0.reset();
    }

    /// Returns the next matching output note.
    pub async fn next(&mut self) -> Result<Option<OutputNoteRecord>, ClientError> {
        match self.0.next().await? {
            Some(NoteRecord::Output(note)) => Ok(Some(note)),
            Some(NoteRecord::Input(_)) => unreachable!("output readers never yield input notes"),
            None => Ok(None),
        }
    }
}

fn block_matches_range(
    block_num: Option<BlockNumber>,
    block_range: Option<(BlockNumber, BlockNumber)>,
) -> bool {
    match block_range {
        Some((from, to)) => block_num.is_some_and(|block_num| block_num >= from && block_num <= to),
        None => true,
    }
}

fn input_note_block_num(note: &InputNoteRecord) -> Option<BlockNumber> {
    match note.state() {
        InputNoteState::Expected(state) => Some(state.after_block_num),
        InputNoteState::Unverified(state) => Some(state.inclusion_proof.location().block_num()),
        InputNoteState::Committed(state) => Some(state.inclusion_proof.location().block_num()),
        InputNoteState::Invalid(state) => {
            Some(state.invalid_inclusion_proof.location().block_num())
        },
        InputNoteState::ProcessingAuthenticated(state) => {
            Some(state.inclusion_proof.location().block_num())
        },
        InputNoteState::ProcessingUnauthenticated(state) => Some(state.after_block_num),
        InputNoteState::ConsumedAuthenticatedLocal(state) => Some(state.nullifier_block_height),
        InputNoteState::ConsumedUnauthenticatedLocal(state) => Some(state.nullifier_block_height),
        InputNoteState::ConsumedExternal(state) => Some(state.nullifier_block_height),
    }
}

fn output_note_block_num(note: &OutputNoteRecord) -> Option<BlockNumber> {
    match note.state() {
        OutputNoteState::ExpectedPartial | OutputNoteState::ExpectedFull { .. } => {
            Some(note.expected_height())
        },
        OutputNoteState::CommittedPartial { inclusion_proof }
        | OutputNoteState::CommittedFull { inclusion_proof, .. } => {
            Some(inclusion_proof.location().block_num())
        },
        OutputNoteState::Consumed { block_height, .. } => Some(*block_height),
    }
}

fn input_note_sort_key(note: &InputNoteRecord) -> (bool, u32, String) {
    let block_num = input_note_block_num(note).map(|block_num| block_num.as_u32());
    (
        block_num.is_none(),
        block_num.unwrap_or_default(),
        note.id().as_word().to_string(),
    )
}

fn output_note_sort_key(note: &OutputNoteRecord) -> (bool, u32, String) {
    let block_num = output_note_block_num(note).map(|block_num| block_num.as_u32());
    (
        block_num.is_none(),
        block_num.unwrap_or_default(),
        note.id().as_word().to_string(),
    )
}
