//! Provides note importing methods.
//!
//! This module allows users to import notes into the client's store.
//! Depending on the variant of [`NoteFile`] provided, the client will either fetch note details
//! from the network or create a new note record from supplied data. If a note already exists in
//! the store, it is updated with the new information. Additionally, the appropriate note tag
//! is tracked based on the imported note's metadata.
//!
//! For more specific information on how the process is performed, refer to the docs for
//! [`Client::import_note()`].
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::ToString;
use alloc::vec::Vec;

use miden_objects::block::BlockNumber;
use miden_objects::note::{
    Note,
    NoteDetails,
    NoteFile,
    NoteId,
    NoteInclusionProof,
    NoteMetadata,
    NoteTag,
};
use miden_tx::auth::TransactionAuthenticator;

use crate::rpc::RpcError;
use crate::rpc::domain::note::FetchedNote;
use crate::store::input_note_states::ExpectedNoteState;
use crate::store::{InputNoteRecord, InputNoteState};
use crate::sync::NoteTagRecord;
use crate::{Client, ClientError};

/// Note importing methods.
impl<AUTH> Client<AUTH>
where
    AUTH: TransactionAuthenticator + Sync + 'static,
{
    // INPUT NOTE CREATION
    // --------------------------------------------------------------------------------------------

    /// Imports a batch of new input notes into the client's store. The information stored depends
    /// on the type of note files provided. If the notes existed previously, it will be updated
    /// with the new information. The tags specified by the `NoteFile`s will start being
    /// tracked.
    ///
    /// - If the note files are [`NoteFile::NoteId`], the notes are fetched from the node and stored
    ///   in the client's store. If the note is private or doesn't exist, an error is returned.
    /// - If the note files are [`NoteFile::NoteDetails`], new notes are created with the provided
    ///   details and tags.
    /// - If the note files are [`NoteFile::NoteWithProof`], the notes are stored with the provided
    ///   inclusion proof and metadata. The block header data is only fetched from the node if the
    ///   note is committed in the past relative to the client.
    ///
    /// # Errors
    ///
    /// - If an attempt is made to overwrite a note that is currently processing.
    /// - If the client has reached the note tags limit
    ///   ([`NOTE_TAG_LIMIT`](crate::rpc::NOTE_TAG_LIMIT)).
    pub async fn import_notes(
        &mut self,
        note_files: &[NoteFile],
    ) -> Result<Vec<NoteId>, ClientError> {
        let mut ids = vec![];

        let mut requests_by_id = BTreeMap::new();
        let mut requests_by_details = vec![];
        let mut requests_by_proof = vec![];
        for note_file in note_files {
            let id = match &note_file {
                NoteFile::NoteId(id) => *id,
                NoteFile::NoteDetails { details, .. } => details.id(),
                NoteFile::NoteWithProof(note, _) => note.id(),
            };
            ids.push(id);

            let previous_note = self.get_input_note(id).await?;

            // If the note is already in the store and is in the state processing we return an
            // error.
            if let Some(true) = previous_note.as_ref().map(InputNoteRecord::is_processing) {
                return Err(ClientError::NoteImportError(format!(
                    "Can't overwrite note with id {id} as it's currently being processed",
                )));
            }

            match note_file.clone() {
                NoteFile::NoteId(id) => {
                    requests_by_id.insert(id, previous_note);
                },
                NoteFile::NoteDetails { details, after_block_num, tag } => {
                    requests_by_details.push((previous_note, details, after_block_num, tag));
                },
                NoteFile::NoteWithProof(note, inclusion_proof) => {
                    requests_by_proof.push((previous_note, note, inclusion_proof));
                },
            }
        }

        let mut imported_notes = vec![];
        if !requests_by_id.is_empty() {
            let notes_by_id = self.import_note_records_by_id(requests_by_id).await?;
            imported_notes.extend(notes_by_id.values().cloned());
        }

        if !requests_by_details.is_empty() {
            let notes_by_details = self.import_note_records_by_details(requests_by_details).await?;
            imported_notes.extend(notes_by_details);
        }

        if !requests_by_proof.is_empty() {
            let notes_by_proof = self.import_note_records_by_proof(requests_by_proof).await?;
            imported_notes.extend(notes_by_proof);
        }

        for note in imported_notes.into_iter().flatten() {
            if let InputNoteState::Expected(ExpectedNoteState { tag: Some(tag), .. }) = note.state()
            {
                self.insert_note_tag(NoteTagRecord::with_note_source(*tag, note.id())).await?;
            }
            self.store.upsert_input_notes(&[note]).await?;
        }

        Ok(ids)
    }

    // HELPERS
    // ================================================================================================

    /// Builds a note record map from the note IDs. If a note with the same ID was already stored it
    /// is passed via `previous_note` so it can be updated. The note information is fetched from
    /// the node and stored in the client's store.
    ///
    /// # Errors:
    /// - If a note doesn't exist on the node.
    /// - If a note exists but is private.
    async fn import_note_records_by_id(
        &self,
        notes: BTreeMap<NoteId, Option<InputNoteRecord>>,
    ) -> Result<BTreeMap<NoteId, Option<InputNoteRecord>>, ClientError> {
        let note_ids = notes.keys().copied().collect::<Vec<_>>();

        let fetched_notes =
            self.rpc_api.get_notes_by_id(&note_ids).await.map_err(|err| match err {
                RpcError::NoteNotFound(note_id) => ClientError::NoteNotFoundOnChain(note_id),
                err => ClientError::RpcError(err),
            })?;

        if fetched_notes.is_empty() {
            return Err(ClientError::NoteNotFoundOnChain(
                *note_ids.first().expect("at least a single note should be present"),
            ));
        }

        let mut note_records = BTreeMap::new();
        for fetched_note in fetched_notes {
            let note_id = fetched_note.id();
            let inclusion_proof = fetched_note.inclusion_proof().clone();

            let previous_note =
                notes.get(&note_id).cloned().expect("note id should be present in map");
            if let Some(mut previous_note) = previous_note {
                if previous_note
                    .inclusion_proof_received(inclusion_proof, *fetched_note.metadata())?
                {
                    self.store.remove_note_tag((&previous_note).try_into()?).await?;

                    note_records.insert(note_id, Some(previous_note));
                } else {
                    note_records.insert(note_id, None);
                }
            } else {
                let fetched_note = match fetched_note {
                    FetchedNote::Public(note, _) => note,
                    FetchedNote::Private(..) => {
                        return Err(ClientError::NoteImportError(
                            "Incomplete imported note is private".to_string(),
                        ));
                    },
                };

                let requested_note = (previous_note, fetched_note, inclusion_proof);
                // TODO: batch requested notes?
                let note_record = self
                    .import_note_records_by_proof(vec![requested_note])
                    .await?
                    .first()
                    .ok_or(ClientError::NoteImportError(
                        "Node should have retrieved requested proof or specify missing".to_string(),
                    ))?
                    .clone();
                note_records.insert(note_id, note_record);
            }
        }
        Ok(note_records)
    }

    /// Builds a note record list from notes and inclusion proofs. If a note with the same ID was
    /// already stored it is passed via `previous_note` so it can be updated. The note's
    /// nullifier is used to determine if the note has been consumed in the node and gives it
    /// the correct state.
    ///
    /// If the note isn't consumed and it was committed in the past relative to the client, then
    /// the MMR for the relevant block is fetched from the node and stored.
    pub(crate) async fn import_note_records_by_proof(
        &self,
        requested_notes: Vec<(Option<InputNoteRecord>, Note, NoteInclusionProof)>,
    ) -> Result<Vec<Option<InputNoteRecord>>, ClientError> {
        // TODO: iterating twice over requested notes
        let mut note_records = vec![];

        let mut nullifier_requests = vec![];
        let mut lowest_block_height: BlockNumber = u32::MAX.into();
        for (previous_note, note, inclusion_proof) in &requested_notes {
            if let Some(previous_note) = previous_note {
                nullifier_requests.push(previous_note.nullifier());
                if inclusion_proof.location().block_num() < lowest_block_height {
                    lowest_block_height = inclusion_proof.location().block_num();
                }
            } else {
                nullifier_requests.push(note.nullifier());
                if inclusion_proof.location().block_num() < lowest_block_height {
                    lowest_block_height = inclusion_proof.location().block_num();
                }
            }
        }

        let nullifier_commit_heights = self
            .rpc_api
            .get_nullifiers_commit_height(&nullifier_requests, lowest_block_height)
            .await?;

        for (previous_note, note, inclusion_proof) in requested_notes {
            let metadata = *note.metadata();
            let mut note_record = previous_note.unwrap_or(InputNoteRecord::new(
                note.into(),
                self.store.get_current_timestamp(),
                ExpectedNoteState {
                    metadata: Some(metadata),
                    after_block_num: inclusion_proof.location().block_num(),
                    tag: Some(metadata.tag()),
                }
                .into(),
            ));

            if let Some(Some(block_height)) = nullifier_commit_heights.get(&note_record.nullifier())
            {
                if note_record.consumed_externally(note_record.nullifier(), *block_height)? {
                    note_records.push(Some(note_record));
                }

                note_records.push(None);
            } else {
                let block_height = inclusion_proof.location().block_num();
                let current_block_num = self.get_sync_height().await?;

                let mut note_changed =
                    note_record.inclusion_proof_received(inclusion_proof, metadata)?;

                if block_height <= current_block_num {
                    // If the note is committed in the past we need to manually fetch the block
                    // header and MMR proof to verify the inclusion proof.
                    let mut current_partial_mmr = self.store.get_current_partial_mmr().await?;

                    let block_header = self
                        .get_and_store_authenticated_block(block_height, &mut current_partial_mmr)
                        .await?;

                    note_changed |= note_record.block_header_received(&block_header)?;
                } else {
                    // If the note is in the future we import it as unverified. We add the note tag
                    // so that the note is verified naturally in the next sync.
                    self.insert_note_tag(NoteTagRecord::with_note_source(
                        metadata.tag(),
                        note_record.id(),
                    ))
                    .await?;
                }

                if note_changed {
                    note_records.push(Some(note_record));
                } else {
                    note_records.push(None);
                }
            }
        }

        Ok(note_records)
    }

    /// Builds a note record list from note details. If a note with the same ID was already stored
    /// it is passed via `previous_note` so it can be updated.
    async fn import_note_records_by_details(
        &mut self,
        requested_notes: Vec<(Option<InputNoteRecord>, NoteDetails, BlockNumber, Option<NoteTag>)>,
    ) -> Result<Vec<Option<InputNoteRecord>>, ClientError> {
        // TODO: refactor to only call from a single block?
        let mut block_number_tag_requests: BTreeMap<&BlockNumber, Vec<(NoteId, &NoteTag)>> =
            BTreeMap::new();
        for (_, details, after_block_num, tag) in &requested_notes {
            if let Some(tag) = tag {
                block_number_tag_requests
                    .entry(after_block_num)
                    .or_default()
                    .push((details.id(), tag));
            }
        }

        let mut committed_notes_data: BTreeMap<NoteId, Option<(NoteMetadata, NoteInclusionProof)>> =
            BTreeMap::new();
        for (block_number, note_requests) in block_number_tag_requests {
            let committed_notes_data_in_block =
                self.check_expected_notes(*block_number, note_requests).await?;
            for (note_id, metadata_and_proof) in committed_notes_data_in_block {
                committed_notes_data.insert(note_id, metadata_and_proof);
            }
        }

        let mut note_records = vec![];
        for (previous_note, details, after_block_num, tag) in requested_notes {
            let mut note_record = previous_note.unwrap_or({
                InputNoteRecord::new(
                    details,
                    self.store.get_current_timestamp(),
                    ExpectedNoteState { metadata: None, after_block_num, tag }.into(),
                )
            });

            match committed_notes_data.remove(&note_record.id()) {
                Some(Some((metadata, inclusion_proof))) => {
                    let mut current_partial_mmr = self.store.get_current_partial_mmr().await?;
                    let block_header = self
                        .get_and_store_authenticated_block(
                            inclusion_proof.location().block_num(),
                            &mut current_partial_mmr,
                        )
                        .await?;

                    let note_changed =
                        note_record.inclusion_proof_received(inclusion_proof, metadata)?;

                    if note_record.block_header_received(&block_header)? | note_changed {
                        self.store
                            .remove_note_tag(NoteTagRecord::with_note_source(
                                metadata.tag(),
                                note_record.id(),
                            ))
                            .await?;

                        note_records.push(Some(note_record));
                    } else {
                        note_records.push(None);
                    }
                },
                _ => {
                    note_records.push(Some(note_record));
                },
            }
        }

        Ok(note_records)
    }

    /// Checks if notes with their given `note_tag` and ID are present in the chain between the
    /// `request_block_num` and the current block. If found it returns their metadata and inclusion
    /// proof.
    async fn check_expected_notes(
        &mut self,
        mut request_block_num: BlockNumber,
        // Expected notes with their tags
        expected_notes: Vec<(NoteId, &NoteTag)>,
    ) -> Result<BTreeMap<NoteId, Option<(NoteMetadata, NoteInclusionProof)>>, ClientError> {
        let tracked_tags: BTreeSet<NoteTag> = expected_notes.iter().map(|(_, tag)| **tag).collect();
        let mut retrieved_proofs = BTreeMap::new();
        let current_block_num = self.get_sync_height().await?;
        loop {
            if request_block_num > current_block_num {
                break;
            }

            let sync_notes =
                self.rpc_api.sync_notes(request_block_num, None, &tracked_tags).await?;

            for sync_note in sync_notes.notes {
                if !expected_notes.iter().any(|(id, _)| id == sync_note.note_id()) {
                    continue;
                }

                // This means that a note with the same id was found.
                // Therefore, we should mark the note as committed.
                let note_block_num = sync_notes.block_header.block_num();

                if note_block_num > current_block_num {
                    break;
                }

                let note_inclusion_proof = NoteInclusionProof::new(
                    note_block_num,
                    sync_note.note_index(),
                    sync_note.inclusion_path().clone(),
                )?;

                retrieved_proofs.insert(
                    *sync_note.note_id(),
                    Some((sync_note.metadata(), note_inclusion_proof)),
                );
            }

            // We might have reached the chain tip without having found some notes, bail if so
            if sync_notes.block_header.block_num() == sync_notes.chain_tip {
                break;
            }

            // This means that a note with the same id was not found.
            // Therefore, we should request again for sync_notes with the same note_tag
            // and with the block_num of the last block header
            // (sync_notes.block_header.unwrap()).
            request_block_num = sync_notes.block_header.block_num();
        }
        Ok(retrieved_proofs)
    }
}
