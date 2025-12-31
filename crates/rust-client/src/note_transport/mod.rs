pub mod errors;
pub mod generated;
#[cfg(feature = "tonic")]
pub mod grpc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use futures::Stream;
use miden_lib::utils::Serializable;
use miden_objects::address::Address;
use miden_objects::crypto::ies::SealedMessage;
use miden_objects::note::{Note, NoteDetails, NoteFile, NoteHeader, NoteTag};
use miden_tx::auth::TransactionAuthenticator;
use miden_tx::utils::Deserializable;
use tracing::debug;

pub use self::errors::NoteTransportError;
use crate::store::Store;
use crate::sync::NoteTagSource;
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
    /// The note will be end-to-end encrypted using the provided recipient's `address` details.
    /// The recipient will be able to retrieve this note through the note's [`NoteTag`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The note transport is not configured
    /// - The recipient's address does not contain an encryption key
    /// - Encryption fails
    /// - Note transport fails
    pub async fn send_private_note(
        &mut self,
        note: Note,
        address: &Address,
    ) -> Result<(), ClientError> {
        let api = self.get_note_transport_api()?;

        let header = *note.header();
        let details = NoteDetails::from(note);
        let details_bytes = details.to_bytes();

        // Encrypt the note details using the recipient's address encryption key
        let encryption_key =
            address.encryption_key().ok_or(NoteTransportError::MissingEncryptionKey)?;

        let mut rng = rand::rng();
        let sealed_message = encryption_key
            .seal_bytes(&mut rng, &details_bytes)
            .map_err(|e| NoteTransportError::EncryptionError(format!("{e:#}")))?;

        api.send_note(header, sealed_message.to_bytes()).await?;

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
    /// end-to-end encryption. Notes that cannot be decrypted are silently ignored.
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
    /// Trial decryption is performed using the encryption key of the address that owns the tag.
    /// Downloaded notes are imported.
    pub(crate) async fn fetch_transport_notes<I>(
        &mut self,
        cursor: NoteTransportCursor,
        tags: I,
    ) -> Result<(), ClientError>
    where
        I: IntoIterator<Item = NoteTag>,
    {
        let tags: Vec<NoteTag> = tags.into_iter().collect();

        // Build a mapping from tag -> address that owns that tag
        let tag_to_address = self.build_tag_to_address_map(&tags).await?;

        let mut notes = Vec::new();

        // Fetch notes
        let (note_infos, rcursor) =
            self.get_note_transport_api()?.fetch_notes(&tags, cursor).await?;

        for note_info in &note_infos {
            // Get the tag from the note header metadata
            let tag = note_info.header.metadata().tag();

            // Try to decrypt with the key of the address that owns this tag
            if let Some(address) = tag_to_address.get(&tag)
                && let Some(note) = self
                    .try_decrypt_note(&note_info.header, &note_info.details_bytes, address)
                    .await
            {
                notes.push(note);
            }
        }

        let sync_height = self.get_sync_height().await?;

        // Import fetched notes
        for note in notes {
            let tag = note.metadata().tag();
            let note_file = NoteFile::NoteDetails {
                details: note.into(),
                after_block_num: sync_height,
                tag: Some(tag),
            };
            self.import_note(note_file).await?;
        }

        // Update cursor (pagination)
        self.store.update_note_transport_cursor(rcursor).await?;

        Ok(())
    }

    /// Builds a mapping from `NoteTag` to the address that owns that tag.
    async fn build_tag_to_address_map(
        &self,
        tags: &[NoteTag],
    ) -> Result<BTreeMap<NoteTag, Address>, ClientError> {
        let note_tag_records = self.store.get_note_tags().await?;
        let mut tag_to_address: BTreeMap<NoteTag, Address> = BTreeMap::new();

        for record in note_tag_records {
            // Only process tags that are in our query set
            if !tags.contains(&record.tag) {
                continue;
            }

            // Get the account that owns this tag
            if let NoteTagSource::Account(account_id) = record.source {
                // Find the address that matches this tag
                let addresses = self.store.get_addresses_by_account_id(account_id).await?;
                for address in addresses {
                    if address.to_note_tag() == record.tag {
                        tag_to_address.insert(record.tag, address);
                        break;
                    }
                }
            }
        }

        Ok(tag_to_address)
    }

    /// Attempts to decrypt a note using the encryption key for the provided address.
    /// Returns `Some(Note)` if decryption succeeds, `None` otherwise.
    async fn try_decrypt_note(
        &self,
        header: &NoteHeader,
        encrypted_details: &[u8],
        address: &Address,
    ) -> Option<Note> {
        let encryption_keystore = self.encryption_keystore.as_ref()?;

        // Parse the sealed message
        let sealed_message = SealedMessage::read_from_bytes(encrypted_details).ok()?;

        // Try decrypt
        let unsealing_key = encryption_keystore.get_encryption_key(address).await.ok()??;
        let details_bytes = unsealing_key.unseal_bytes(sealed_message).ok()?;

        match rejoin_note(header, &details_bytes) {
            Ok(note) => {
                debug!("Successfully decrypted note {}", note.id());
                Some(note)
            },
            Err(e) => {
                debug!("Failed to parse decrypted note details: {:?}", e);
                None
            },
        }
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

impl miden_tx::utils::Serializable for NoteTransportCursor {
    fn write_into<W: miden_tx::utils::ByteWriter>(&self, target: &mut W) {
        target.write_u64(self.0);
    }
}

impl miden_tx::utils::Deserializable for NoteTransportCursor {
    fn read_from<R: miden_tx::utils::ByteReader>(
        source: &mut R,
    ) -> Result<Self, miden_tx::utils::DeserializationError> {
        let value = source.read_u64()?;
        Ok(Self::new(value))
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
        tags: &[NoteTag],
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

/// Represents a note info returned from the note transport layer.
#[derive(Clone)]
pub struct NoteInfo {
    /// Note header (metadata + id).
    pub header: NoteHeader,
    /// Note details serialized as bytes (may be encrypted).
    pub details_bytes: Vec<u8>,
}

// SERIALIZATION
// ================================================================================================

impl miden_tx::utils::Serializable for NoteInfo {
    fn write_into<W: miden_tx::utils::ByteWriter>(&self, target: &mut W) {
        self.header.write_into(target);
        self.details_bytes.write_into(target);
    }
}

impl miden_tx::utils::Deserializable for NoteInfo {
    fn read_from<R: miden_tx::utils::ByteReader>(
        source: &mut R,
    ) -> Result<Self, miden_tx::utils::DeserializationError> {
        let header = NoteHeader::read_from(source)?;
        let details_bytes = Vec::<u8>::read_from(source)?;
        Ok(Self { header, details_bytes })
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Reconstructs a full Note from header and decrypted details bytes.
fn rejoin_note(header: &NoteHeader, details_bytes: &[u8]) -> Result<Note, NoteTransportError> {
    let details = NoteDetails::read_from_bytes(details_bytes)
        .map_err(|e| NoteTransportError::NoteDecodingError(format!("{e:#}")))?;

    let note = Note::new(details.assets().clone(), *header.metadata(), details.recipient().clone());

    // Verify that the reconstructed note matches the header
    if *note.header() != *header {
        return Err(NoteTransportError::NoteReconstructionError(format!(
            "Reconstructed note header doesn't match received header. Got {:?}, expected {:?}",
            note.header(),
            header
        )));
    }

    Ok(note)
}
