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
use miden_tx::utils::{Deserializable, DeserializationError, SliceReader};

pub use self::errors::NoteTransportError;
use crate::store::{InputNoteRecord, Store};
use crate::{Client, ClientError};

pub const NOTE_TRANSPORT_DEFAULT_ENDPOINT: &str = "http://localhost:57292";
pub const NOTE_TRANSPORT_CURSOR_STORE_SETTING: &str = "note_transport_cursor";

/// Client note transport methods.
impl<AUTH> Client<AUTH> {
    /// Check if note transport connection is configured
    pub fn is_note_transport_enabled(&self) -> bool {
        self.note_transport_api.is_some()
    }

    /// Send a note through the note transport network.
    ///
    /// The note will be end-to-end encrypted (unimplemented, currently plaintext)
    /// using the provided recipient's `address` details.
    /// The recipient will be able to retrieve this note through the note's [`NoteTag`].
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

    /// Fetch notes for tracked note tags.
    ///
    /// The client will query the configured note transport node for all tracked note tags.
    /// To list tracked tags please use [`Client::get_note_tags`]. To add a new note tag please use
    /// [`Client::add_note_tag`].
    /// Only notes directed at your addresses will be stored and readable given the use of
    /// end-to-end encryption (unimplemented).
    /// Fetched notes will be stored into the client's store.
    ///
    /// An internal pagination mechanism is employed to reduce the number of downloaded notes.
    /// To fetch the full history of private notes for the tracked tags, use
    /// [`Client::fetch_all_private_notes`].
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

    /// Fetches all notes for tracked note tags.
    ///
    /// Similar to [`Client::fetch_private_notes`] however does not employ pagination,
    /// fetching all notes stored in the note transport network for the tracked tags.
    /// Please prefer using [`Client::fetch_private_notes`] to avoid downloading repeated notes.
    pub async fn fetch_all_private_notes(&mut self) -> Result<(), ClientError> {
        let api = self.get_note_transport_api()?;

        let note_tags = self.store.get_unique_note_tags().await?;

        let update = NoteTransport::new(api)
            .fetch_notes(NoteTransportCursor::init(), note_tags)
            .await?;

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

/// Note transport cursor
///
/// Pagination integer used to reduce the number of fetched notes from the note transport network,
/// avoiding duplicate downloads.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct NoteTransportCursor(u64);

/// Note Transport update
pub struct NoteTransportUpdate {
    /// Pagination cursor for next fetch
    pub cursor: NoteTransportCursor,
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
        cursor: NoteTransportCursor,
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

impl NoteTransportCursor {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn init() -> Self {
        Self::new(0)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl From<u64> for NoteTransportCursor {
    fn from(value: u64) -> Self {
        Self::new(value)
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
        cursor: NoteTransportCursor,
    ) -> Result<(Vec<NoteInfo>, NoteTransportCursor), NoteTransportError>;

    /// Stream notes for a given tag
    async fn stream_notes(
        &self,
        tag: NoteTag,
        cursor: NoteTransportCursor,
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

fn rejoin_note(header: &NoteHeader, details_bytes: &[u8]) -> Result<Note, DeserializationError> {
    let mut reader = SliceReader::new(details_bytes);
    let details = NoteDetails::read_from(&mut reader)?;
    let metadata = *header.metadata();
    Ok(Note::new(details.assets().clone(), metadata, details.recipient().clone()))
}
