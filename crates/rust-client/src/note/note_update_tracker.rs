use alloc::collections::BTreeMap;

use miden_protocol::account::AccountId;
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::note::{Note, NoteHeader, NoteId, NoteInclusionProof, Nullifier};
use miden_standards::note::NetworkAccountTarget;
use miden_tx::utils::serde::{
    ByteReader,
    ByteWriter,
    Deserializable,
    DeserializationError,
    Serializable,
};

use crate::ClientError;
use crate::rpc::domain::note::CommittedNote;
use crate::store::{InputNoteRecord, OutputNoteRecord};
use crate::transaction::{TransactionRecord, TransactionStatus};

// NOTE CONSUMPTION
// ================================================================================================

/// A note consumption event observed on chain.
pub struct NoteConsumption {
    /// The nullifier of the consumed note.
    pub nullifier: Nullifier,
    /// The block number at which the note consumption was registered on chain.
    pub block_num: BlockNumber,
    /// The account ID of the consumer of the note. Will be set if the note was consumed by a
    /// transaction submitted outside this client by an account that is tracked locally.
    /// Otherwise, it will be `None`.
    pub external_consumer: Option<AccountId>,
}

// NOTE UPDATE
// ================================================================================================

/// Represents the possible types of updates that can be applied to a note in a
/// [`NoteUpdateTracker`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NoteUpdateType {
    /// Indicates that the note was already tracked but it was not updated.
    None = 0,
    /// Indicates that the note is new and should be inserted in the store.
    Insert = 1,
    /// Indicates that the note was already tracked and should be updated.
    Update = 2,
}

impl TryFrom<u8> for NoteUpdateType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NoteUpdateType::None),
            1 => Ok(NoteUpdateType::Insert),
            2 => Ok(NoteUpdateType::Update),
            other => Err(other),
        }
    }
}

/// Represents the possible states of an input note record in a [`NoteUpdateTracker`].
#[derive(Clone, Debug, PartialEq)]
pub struct InputNoteUpdate {
    /// Input note being updated.
    note: InputNoteRecord,
    /// Type of the note update.
    update_type: NoteUpdateType,
}

impl InputNoteUpdate {
    /// Creates a new [`InputNoteUpdate`] with the provided note with a `None` update type.
    fn new_none(note: InputNoteRecord) -> Self {
        Self { note, update_type: NoteUpdateType::None }
    }

    /// Creates a new [`InputNoteUpdate`] with the provided note with an `Insert` update type.
    fn new_insert(note: InputNoteRecord) -> Self {
        Self {
            note,
            update_type: NoteUpdateType::Insert,
        }
    }

    /// Creates a new [`InputNoteUpdate`] with the provided note with an `Update` update type.
    fn new_update(note: InputNoteRecord) -> Self {
        Self {
            note,
            update_type: NoteUpdateType::Update,
        }
    }

    /// Returns a reference the inner note record.
    pub fn inner(&self) -> &InputNoteRecord {
        &self.note
    }

    /// Returns a mutable reference to the inner note record. If the update type is `None` or
    /// `Update`, it will be set to `Update`.
    fn inner_mut(&mut self) -> &mut InputNoteRecord {
        self.update_type = match self.update_type {
            NoteUpdateType::None | NoteUpdateType::Update => NoteUpdateType::Update,
            NoteUpdateType::Insert => NoteUpdateType::Insert,
        };

        &mut self.note
    }

    /// Returns the type of the note update.
    pub fn update_type(&self) -> &NoteUpdateType {
        &self.update_type
    }

    /// Returns the identifier of the inner note.
    pub fn id(&self) -> NoteId {
        self.note.id()
    }

    /// Returns the per-account position of the consuming transaction within the account's
    /// execution chain for the block. `None` for non-consumed notes or when the order has not
    /// been determined yet.
    pub fn consumed_tx_order(&self) -> Option<u32> {
        self.note.state().consumed_tx_order()
    }
}

/// Represents the possible states of an output note record in a [`NoteUpdateTracker`].
#[derive(Clone, Debug, PartialEq)]
pub struct OutputNoteUpdate {
    /// Output note being updated.
    note: OutputNoteRecord,
    /// Type of the note update.
    update_type: NoteUpdateType,
}

impl OutputNoteUpdate {
    /// Creates a new [`OutputNoteUpdate`] with the provided note with a `None` update type.
    fn new_none(note: OutputNoteRecord) -> Self {
        Self { note, update_type: NoteUpdateType::None }
    }

    /// Creates a new [`OutputNoteUpdate`] with the provided note with an `Insert` update type.
    fn new_insert(note: OutputNoteRecord) -> Self {
        Self {
            note,
            update_type: NoteUpdateType::Insert,
        }
    }

    /// Creates a new [`OutputNoteUpdate`] with the provided note with an `Update` update type.
    fn new_update(note: OutputNoteRecord) -> Self {
        Self {
            note,
            update_type: NoteUpdateType::Update,
        }
    }

    /// Returns a reference the inner note record.
    pub fn inner(&self) -> &OutputNoteRecord {
        &self.note
    }

    /// Returns a mutable reference to the inner note record. If the update type is `None` or
    /// `Update`, it will be set to `Update`.
    fn inner_mut(&mut self) -> &mut OutputNoteRecord {
        self.update_type = match self.update_type {
            NoteUpdateType::None | NoteUpdateType::Update => NoteUpdateType::Update,
            NoteUpdateType::Insert => NoteUpdateType::Insert,
        };

        &mut self.note
    }

    /// Returns the type of the note update.
    pub fn update_type(&self) -> &NoteUpdateType {
        &self.update_type
    }

    /// Returns the identifier of the inner note.
    pub fn id(&self) -> NoteId {
        self.note.id()
    }
}

// NOTE UPDATE TRACKER
// ================================================================================================

/// Contains note changes to apply to the store.
///
/// This includes new notes that have been created and existing notes that have been updated. The
/// tracker also lets state changes be applied to the contained notes, this allows for already
/// updated notes to be further updated as new information is received.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NoteUpdateTracker {
    /// A map of new and updated input note records to be upserted in the store.
    input_notes: BTreeMap<NoteId, InputNoteUpdate>,
    /// A map of updated output note records to be upserted in the store.
    output_notes: BTreeMap<NoteId, OutputNoteUpdate>,
    /// Fast lookup map from nullifier to input note id.
    input_notes_by_nullifier: BTreeMap<Nullifier, NoteId>,
    /// Fast lookup map from nullifier to output note id.
    output_notes_by_nullifier: BTreeMap<Nullifier, NoteId>,
    /// Map from nullifier to its per-account position in the consuming transaction order.
    /// Nullifiers from the same account are in execution order; ordering across different
    /// accounts is not guaranteed.
    nullifier_order: BTreeMap<Nullifier, u32>,
}

impl NoteUpdateTracker {
    /// Creates a [`NoteUpdateTracker`] with already-tracked notes.
    pub fn new(
        input_notes: impl IntoIterator<Item = InputNoteRecord>,
        output_notes: impl IntoIterator<Item = OutputNoteRecord>,
    ) -> Self {
        let mut tracker = Self::default();
        for note in input_notes {
            tracker.insert_input_note(note, NoteUpdateType::None);
        }
        for note in output_notes {
            tracker.insert_output_note(note, NoteUpdateType::None);
        }

        tracker
    }

    /// Creates a [`NoteUpdateTracker`] for updates related to transactions.
    ///
    /// A transaction can:
    ///
    /// - Create input notes
    /// - Update existing input notes (by consuming them)
    /// - Create output notes
    pub fn for_transaction_updates(
        new_input_notes: impl IntoIterator<Item = InputNoteRecord>,
        updated_input_notes: impl IntoIterator<Item = InputNoteRecord>,
        new_output_notes: impl IntoIterator<Item = OutputNoteRecord>,
    ) -> Self {
        let mut tracker = Self::default();

        for note in new_input_notes {
            tracker.insert_input_note(note, NoteUpdateType::Insert);
        }

        for note in updated_input_notes {
            tracker.insert_input_note(note, NoteUpdateType::Update);
        }

        for note in new_output_notes {
            tracker.insert_output_note(note, NoteUpdateType::Insert);
        }

        tracker
    }

    // GETTERS
    // --------------------------------------------------------------------------------------------

    /// Returns all input note records that have been updated.
    ///
    /// This may include:
    /// - New notes that have been created that should be inserted.
    /// - Existing tracked notes that should be updated.
    pub fn updated_input_notes(&self) -> impl Iterator<Item = &InputNoteUpdate> {
        self.input_notes.values().filter(|note| {
            matches!(note.update_type, NoteUpdateType::Insert | NoteUpdateType::Update)
        })
    }

    /// Returns all output note records that have been updated.
    ///
    /// This may include:
    /// - New notes that have been created that should be inserted.
    /// - Existing tracked notes that should be updated.
    pub fn updated_output_notes(&self) -> impl Iterator<Item = &OutputNoteUpdate> {
        self.output_notes.values().filter(|note| {
            matches!(note.update_type, NoteUpdateType::Insert | NoteUpdateType::Update)
        })
    }

    /// Returns whether no new note-related information has been retrieved.
    pub fn is_empty(&self) -> bool {
        self.input_notes.is_empty() && self.output_notes.is_empty()
    }

    /// Returns input and output note unspent nullifiers.
    pub fn unspent_nullifiers(&self) -> impl Iterator<Item = Nullifier> {
        let input_note_unspent_nullifiers = self
            .input_notes
            .values()
            .filter(|note| !note.inner().is_consumed())
            .map(|note| note.inner().nullifier());

        let output_note_unspent_nullifiers = self
            .output_notes
            .values()
            .filter(|note| !note.inner().is_consumed())
            .filter_map(|note| note.inner().nullifier());

        input_note_unspent_nullifiers.chain(output_note_unspent_nullifiers)
    }

    /// Appends nullifiers to the per-account ordered nullifier list.
    ///
    /// Nullifiers from the same account must be in execution order; ordering across different
    /// accounts is not guaranteed.
    pub fn extend_nullifiers(&mut self, nullifiers: impl IntoIterator<Item = Nullifier>) {
        for nullifier in nullifiers {
            let next_pos =
                u32::try_from(self.nullifier_order.len()).expect("nullifier count exceeds u32");
            self.nullifier_order.entry(nullifier).or_insert(next_pos);
        }
    }

    // UPDATE METHODS
    // --------------------------------------------------------------------------------------------

    /// Inserts the new public note data into the tracker. This method doesn't check the relevance
    /// of the note, so it should only be used for notes that are guaranteed to be relevant to the
    /// client.
    pub(crate) fn apply_new_public_note(
        &mut self,
        mut public_note_data: InputNoteRecord,
        block_header: &BlockHeader,
    ) -> Result<(), ClientError> {
        public_note_data.block_header_received(block_header)?;
        self.insert_input_note(public_note_data, NoteUpdateType::Insert);

        Ok(())
    }

    /// Applies the necessary state transitions to the [`NoteUpdateTracker`] when a note is
    /// committed in a block and returns whether the committed note is tracked as input note.
    pub(crate) fn apply_committed_note_state_transitions(
        &mut self,
        committed_note: &CommittedNote,
        block_header: &BlockHeader,
    ) -> Result<bool, ClientError> {
        let inclusion_proof = committed_note.inclusion_proof().clone();

        let is_tracked_as_input_note =
            if let Some(input_note_record) = self.get_input_note_by_id(*committed_note.note_id()) {
                let metadata = *committed_note.metadata();
                input_note_record.inclusion_proof_received(inclusion_proof.clone(), metadata)?;
                input_note_record.block_header_received(block_header)?;

                true
            } else {
                false
            };

        self.try_commit_output_note(*committed_note.note_id(), inclusion_proof)?;

        Ok(is_tracked_as_input_note)
    }

    /// Applies inclusion proofs from the transaction sync response to tracked output notes.
    ///
    /// This transitions output notes from `Expected` to `Committed` state using the
    /// inclusion proofs returned by `SyncTransactions`.
    pub(crate) fn apply_output_note_inclusion_proofs(
        &mut self,
        committed_notes: &[CommittedNote],
    ) -> Result<(), ClientError> {
        for committed_note in committed_notes {
            self.try_commit_output_note(
                *committed_note.note_id(),
                committed_note.inclusion_proof().clone(),
            )?;
        }
        Ok(())
    }

    /// Marks an erased note as consumed.
    ///
    /// This handles notes that were erased due to same-batch note erasure: the note was
    /// created and consumed within the same batch, so it never appeared in the block body.
    /// The `block_num` is the block in which the creating transaction was committed.
    ///
    /// The consumer account id is not derivable from a [`NoteHeader`] alone: attachment
    /// content lives on `NoteAttachments`, which the erased-note RPC stream does not deliver.
    /// Any input record for the erased note is marked consumed with an unknown consumer.
    pub(crate) fn mark_erased_note_as_consumed(
        &mut self,
        note_header: &NoteHeader,
        block_num: BlockNumber,
    ) -> Result<(), ClientError> {
        let note_id = note_header.id();

        if let Some(output_note) = self.get_output_note_by_id(note_id)
            && !output_note.is_consumed()
            && !output_note.is_committed()
            && let Some(nullifier) = output_note.nullifier()
        {
            output_note.nullifier_received(nullifier, block_num)?;
        }

        if let Some(input_note_update) = self.input_notes.get_mut(&note_id)
            && !input_note_update.inner().is_consumed()
        {
            let nullifier = input_note_update.inner().nullifier();
            let consumer_account =
                NetworkAccountTarget::try_from(input_note_update.inner().attachments())
                    .ok()
                    .map(|target| target.target_id());
            input_note_update.inner_mut().consumed_externally(
                nullifier,
                block_num,
                consumer_account,
            )?;
            input_note_update.inner_mut().set_consumed_tx_order(Some(0));
        }

        Ok(())
    }

    /// Builds a consumed input note record from a tracked output note and inserts it.
    ///
    /// Used when an output note is consumed externally and the client should also surface
    /// it as a consumed input — for example, when the same client tracks both the sender
    /// and the consumer of the note. No-op if the input is already tracked, the output is
    /// not tracked, or the output cannot be converted to a [`Note`].
    fn try_insert_consumed_input_from_output(
        &mut self,
        note_id: NoteId,
        consumer: AccountId,
        block_num: BlockNumber,
        consumed_tx_order: Option<u32>,
    ) -> Result<(), ClientError> {
        if self.input_notes.contains_key(&note_id) {
            return Ok(());
        }
        let Some(output_note) = self.output_notes.get(&note_id) else {
            return Ok(());
        };
        let Ok(note) = Note::try_from(output_note.inner().clone()) else {
            return Ok(());
        };

        let mut input_record = InputNoteRecord::from(note);
        let nullifier = input_record.nullifier();
        input_record.consumed_externally(nullifier, block_num, Some(consumer))?;
        input_record.set_consumed_tx_order(consumed_tx_order);
        self.insert_input_note(input_record, NoteUpdateType::Insert);
        Ok(())
    }

    /// If the note is tracked as an output note, transitions it to `Committed` with the
    /// given inclusion proof. No-op if the note is not tracked.
    fn try_commit_output_note(
        &mut self,
        note_id: NoteId,
        inclusion_proof: NoteInclusionProof,
    ) -> Result<(), ClientError> {
        if let Some(output_note) = self.get_output_note_by_id(note_id) {
            output_note.inclusion_proof_received(inclusion_proof)?;
        }
        Ok(())
    }

    /// Applies the necessary state transitions to the [`NoteUpdateTracker`] when a note is
    /// nullified in a block.
    ///
    /// For input note records two possible scenarios are considered:
    /// 1. The note was being processed by a local transaction that just got committed.
    /// 2. The note was consumed by a transaction not submitted by this client. This includes
    ///    consumption by untracked accounts as well as consumption by tracked accounts whose
    ///    transactions were submitted by other client instances. If a local transaction was
    ///    processing the note and it didn't get committed, the transaction should be discarded.
    ///
    /// If the note is tracked as an output but not as an input (e.g. the client tracks both the
    /// sender and the consumer), a new input record is created from the output details so the
    /// consumption surfaces through `InputNoteReader`.
    pub(crate) fn apply_note_consumption<'a>(
        &mut self,
        consumption: &NoteConsumption,
        mut committed_transactions: impl Iterator<Item = &'a TransactionRecord>,
    ) -> Result<(), ClientError> {
        let nullifier = consumption.nullifier;
        let block_num = consumption.block_num;
        let external_consumer = consumption.external_consumer;
        let order = self.get_nullifier_order(nullifier);
        let input_present = self.input_notes_by_nullifier.contains_key(&nullifier);

        if let Some(input_note_update) = self.get_input_note_update_by_nullifier(nullifier) {
            if let Some(consumer_transaction) = committed_transactions
                .find(|t| input_note_update.inner().consumer_transaction_id() == Some(&t.id))
            {
                // The note was being processed by a local transaction that just got committed
                if let TransactionStatus::Committed { block_number, .. } =
                    consumer_transaction.status
                {
                    input_note_update
                        .inner_mut()
                        .transaction_committed(consumer_transaction.id, block_number)?;
                }
            } else {
                // The note was consumed by a transaction not submitted by this client.
                // If the consuming account is tracked, external_consumer will be Some.
                input_note_update.inner_mut().consumed_externally(
                    nullifier,
                    block_num,
                    external_consumer,
                )?;
            }
            input_note_update.inner_mut().set_consumed_tx_order(order);
        }

        if let Some(output_note_record) = self.get_output_note_by_nullifier(nullifier) {
            output_note_record.nullifier_received(nullifier, block_num)?;
        }

        if !input_present
            && let Some(consumer) = external_consumer
            && let Some(note_id) = self.output_notes_by_nullifier.get(&nullifier).copied()
        {
            self.try_insert_consumed_input_from_output(note_id, consumer, block_num, order)?;
        }

        Ok(())
    }

    // PRIVATE HELPERS
    // --------------------------------------------------------------------------------------------

    /// Returns the position of the given nullifier in the consuming transaction order, or `None`
    /// if it is not present.
    fn get_nullifier_order(&self, nullifier: Nullifier) -> Option<u32> {
        self.nullifier_order.get(&nullifier).copied()
    }

    /// Returns a mutable reference to the input note record with the provided ID if it exists.
    fn get_input_note_by_id(&mut self, note_id: NoteId) -> Option<&mut InputNoteRecord> {
        self.input_notes.get_mut(&note_id).map(InputNoteUpdate::inner_mut)
    }

    /// Returns a mutable reference to the output note record with the provided ID if it exists.
    fn get_output_note_by_id(&mut self, note_id: NoteId) -> Option<&mut OutputNoteRecord> {
        self.output_notes.get_mut(&note_id).map(OutputNoteUpdate::inner_mut)
    }

    /// Returns a mutable reference to the input note update with the provided nullifier if it
    /// exists.
    fn get_input_note_update_by_nullifier(
        &mut self,
        nullifier: Nullifier,
    ) -> Option<&mut InputNoteUpdate> {
        let note_id = self.input_notes_by_nullifier.get(&nullifier).copied()?;
        self.input_notes.get_mut(&note_id)
    }

    /// Returns a mutable reference to the output note record with the provided nullifier if it
    /// exists.
    fn get_output_note_by_nullifier(
        &mut self,
        nullifier: Nullifier,
    ) -> Option<&mut OutputNoteRecord> {
        let note_id = self.output_notes_by_nullifier.get(&nullifier).copied()?;
        self.output_notes.get_mut(&note_id).map(OutputNoteUpdate::inner_mut)
    }

    /// Insert an input note update
    fn insert_input_note(&mut self, note: InputNoteRecord, update_type: NoteUpdateType) {
        let note_id = note.id();
        let nullifier = note.nullifier();
        self.input_notes_by_nullifier.insert(nullifier, note_id);
        let update = match update_type {
            NoteUpdateType::None => InputNoteUpdate::new_none(note),
            NoteUpdateType::Insert => InputNoteUpdate::new_insert(note),
            NoteUpdateType::Update => InputNoteUpdate::new_update(note),
        };
        self.input_notes.insert(note_id, update);
    }

    /// Insert an output note update
    fn insert_output_note(&mut self, note: OutputNoteRecord, update_type: NoteUpdateType) {
        let note_id = note.id();
        if let Some(nullifier) = note.nullifier() {
            self.output_notes_by_nullifier.insert(nullifier, note_id);
        }
        let update = match update_type {
            NoteUpdateType::None => OutputNoteUpdate::new_none(note),
            NoteUpdateType::Insert => OutputNoteUpdate::new_insert(note),
            NoteUpdateType::Update => OutputNoteUpdate::new_update(note),
        };
        self.output_notes.insert(note_id, update);
    }
}

// SERIALIZATION
// ================================================================================================

impl Serializable for NoteUpdateType {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_u8(*self as u8);
    }
}

impl Deserializable for NoteUpdateType {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        NoteUpdateType::try_from(source.read_u8()?).map_err(|val| {
            DeserializationError::InvalidValue(format!("invalid note update type: {val}"))
        })
    }
}

impl Serializable for InputNoteUpdate {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.note.write_into(target);
        self.update_type.write_into(target);
    }
}

impl Deserializable for InputNoteUpdate {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let note = InputNoteRecord::read_from(source)?;
        let update_type = NoteUpdateType::read_from(source)?;
        Ok(Self { note, update_type })
    }
}

impl Serializable for OutputNoteUpdate {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.note.write_into(target);
        self.update_type.write_into(target);
    }
}

impl Deserializable for OutputNoteUpdate {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let note = OutputNoteRecord::read_from(source)?;
        let update_type = NoteUpdateType::read_from(source)?;
        Ok(Self { note, update_type })
    }
}

impl Serializable for NoteUpdateTracker {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        // `input_notes_by_nullifier` and `output_notes_by_nullifier` are lookup indices that can
        // be reconstructed from `input_notes` and `output_notes`, so they are not serialized.
        self.input_notes.write_into(target);
        self.output_notes.write_into(target);
        self.nullifier_order.write_into(target);
    }
}

impl Deserializable for NoteUpdateTracker {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let input_notes = BTreeMap::<NoteId, InputNoteUpdate>::read_from(source)?;
        let output_notes = BTreeMap::<NoteId, OutputNoteUpdate>::read_from(source)?;
        let nullifier_order = BTreeMap::<Nullifier, u32>::read_from(source)?;

        let input_notes_by_nullifier = input_notes
            .iter()
            .map(|(note_id, update)| (update.inner().nullifier(), *note_id))
            .collect();
        let output_notes_by_nullifier = output_notes
            .iter()
            .filter_map(|(note_id, update)| {
                update.inner().nullifier().map(|nullifier| (nullifier, *note_id))
            })
            .collect();

        Ok(Self {
            input_notes,
            output_notes,
            input_notes_by_nullifier,
            output_notes_by_nullifier,
            nullifier_order,
        })
    }
}
