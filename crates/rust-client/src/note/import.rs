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
use alloc::collections::BTreeMap;
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

    /// Imports a new input note into the client's store. The information stored depends on the
    /// type of note file provided. If the note existed previously, it will be updated with the
    /// new information. The tag specified by the `NoteFile` will start being tracked.
    ///
    /// - If the note file is a [`NoteFile::NoteId`], the note is fetched from the node and stored
    ///   in the client's store. If the note is private or doesn't exist, an error is returned.
    /// - If the note file is a [`NoteFile::NoteDetails`], a new note is created with the provided
    ///   details and tag.
    /// - If the note file is a [`NoteFile::NoteWithProof`], the note is stored with the provided
    ///   inclusion proof and metadata. The block header data is only fetched from the node if the
    ///   note is committed in the past relative to the client.
    ///
    /// # Errors
    ///
    /// - If an attempt is made to overwrite a note that is currently processing.
    /// - If the client has reached the note tags limit
    ///   ([`NOTE_TAG_LIMIT`](crate::rpc::NOTE_TAG_LIMIT)).
    pub async fn import_note(&mut self, note_file: NoteFile) -> Result<NoteId, ClientError> {
        let id = match &note_file {
            NoteFile::NoteId(id) => *id,
            NoteFile::NoteDetails { details, .. } => details.id(),
            NoteFile::NoteWithProof(note, _) => note.id(),
        };

        let previous_note = self.get_input_note(id).await?;

        // If the note is already in the store and is in the state processing we return an error.
        if let Some(true) = previous_note.as_ref().map(InputNoteRecord::is_processing) {
            return Err(ClientError::NoteImportError(format!(
                "Can't overwrite note with id {id} as it's currently being processed",
            )));
        }

        let note = match note_file {
            NoteFile::NoteId(id) => self.import_note_record_by_id(previous_note, id).await?,
            NoteFile::NoteDetails { details, after_block_num, tag } => {
                self.import_note_record_by_details(previous_note, details, after_block_num, tag)
                    .await?
            },
            NoteFile::NoteWithProof(note, inclusion_proof) => {
                self.import_note_record_by_proof(previous_note, note, inclusion_proof).await?
            },
        };

        if let Some(note) = note {
            if let InputNoteState::Expected(ExpectedNoteState { tag: Some(tag), .. }) = note.state()
            {
                self.insert_note_tag(NoteTagRecord::with_note_source(*tag, note.id())).await?;
            }
            self.store.upsert_input_notes(&[note]).await?;
        }

        Ok(id)
    }

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
        note_files: Vec<NoteFile>,
    ) -> Result<Vec<NoteId>, ClientError> {
        let ids = vec![];

        let mut requests_by_id = BTreeMap::new();
        let mut requests_by_details = vec![];
        let mut requests_by_proof = vec![];
        for note_file in note_files {
            let id = match &note_file {
                NoteFile::NoteId(id) => *id,
                NoteFile::NoteDetails { details, .. } => details.id(),
                NoteFile::NoteWithProof(note, _) => note.id(),
            };

            let previous_note = self.get_input_note(id).await?;

            // If the note is already in the store and is in the state processing we return an
            // error.
            if let Some(true) = previous_note.as_ref().map(InputNoteRecord::is_processing) {
                return Err(ClientError::NoteImportError(format!(
                    "Can't overwrite note with id {id} as it's currently being processed",
                )));
            }

            match note_file {
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

        let notes_by_id = self.import_note_records_by_id(requests_by_id).await?;
        for (note_id, note) in notes_by_id {
            if let Some(note) = note {
                if let InputNoteState::Expected(ExpectedNoteState { tag: Some(tag), .. }) =
                    note.state()
                {
                    self.insert_note_tag(NoteTagRecord::with_note_source(*tag, note_id)).await?;
                }
                self.store.upsert_input_notes(&[note]).await?;
            }
        }

        let notes_by_details = self.import_note_records_by_details(requests_by_details).await?;
        for note in notes_by_details.into_iter().flatten() {
            if let InputNoteState::Expected(ExpectedNoteState { tag: Some(tag), .. }) = note.state()
            {
                self.insert_note_tag(NoteTagRecord::with_note_source(*tag, note.id())).await?;
            }
            self.store.upsert_input_notes(&[note]).await?;
        }

        let notes_by_proof = self.import_note_records_by_proof(requests_by_proof).await?;
        for note in notes_by_proof.into_iter().flatten() {
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

    /// Builds a note record from the note ID. If a note with the same ID was already stored it is
    /// passed via `previous_note` so it can be updated. The note information is fetched from the
    /// node and stored in the client's store.
    ///
    /// # Errors:
    /// - If the note doesn't exist on the node.
    /// - If the note exists but is private.
    async fn import_note_record_by_id(
        &self,
        previous_note: Option<InputNoteRecord>,
        id: NoteId,
    ) -> Result<Option<InputNoteRecord>, ClientError> {
        let fetched_note = self.rpc_api.get_note_by_id(id).await.map_err(|err| match err {
            RpcError::NoteNotFound(note_id) => ClientError::NoteNotFoundOnChain(note_id),
            err => ClientError::RpcError(err),
        })?;

        let inclusion_proof = fetched_note.inclusion_proof().clone();

        if let Some(mut previous_note) = previous_note {
            if previous_note.inclusion_proof_received(inclusion_proof, *fetched_note.metadata())? {
                self.store.remove_note_tag((&previous_note).try_into()?).await?;

                Ok(Some(previous_note))
            } else {
                Ok(None)
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

            self.import_note_record_by_proof(previous_note, fetched_note, inclusion_proof)
                .await
        }
    }

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

                let note_record = self
                    .import_note_record_by_proof(previous_note, fetched_note, inclusion_proof)
                    .await?;
                note_records.insert(note_id, note_record);
            }
        }
        Ok(note_records)
    }

    /// Builds a note record from the note and inclusion proof. If a note with the same ID was
    /// already stored it is passed via `previous_note` so it can be updated. The note's
    /// nullifier is used to determine if the note has been consumed in the node and gives it
    /// the correct state.
    ///
    /// If the note isn't consumed and it was committed in the past relative to the client, then
    /// the MMR for the relevant block is fetched from the node and stored.
    pub(crate) async fn import_note_record_by_proof(
        &self,
        previous_note: Option<InputNoteRecord>,
        note: Note,
        inclusion_proof: NoteInclusionProof,
    ) -> Result<Option<InputNoteRecord>, ClientError> {
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

        if let Some(block_height) = self
            .rpc_api
            .get_nullifier_commit_height(
                &note_record.nullifier(),
                inclusion_proof.location().block_num(),
            )
            .await?
        {
            if note_record.consumed_externally(note_record.nullifier(), block_height)? {
                return Ok(Some(note_record));
            }

            Ok(None)
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
                // If the note is in the future we import it as unverified. We add the note tag so
                // that the note is verified naturally in the next sync.
                self.insert_note_tag(NoteTagRecord::with_note_source(
                    metadata.tag(),
                    note_record.id(),
                ))
                .await?;
            }

            if note_changed { Ok(Some(note_record)) } else { Ok(None) }
        }
    }

    pub(crate) async fn import_note_records_by_proof(
        &self,
        requested_notes: Vec<(Option<InputNoteRecord>, Note, NoteInclusionProof)>,
    ) -> Result<Vec<Option<InputNoteRecord>>, ClientError> {
        // TODO: this is not batching any call to the rpc_api
        // how do we optimize this?
        let mut note_records = vec![];

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

            if let Some(block_height) = self
                .rpc_api
                .get_nullifier_commit_height(
                    &note_record.nullifier(),
                    inclusion_proof.location().block_num(),
                )
                .await?
            {
                if note_record.consumed_externally(note_record.nullifier(), block_height)? {
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

    /// Builds a note record from the note details. If a note with the same ID was already stored it
    /// is passed via `previous_note` so it can be updated.
    async fn import_note_record_by_details(
        &mut self,
        previous_note: Option<InputNoteRecord>,
        details: NoteDetails,
        after_block_num: BlockNumber,
        tag: Option<NoteTag>,
    ) -> Result<Option<InputNoteRecord>, ClientError> {
        let mut note_record = previous_note.unwrap_or({
            InputNoteRecord::new(
                details,
                self.store.get_current_timestamp(),
                ExpectedNoteState { metadata: None, after_block_num, tag }.into(),
            )
        });

        let committed_note_data = if let Some(tag) = tag {
            self.check_expected_note(after_block_num, tag, note_record.details()).await?
        } else {
            None
        };

        match committed_note_data {
            Some((metadata, inclusion_proof)) => {
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

                    Ok(Some(note_record))
                } else {
                    Ok(None)
                }
            },
            None => Ok(Some(note_record)),
        }
    }

    async fn import_note_records_by_details(
        &mut self,
        requested_notes: Vec<(Option<InputNoteRecord>, NoteDetails, BlockNumber, Option<NoteTag>)>,
    ) -> Result<Vec<Option<InputNoteRecord>>, ClientError> {
        // TODO: this is not batching any call to the rpc_api
        // how do we optimize this?
        let mut note_records = vec![];

        for (previous_note, details, after_block_num, tag) in requested_notes {
            let mut note_record = previous_note.unwrap_or({
                InputNoteRecord::new(
                    details,
                    self.store.get_current_timestamp(),
                    ExpectedNoteState { metadata: None, after_block_num, tag }.into(),
                )
            });

            let committed_note_data = if let Some(tag) = tag {
                self.check_expected_note(after_block_num, tag, note_record.details()).await?
            } else {
                None
            };

            match committed_note_data {
                Some((metadata, inclusion_proof)) => {
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
                None => {
                    note_records.push(Some(note_record));
                },
            }
        }

        Ok(note_records)
    }

    /// Checks if a note with the given `note_tag` and ID is present in the chain between the
    /// `request_block_num` and the current block. If found it returns its metadata and inclusion
    /// proof.
    async fn check_expected_note(
        &mut self,
        mut request_block_num: BlockNumber,
        tag: NoteTag,
        expected_note: &miden_objects::note::NoteDetails,
    ) -> Result<Option<(NoteMetadata, NoteInclusionProof)>, ClientError> {
        let current_block_num = self.get_sync_height().await?;
        loop {
            if request_block_num > current_block_num {
                return Ok(None);
            }

            let sync_notes = self
                .rpc_api
                .sync_notes(request_block_num, None, &[tag].into_iter().collect())
                .await?;

            // This means that notes with that note_tag were found.
            // Therefore, we should check if a note with the same id was found.
            let committed_note =
                sync_notes.notes.iter().find(|note| note.note_id() == &expected_note.id());

            if let Some(note) = committed_note {
                // This means that a note with the same id was found.
                // Therefore, we should mark the note as committed.
                let note_block_num = sync_notes.block_header.block_num();

                if note_block_num > current_block_num {
                    return Ok(None);
                }

                let note_inclusion_proof = NoteInclusionProof::new(
                    note_block_num,
                    note.note_index(),
                    note.inclusion_path().clone(),
                )?;

                return Ok(Some((note.metadata(), note_inclusion_proof)));
            }

            // We might have reached the chain tip without having found the note, bail if so
            if sync_notes.block_header.block_num() == sync_notes.chain_tip {
                return Ok(None);
            }

            // This means that a note with the same id was not found.
            // Therefore, we should request again for sync_notes with the same note_tag
            // and with the block_num of the last block header
            // (sync_notes.block_header.unwrap()).
            request_block_num = sync_notes.block_header.block_num();
        }
    }
}
