pub mod errors;
pub mod generated;
#[cfg(feature = "tonic")]
pub mod grpc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use futures::Stream;
use miden_protocol::address::Address;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::{Note, NoteDetails, NoteFile, NoteHeader, NoteTag};
use miden_protocol::utils::Serializable;
use miden_tx::auth::TransactionAuthenticator;
use miden_tx::utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, SliceReader};

pub use self::errors::NoteTransportError;
use crate::store::Store;
use crate::{Client, ClientError};

pub const NOTE_TRANSPORT_DEFAULT_ENDPOINT: &str = "https://transport.miden.io";
pub const NOTE_TRANSPORT_CURSOR_STORE_SETTING: &str = "note_transport_cursor";

/// Client note transport methods.
impl<AUTH> Client<AUTH> {
    /// Check if note transport connection is configured
    pub fn is_note_transport_enabled(&self) -> bool {
        self.note_transport_api.is_some()
    }

    /// Returns the Note Transport client
    ///
    /// Errors if the note transport is not configured.
    pub(crate) fn get_note_transport_api(
        &self,
    ) -> Result<Arc<dyn NoteTransportClient>, NoteTransportError> {
        self.note_transport_api.clone().ok_or(NoteTransportError::Disabled)
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

        let header = note.header().clone();
        let details = NoteDetails::from(note);
        let details_bytes = details.to_bytes();
        // e2ee impl hint:
        // address.key().encrypt(details_bytes)
        api.send_note(header, details_bytes).await?;

        Ok(())
    }
}

impl<AUTH> Client<AUTH>
where
    AUTH: TransactionAuthenticator + Sync + 'static,
{
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
        // Unique tags
        let note_tags = self.store.get_unique_note_tags().await?;
        // Get global cursor
        let cursor = self.store.get_note_transport_cursor().await?;

        self.fetch_transport_notes(cursor, note_tags).await?;

        Ok(())
    }

    /// Fetches all notes for tracked note tags.
    ///
    /// Similar to [`Client::fetch_private_notes`] however does not employ pagination,
    /// fetching all notes stored in the note transport network for the tracked tags.
    /// Please prefer using [`Client::fetch_private_notes`] to avoid downloading repeated notes.
    pub async fn fetch_all_private_notes(&mut self) -> Result<(), ClientError> {
        let note_tags = self.store.get_unique_note_tags().await?;

        self.fetch_transport_notes(NoteTransportCursor::init(), note_tags).await?;

        Ok(())
    }

    /// Fetch notes from the note transport network for provided note tags
    ///
    /// Pagination is employed, where only notes after the provided cursor are requested.
    /// Downloaded notes are imported.
    pub(crate) async fn fetch_transport_notes<I>(
        &mut self,
        cursor: NoteTransportCursor,
        tags: I,
    ) -> Result<(), ClientError>
    where
        I: IntoIterator<Item = NoteTag>,
    {
        let mut notes = Vec::new();
        // Fetch notes
        let (note_infos, rcursor) = self
            .get_note_transport_api()?
            .fetch_notes(&tags.into_iter().collect::<Vec<_>>(), cursor)
            .await?;
        for note_info in &note_infos {
            // e2ee impl hint:
            // for key in self.store.decryption_keys() try
            // key.decrypt(details_bytes_encrypted)
            let note = rejoin_note(&note_info.header, &note_info.details_bytes)?;
            notes.push(note);
        }

        // Import fetched notes.
        // Use block 0 so that check_expected_notes() scans the entire chain.
        // Transport-fetched notes may have been committed before the recipient's
        // current sync height.
        let mut note_requests = Vec::with_capacity(notes.len());
        for note in notes {
            let tag = note.metadata().tag();
            let note_file = NoteFile::NoteDetails {
                details: note.into(),
                after_block_num: BlockNumber::from(0u32),
                tag: Some(tag),
            };
            note_requests.push(note_file);
        }
        self.import_notes(&note_requests).await?;

        // Update cursor (pagination)
        self.store.update_note_transport_cursor(rcursor).await?;

        Ok(())
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
    pub notes: Vec<Note>,
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

/// Information about a note fetched from the note transport network
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

impl Serializable for NoteTransportCursor {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.0.write_into(target);
    }
}

impl Deserializable for NoteTransportCursor {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let value = u64::read_from(source)?;
        Ok(Self::new(value))
    }
}

fn rejoin_note(header: &NoteHeader, details_bytes: &[u8]) -> Result<Note, DeserializationError> {
    let mut reader = SliceReader::new(details_bytes);
    let details = NoteDetails::read_from(&mut reader)?;
    let metadata = header.metadata().clone();
    Ok(Note::new(details.assets().clone(), metadata, details.recipient().clone()))
}
