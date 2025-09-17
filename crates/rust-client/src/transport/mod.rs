pub mod errors;
#[cfg(any(feature = "tonic", feature = "web-tonic"))]
pub mod grpc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use futures::Stream;
use miden_lib::utils::{Deserializable, DeserializationError, Serializable};
use miden_objects::address::Address;
use miden_objects::note::{Note, NoteDetails, NoteHeader, NoteTag};
use miden_tx::auth::TransactionAuthenticator;

pub use self::errors::TransportError;
use crate::{Client, ClientError};

/// Client transport layer methods.
impl<'a, AUTH> Client<AUTH>
where
    AUTH: TransactionAuthenticator + Sync + 'static,
{
    pub fn transport_layer(&'a mut self) -> TransportLayer<'a, AUTH> {
        TransportLayer::new(self)
    }
}

/// Transport layer methods
pub struct TransportLayer<'a, AUTH> {
    client: &'a mut Client<AUTH>,
}

impl<'a, AUTH> TransportLayer<'a, AUTH> {
    pub fn new(client: &'a mut Client<AUTH>) -> Self {
        Self { client }
    }

    pub fn is_enabled(&self) -> bool {
        self.client.transport_api.is_some()
    }

    /// Send a note
    pub async fn send_note(&mut self, note: Note, _address: &Address) -> Result<(), ClientError> {
        if let Some(transport) = self.client.transport_api.as_mut() {
            let header = *note.header();
            let details = NoteDetails::from(note);
            let details_bytes = details.to_bytes();
            // e2ee impl hint:
            // address.key().encrypt(details_bytes)
            transport.send_note(header, details_bytes).await?;
        }
        Ok(())
    }

    /// Fetch notes for tracked note tags
    pub async fn fetch_notes(&mut self) -> Result<(), ClientError> {
        let note_tag_records = self.client.store.get_note_tags().await?;
        // Unique tags, cursors
        let note_tags_pg = note_tag_records
            .iter()
            .map(|record| (record.tag, record.transport_layer_cursor.unwrap_or(0)))
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .collect::<Vec<_>>();

        self.fetch_notes_pg(&note_tags_pg).await
    }

    /// Fetch notes for provided note tags with pagination
    ///
    /// Only notes after the pairing cursor for each tag are fetched.
    pub(crate) async fn fetch_notes_pg<'t, I>(&mut self, pg_tags: I) -> Result<(), ClientError>
    where
        I: IntoIterator<Item = &'t (NoteTag, u64)>,
    {
        if let Some(transport) = self.client.transport_api.as_mut() {
            let mut notes_to_store = vec![];
            // Fetch notes for all tracked tags
            for &(tag, cursor) in pg_tags {
                let note_infos = transport.fetch_notes(tag, cursor).await?;
                let mut latest_cursor = cursor;
                for note_info in &note_infos {
                    // e2ee impl hint:
                    // for key in self.store.decryption_keys() try
                    // key.decrypt(details_bytes_encrypted)
                    let note = rejoin_note(&note_info.header, &note_info.details_bytes)?;
                    notes_to_store.push(note.into());
                    latest_cursor = latest_cursor.max(note_info.cursor);
                }
                if latest_cursor > cursor {
                    self.client.store.update_note_tag_cursor(tag, latest_cursor).await?;
                }
            }
            // Store fetched notes
            self.client.store.upsert_input_notes(&notes_to_store).await?;
        }
        Ok(())
    }
}

/// The main transport client trait for sending and receiving encrypted notes
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait NoteTransportClient: Send + Sync {
    /// Send a note with optionally encrypted details
    async fn send_note(
        &mut self,
        header: NoteHeader,
        details: Vec<u8>,
    ) -> Result<(), TransportError>;

    /// Fetch notes for a given tag
    ///
    /// Only notes after the given cursor will be fetched.
    async fn fetch_notes(
        &mut self,
        tag: NoteTag,
        cursor: u64,
    ) -> Result<Vec<NoteInfo>, TransportError>;

    /// Stream notes for a given tag
    async fn stream_notes(
        &mut self,
        tag: NoteTag,
        cursor: u64,
    ) -> Result<Box<dyn NoteStream>, TransportError>;
}

/// Stream trait for note streaming
pub trait NoteStream: Stream<Item = Result<Vec<NoteInfo>, TransportError>> + Send + Unpin {}

/// Information about a note in API responses
#[derive(Debug, Clone)]
pub struct NoteInfo {
    /// Note header
    pub header: NoteHeader,
    /// Note details, can be encrypted
    pub details_bytes: Vec<u8>,
    /// Note transport layer cursor
    pub cursor: u64,
}

fn rejoin_note(header: &NoteHeader, details_bytes: &[u8]) -> Result<Note, DeserializationError> {
    let details = NoteDetails::read_from_bytes(details_bytes)?;
    let metadata = *header.metadata();
    Ok(Note::new(details.assets().clone(), metadata, details.recipient().clone()))
}
