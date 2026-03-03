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

use miden_protocol::block::BlockNumber;
use miden_protocol::note::{
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
use crate::store::{InputNoteRecord, InputNoteState, NoteFilter};
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
    /// tracked. Returns the IDs of notes that were successfully imported or updated.
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
    /// - If the client has reached the note tags limit.
    ///
    /// Note: This operation is atomic. If any note file is invalid or any existing note is in the
    /// processing state, the entire operation fails and no notes are imported.
    pub async fn import_notes(
        &mut self,
        note_files: &[NoteFile],
    ) -> Result<Vec<NoteId>, ClientError> {
        let mut note_ids_map = BTreeMap::new();
        for note_file in note_files {
            let id = match &note_file {
                NoteFile::NoteId(id) => *id,
                NoteFile::NoteDetails { details, .. } => details.id(),
                NoteFile::NoteWithProof(note, _) => note.id(),
            };
            note_ids_map.insert(id, note_file);
        }

        let note_ids: Vec<NoteId> = note_ids_map.keys().copied().collect();
        let previous_notes: Vec<InputNoteRecord> =
            self.get_input_notes(NoteFilter::List(note_ids)).await?;
        let previous_notes_map: BTreeMap<NoteId, InputNoteRecord> =
            previous_notes.into_iter().map(|note| (note.id(), note)).collect();

        let mut requests_by_id = BTreeMap::new();
        let mut requests_by_details = vec![];
        let mut requests_by_proof = vec![];

        for (note_id, note_file) in note_ids_map {
            let previous_note = previous_notes_map.get(&note_id).cloned();

            // If the note is already in the store and is in the state processing we return an
            // error.
            if let Some(true) = previous_note.as_ref().map(InputNoteRecord::is_processing) {
                return Err(ClientError::NoteImportError(format!(
                    "Can't overwrite note with id {note_id} as it's currently being processed",
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

        let mut imported_note_ids = Vec::with_capacity(imported_notes.len());
        for note in imported_notes.into_iter().flatten() {
            imported_note_ids.push(note.id());
            if let InputNoteState::Expected(ExpectedNoteState { tag: Some(tag), .. }) = note.state()
            {
                self.insert_note_tag(NoteTagRecord::with_note_source(*tag, note.id())).await?;
            }
            self.store.upsert_input_notes(&[note]).await?;
        }

        Ok(imported_note_ids)
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
            return Err(ClientError::NoteImportError("No notes fetched from node".to_string()));
        }

        let mut note_records = BTreeMap::new();
        let mut notes_to_request = vec![];
        for fetched_note in fetched_notes {
            let note_id = fetched_note.id();
            let inclusion_proof = fetched_note.inclusion_proof().clone();

            let previous_note =
                notes.get(&note_id).cloned().ok_or(ClientError::NoteImportError(format!(
                    "Failed to retrieve note with id {note_id} from node"
                )))?;
            if let Some(mut previous_note) = previous_note {
                if previous_note
                    .inclusion_proof_received(inclusion_proof, fetched_note.metadata().clone())?
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

                let note_request = (previous_note, fetched_note, inclusion_proof);
                notes_to_request.push(note_request);
            }
        }

        if !notes_to_request.is_empty() {
            let note_records_by_proof = self.import_note_records_by_proof(notes_to_request).await?;
            for note_record in note_records_by_proof.iter().flatten().cloned() {
                note_records.insert(note_record.id(), Some(note_record));
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

        let mut nullifier_requests = BTreeSet::new();
        let mut lowest_block_height: BlockNumber = u32::MAX.into();
        for (previous_note, note, inclusion_proof) in &requested_notes {
            if let Some(previous_note) = previous_note {
                nullifier_requests.insert(previous_note.nullifier());
                if inclusion_proof.location().block_num() < lowest_block_height {
                    lowest_block_height = inclusion_proof.location().block_num();
                }
            } else {
                nullifier_requests.insert(note.nullifier());
                if inclusion_proof.location().block_num() < lowest_block_height {
                    lowest_block_height = inclusion_proof.location().block_num();
                }
            }
        }

        let nullifier_commit_heights = self
            .rpc_api
            .get_nullifier_commit_heights(nullifier_requests, lowest_block_height)
            .await?;

        for (previous_note, note, inclusion_proof) in requested_notes {
            let metadata = note.metadata().clone();
            let mut note_record = previous_note.unwrap_or(InputNoteRecord::new(
                note.into(),
                self.store.get_current_timestamp(),
                ExpectedNoteState {
                    metadata: Some(metadata.clone()),
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

                let tag = metadata.tag();
                let mut note_changed =
                    note_record.inclusion_proof_received(inclusion_proof, metadata)?;

                if block_height <= current_block_num {
                    // FIXME: We should be able to build the mmr only once (outside the for loop).
                    // For some reason this leads to error, probably related to:
                    // https://github.com/0xMiden/miden-client/issues/1205
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
                    self.insert_note_tag(NoteTagRecord::with_note_source(tag, note_record.id()))
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
        let mut lowest_request_block: BlockNumber = u32::MAX.into();
        let mut note_requests = vec![];
        for (_, details, after_block_num, tag) in &requested_notes {
            if let Some(tag) = tag {
                note_requests.push((details.id(), tag));
                if after_block_num < &lowest_request_block {
                    lowest_request_block = *after_block_num;
                }
            }
        }
        let mut committed_notes_data =
            self.check_expected_notes(lowest_request_block, note_requests).await?;

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
                    // FIXME: We should be able to build the mmr only once (outside the for loop).
                    // For some reason this leads to error, probably related to:
                    // https://github.com/0xMiden/miden-client/issues/1205
                    let mut current_partial_mmr = self.store.get_current_partial_mmr().await?;
                    let block_header = self
                        .get_and_store_authenticated_block(
                            inclusion_proof.location().block_num(),
                            &mut current_partial_mmr,
                        )
                        .await?;

                    let tag = metadata.tag();
                    let note_changed =
                        note_record.inclusion_proof_received(inclusion_proof, metadata)?;

                    if note_record.block_header_received(&block_header)? | note_changed {
                        self.store
                            .remove_note_tag(NoteTagRecord::with_note_source(tag, note_record.id()))
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
