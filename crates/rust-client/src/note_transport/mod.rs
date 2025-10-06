pub mod errors;
#[cfg(feature = "tonic")]
pub mod grpc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use futures::Stream;
use miden_lib::utils::Serializable;
use miden_objects::address::Address;
use miden_objects::note::{Note, NoteDetails, NoteHeader, NoteTag};
use miden_tx::utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, SliceReader};

pub use self::errors::NoteTransportError;
use crate::store::{InputNoteRecord, Store};
use crate::{Client, ClientError};

pub const NOTE_TRANSPORT_DEFAULT_ENDPOINT: &str = "http://localhost:57292";
pub const NOTE_TRANSPORT_CURSOR_STORE_SETTING: &str = "note_transport_cursor";

/// Client note transport methods.
impl<AUTH> Client<AUTH> {
    /// Check if note transport is configured
    pub fn is_note_transport_enabled(&self) -> bool {
        self.note_transport_api.is_some()
    }

    /// Send a note
    pub async fn send_private_note(
        &mut self,
        note: Note,
        _address: &Address,
    ) -> Result<(), ClientError> {
        let api = self.get_note_transport_api()?;

        let header = *note.header();
        let details = NoteDetails::from(note);
        let details_bytes = details.to_bytes();
        // e2ee impl hint:
        // address.key().encrypt(details_bytes)
        api.send_note(header, details_bytes).await?;

        Ok(())
    }

    /// Fetch notes for tracked note tags
    ///
    /// An internal pagination mechanism is employed to reduce the number of downloaded notes.
    pub async fn fetch_private_notes(&mut self) -> Result<(), ClientError> {
        let api = self.get_note_transport_api()?;

        // Unique tags
        let note_tags = self.store.get_unique_note_tags().await?;
        // Get global cursor
        let cursor = self.store.get_note_transport_cursor().await?;

        let update = NoteTransport::new(api).fetch_notes(cursor, note_tags).await?;

        self.store.apply_note_transport_update(update).await?;

        Ok(())
    }

    /// Fetches all notes for tracked note tags
    ///
    /// All notes stored in the note transport network will be fetched.
    pub async fn fetch_all_private_notes(&mut self) -> Result<(), ClientError> {
        let api = self.get_note_transport_api()?;

        let note_tags = self.store.get_unique_note_tags().await?;

        let update = NoteTransport::new(api.clone()).fetch_notes(0, note_tags).await?;

        self.store.apply_note_transport_update(update).await?;

        Ok(())
    }

    /// Returns the Note Transport client, if configured
    pub(crate) fn get_note_transport_api(
        &self,
    ) -> Result<Arc<dyn NoteTransportClient>, NoteTransportError> {
        self.note_transport_api.clone().ok_or(NoteTransportError::Disabled)
    }
}

/// Populates the note transport cursor setting with 0, if it is not setup
pub(crate) async fn init_note_transport_cursor(store: Arc<dyn Store>) -> Result<(), ClientError> {
    let setting = NOTE_TRANSPORT_CURSOR_STORE_SETTING;
    if store.get_setting(setting.into()).await?.is_none() {
        let initial_cursor = 0u64.to_be_bytes().to_vec();
        store.set_setting(setting.into(), initial_cursor).await?;
    }
    Ok(())
}

/// Note Transport methods
pub struct NoteTransport {
    api: Arc<dyn NoteTransportClient>,
}

/// Note Transport update
pub struct NoteTransportUpdate {
    /// Pagination cursor for next fetch
    pub cursor: u64,
    /// Fetched notes
    pub note_updates: Vec<InputNoteRecord>,
}

impl NoteTransport {
    pub fn new(api: Arc<dyn NoteTransportClient>) -> Self {
        Self { api }
    }

    /// Fetch notes for provided note tags with pagination
    ///
    /// Only notes after the provided cursor are requested.
    pub(crate) async fn fetch_notes<I>(
        &mut self,
        cursor: u64,
        tags: I,
    ) -> Result<NoteTransportUpdate, ClientError>
    where
        I: IntoIterator<Item = NoteTag>,
    {
        let mut note_updates = vec![];
        // Fetch notes
        let (note_infos, rcursor) =
            self.api.fetch_notes(&tags.into_iter().collect::<Vec<_>>(), cursor).await?;
        for note_info in &note_infos {
            // e2ee impl hint:
            // for key in self.store.decryption_keys() try
            // key.decrypt(details_bytes_encrypted)
            let note = rejoin_note(&note_info.header, &note_info.details_bytes)?;
            let input_note = InputNoteRecord::from(note);
            note_updates.push(input_note);
        }

        let update = NoteTransportUpdate { note_updates, cursor: rcursor };

        Ok(update)
    }
}

/// The main transport client trait for sending and receiving encrypted notes
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait NoteTransportClient: Send + Sync {
    /// Send a note with optionally encrypted details
    async fn send_note(
        &self,
        header: NoteHeader,
        details: Vec<u8>,
    ) -> Result<(), NoteTransportError>;

    /// Fetch notes for given tags
    ///
    /// Downloads notes for given tags.
    /// Returns notes labelled after the provided cursor (pagination), and an updated cursor.
    async fn fetch_notes(
        &self,
        tag: &[NoteTag],
        cursor: u64,
    ) -> Result<(Vec<NoteInfo>, u64), NoteTransportError>;

    /// Stream notes for a given tag
    async fn stream_notes(
        &self,
        tag: NoteTag,
        cursor: u64,
    ) -> Result<Box<dyn NoteStream>, NoteTransportError>;
}

/// Stream trait for note streaming
pub trait NoteStream:
    Stream<Item = Result<Vec<NoteInfo>, NoteTransportError>> + Send + Unpin
{
}

/// Information about a note in API responses
#[derive(Debug, Clone)]
pub struct NoteInfo {
    /// Note header
    pub header: NoteHeader,
    /// Note details, can be encrypted
    pub details_bytes: Vec<u8>,
}

// SERIALIZATION
// ================================================================================================

impl Serializable for NoteInfo {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.header.write_into(target);
        self.details_bytes.write_into(target);
    }
}

impl Deserializable for NoteInfo {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let header = NoteHeader::read_from(source)?;
        let details_bytes = Vec::<u8>::read_from(source)?;
        Ok(NoteInfo { header, details_bytes })
    }
}

fn rejoin_note(header: &NoteHeader, details_bytes: &[u8]) -> Result<Note, DeserializationError> {
    let mut reader = SliceReader::new(details_bytes);
    let details = NoteDetails::read_from(&mut reader)?;
    let metadata = *header.metadata();
    Ok(Note::new(details.assets().clone(), metadata, details.recipient().clone()))
}
