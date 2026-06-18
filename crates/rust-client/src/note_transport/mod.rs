pub mod errors;
pub mod generated;
#[cfg(feature = "tonic")]
pub mod grpc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use futures::Stream;
use miden_protocol::address::Address;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::{
    Note,
    NoteDetails,
    NoteDetailsCommitment,
    NoteFile,
    NoteHeader,
    NoteId,
    NoteTag,
};
use miden_protocol::utils::serde::Serializable;
use miden_tx::auth::TransactionAuthenticator;
use miden_tx::utils::serde::{
    ByteReader,
    ByteWriter,
    Deserializable,
    DeserializationError,
    SliceReader,
};

pub use self::errors::NoteTransportError;
use crate::store::{NoteFilter, OutputNoteRecord};
use crate::{Client, ClientError};

pub const NOTE_TRANSPORT_TESTNET_ENDPOINT: &str = "https://transport.miden.io";
pub const NOTE_TRANSPORT_DEVNET_ENDPOINT: &str = "https://transport.devnet.miden.io";
pub const NOTE_TRANSPORT_CURSOR_STORE_SETTING: &str = "note_transport_cursor";

/// Settings key for the durable relay outbox: a serialized `Vec<NoteInfo>` of
/// private notes whose transport delivery has not yet succeeded.
/// `send_private_note` appends (replacing any entry with the same note id)
/// before relaying; [`Client::flush_relay_outbox`] drains entries that re-send
/// successfully. Reusing the settings k/v avoids a Store-trait schema change
/// while surviving process restarts.
pub const NOTE_TRANSPORT_OUTBOX_KEY: &str = "note_transport_outbox";

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
    ///
    /// **Durability.** The relay payload is persisted to the outbox before the
    /// transport call. If the call fails or is interrupted, the entry stays in
    /// the outbox and is retried on the next [`Client::flush_relay_outbox`]
    /// (which [`Client::sync_note_transport`] runs), so a transient transport
    /// failure does not drop the note. The receiver dedupes by note id, so a
    /// re-send after a partial success is harmless.
    pub async fn send_private_note(
        &mut self,
        note: Note,
        _address: &Address,
    ) -> Result<(), ClientError> {
        let api = self.get_note_transport_api()?;

        let header = *note.header();
        let note_id = header.id();
        let details = NoteDetails::from(note);
        let details_bytes = details.to_bytes();
        // e2ee impl hint:
        // address.key().encrypt(details_bytes)

        // The note's creating transaction cannot commit before the block at which it was
        // submitted, so the output note's `expected_height` (the submission height) is a safe
        // lower bound on the note's on-chain commitment block: relaying it lets the recipient
        // scan from exactly there rather than guess with a lookback window. `None` when the note
        // isn't tracked as an output note here, in which case the recipient falls back to its own
        // lookback heuristic.
        let after_block_num = self
            .store
            .get_output_notes(NoteFilter::List(Vec::from([note_id])))
            .await?
            .first()
            .map(OutputNoteRecord::expected_height);

        // Persist the payload before the network call so a failed or
        // interrupted `send_note` leaves a recoverable record rather than
        // losing the only copy with the call frame.
        let entry = NoteInfo {
            header,
            details_bytes: details_bytes.clone(),
            after_block_num,
        };
        let mut outbox = self.load_relay_outbox().await?;
        // Replace any existing entry for this note id so the latest payload
        // wins when a still-pending note is re-sent.
        outbox.retain(|e| e.header.id() != note_id);
        outbox.push(entry);
        self.save_relay_outbox(outbox).await?;

        api.send_note(header, details_bytes, after_block_num).await?;

        // Relay succeeded — drop the entry. A failed store write here is
        // tolerable: the next flush re-sends and the receiver dedupes by note
        // id, so a stale entry never causes loss.
        let mut outbox = self.load_relay_outbox().await?;
        outbox.retain(|e| e.header.id() != note_id);
        self.save_relay_outbox(outbox).await?;

        Ok(())
    }

    /// Re-attempt every relay payload in the durable outbox. Each entry is a
    /// private note whose previous transport delivery failed. Successful
    /// re-sends are dropped; failures are kept for the next call. Every entry
    /// is attempted independently, so one persistently-failing note does not
    /// block the others.
    ///
    /// [`Client::sync_note_transport`] runs this automatically and ignores its
    /// error, so a relay failure can't block a sync. Callers driving retries
    /// themselves can invoke it directly and inspect the returned error.
    pub async fn flush_relay_outbox(&self) -> Result<(), ClientError> {
        let api = self.get_note_transport_api()?;

        let entries = self.load_relay_outbox().await?;
        if entries.is_empty() {
            return Ok(());
        }

        // Attempt every entry independently so a single persistently-failing
        // note can't block the rest. The outbox holds only the caller's own
        // failed sends, so it stays small and this is not a meaningful burst.
        let mut remaining = Vec::new();
        let mut last_err: Option<NoteTransportError> = None;

        for entry in entries {
            match api
                .send_note(entry.header, entry.details_bytes.clone(), entry.after_block_num)
                .await
            {
                Ok(()) => {},
                Err(err) => {
                    tracing::warn!(?err, "relay-outbox entry retry failed; will retry next sync");
                    remaining.push(entry);
                    last_err = Some(err);
                },
            }
        }

        self.save_relay_outbox(remaining).await?;

        if let Some(err) = last_err {
            return Err(err.into());
        }
        Ok(())
    }

    /// Load the durable relay outbox.
    ///
    /// Returns an empty `Vec` if the outbox key is absent. On deserialization
    /// failure (schema mismatch or storage corruption) the entry is dropped and
    /// an empty `Vec` is returned — leaving unreadable bytes in place would
    /// block every subsequent relay because each sync would re-read them.
    async fn load_relay_outbox(&self) -> Result<Vec<NoteInfo>, ClientError> {
        let bytes = self
            .store
            .get_setting(String::from(NOTE_TRANSPORT_OUTBOX_KEY))
            .await
            .map_err(ClientError::StoreError)?;
        let Some(bytes) = bytes else {
            return Ok(Vec::new());
        };
        match Vec::<NoteInfo>::read_from_bytes(&bytes) {
            Ok(entries) => Ok(entries),
            Err(err) => {
                // A relay-outbox blob written before `after_block_num` was added to `NoteInfo`
                // lacks the trailing field, so the current decoder rejects it. Read it with the
                // legacy layout (defaulting the hint to `None`) so a still-pending relay survives
                // a client upgrade instead of being dropped as unreadable.
                if let Ok(legacy) = Vec::<LegacyNoteInfo>::read_from_bytes(&bytes) {
                    return Ok(legacy.into_iter().map(NoteInfo::from).collect());
                }
                tracing::warn!(?err, "dropping unreadable relay outbox; resetting to empty");
                self.store
                    .remove_setting(String::from(NOTE_TRANSPORT_OUTBOX_KEY))
                    .await
                    .map_err(ClientError::StoreError)?;
                Ok(Vec::new())
            },
        }
    }

    /// Persist the relay outbox, removing the key entirely when empty so the
    /// settings table doesn't accumulate empty-vec blobs.
    async fn save_relay_outbox(&self, entries: Vec<NoteInfo>) -> Result<(), ClientError> {
        let key = String::from(NOTE_TRANSPORT_OUTBOX_KEY);
        if entries.is_empty() {
            return self.store.remove_setting(key).await.map_err(ClientError::StoreError);
        }
        let bytes = entries.to_bytes();
        self.store.set_setting(key, bytes).await.map_err(ClientError::StoreError)
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
        let note_tags: Vec<NoteTag> =
            self.store.get_unique_note_tags().await?.into_iter().collect();
        let cursor = self.store.get_note_transport_cursor().await?;

        let (_, new_cursor) = self.fetch_transport_notes(cursor, &note_tags).await?;
        self.store.update_note_transport_cursor(new_cursor).await?;

        Ok(())
    }

    /// Fetches all notes for tracked note tags, draining the server's paginated
    /// response by looping until the cursor stops advancing.
    ///
    /// Similar to [`Client::fetch_private_notes`] but ignores the stored
    /// pagination cursor and re-scans from the beginning. The server-side
    /// transport caps each response at a fixed batch size; this method issues
    /// repeated fetch calls until one returns the same cursor it was given
    /// (i.e. no new notes), so the documented "fetches all notes" semantics
    /// hold regardless of how large the backlog is. Prefer
    /// [`Client::fetch_private_notes`] for steady-state syncing to avoid
    /// re-downloading already-seen notes.
    pub async fn fetch_all_private_notes(&mut self) -> Result<(), ClientError> {
        // Safety cap on a misbehaving server. At 500 notes per batch, 1000
        // iterations covers 500k notes — well beyond any plausible retention
        // window — and bounds the worst-case wall-clock at ~50s at 50ms/req.
        // Hitting this signals a server bug, not an honest backlog.
        const MAX_ITERATIONS: usize = 1_000;

        let note_tags: Vec<NoteTag> =
            self.store.get_unique_note_tags().await?.into_iter().collect();
        // Snapshot the stored cursor up front so we can advance (never regress)
        // it after the drain. Without this guard, starting the drain at
        // `init()` and persisting per-batch would clobber a previously
        // advanced cursor with the small `rcursor` of the first batch.
        let stored_cursor = self.store.get_note_transport_cursor().await?;

        let mut cursor = NoteTransportCursor::init();
        for _ in 0..MAX_ITERATIONS {
            let (_, new_cursor) = self.fetch_transport_notes(cursor, &note_tags).await?;
            // Terminate on any lack of forward progress. A well-behaved server
            // returns `new_cursor == cursor` when there are no new notes (since
            // `rcursor = max(cursor, max_seq_returned)`); using `<=` here also
            // handles implementations that return an `init()` cursor on empty
            // batches (see the in-tree mock transport).
            if new_cursor <= cursor {
                let final_cursor = core::cmp::max(cursor, stored_cursor);
                self.store.update_note_transport_cursor(final_cursor).await?;
                return Ok(());
            }
            cursor = new_cursor;
        }

        Err(ClientError::NoteTransportError(NoteTransportError::PaginationDidNotTerminate(
            MAX_ITERATIONS,
        )))
    }

    /// Fetch one batch of notes from the note transport network for the provided tags.
    ///
    /// The server paginates; this method issues one RPC and returns the imported details
    /// commitments together with the new cursor. The returned cursor equals the input cursor when
    /// the batch was empty (i.e. no new notes). Callers that want to drain the full backlog should
    /// loop until `new_cursor == cursor` (see [`Client::fetch_all_private_notes`]). Callers that do
    /// steady-state polling (see [`Client::sync_state`] / [`Client::fetch_private_notes`]) should
    /// call this once per tick with the stored cursor.
    ///
    /// Downloaded notes are imported into the local store. Persistence of the returned cursor is
    /// left to the caller so that drain loops can guard against regression of an already-advanced
    /// stored cursor.
    pub(crate) async fn fetch_transport_notes(
        &mut self,
        cursor: NoteTransportCursor,
        tags: &[NoteTag],
    ) -> Result<(Vec<NoteId>, NoteTransportCursor), ClientError> {
        // Number of blocks to look back from the sync height when scanning for committed notes.
        const NOTE_LOOKBACK_BLOCKS: u32 = 20;

        let (note_infos, rcursor) =
            self.get_note_transport_api()?.fetch_notes(tags, cursor).await?;

        // Steady-state polling fetches empty batches; skip the rest (including the sync-height
        // read) so an empty tick does no extra store work.
        if note_infos.is_empty() {
            return Ok((Vec::new(), rcursor));
        }

        let sync_height = self.get_sync_height().await?;
        // Always scan back at least NOTE_LOOKBACK_BLOCKS from the sync height. This handles the
        // race where a note is committed on-chain just before the NTL delivers its data —
        // without it, the import scan would start at the sync height and miss the already-
        // committed note. A sender-provided `after_block_num` can only lower this floor further
        // (a note committed longer ago than the window), so the scanned range is always a
        // superset of the lookback window: the hint extends coverage but never shrinks it.
        let lookback_floor = sync_height.as_u32().saturating_sub(NOTE_LOOKBACK_BLOCKS);

        let mut notes_with_floor = Vec::with_capacity(note_infos.len());
        for note_info in &note_infos {
            // e2ee impl hint:
            // for key in self.store.decryption_keys() try
            // key.decrypt(details_bytes_encrypted)
            let note = rejoin_note(&note_info.header, &note_info.details_bytes)?;
            let after_block_num = note_info
                .after_block_num
                .map_or(lookback_floor, |hint| hint.as_u32().min(lookback_floor));
            notes_with_floor.push((note, BlockNumber::from(after_block_num)));
        }

        let id_by_commitment: BTreeMap<NoteDetailsCommitment, NoteId> = notes_with_floor
            .iter()
            .map(|(note, _)| (note.details_commitment(), note.id()))
            .collect();

        let mut note_requests = Vec::with_capacity(notes_with_floor.len());
        for (note, after_block_num) in notes_with_floor {
            let tag = note.metadata().tag();
            let note_file = NoteFile::NoteDetails {
                details: note.into(),
                after_block_num,
                tag: Some(tag),
            };
            note_requests.push(note_file);
        }
        let imported_commitments = self.import_notes(&note_requests).await?;
        let imported_ids = imported_commitments
            .into_iter()
            .filter_map(|commitment| id_by_commitment.get(&commitment).copied())
            .collect();

        Ok((imported_ids, rcursor))
    }
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
    ///
    /// `after_block_num` is a sender-provided lower bound on the block at which the note's
    /// on-chain commitment landed; it is relayed verbatim so the recipient can scan from it.
    async fn send_note(
        &self,
        header: NoteHeader,
        details: Vec<u8>,
        after_block_num: Option<BlockNumber>,
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
    /// Sender-provided lower bound on the block at which the note's on-chain commitment landed
    /// (the NTL `after_block_num` wire field). The receiver scans from this block forward when
    /// committing the note. `None` when the sender did not set it, in which case the receiver
    /// falls back to its own lookback heuristic.
    pub after_block_num: Option<BlockNumber>,
}

// SERIALIZATION
// ================================================================================================

impl Serializable for NoteInfo {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.header.write_into(target);
        self.details_bytes.write_into(target);
        self.after_block_num.write_into(target);
    }
}

impl Deserializable for NoteInfo {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let header = NoteHeader::read_from(source)?;
        let details_bytes = Vec::<u8>::read_from(source)?;
        let after_block_num = Option::<BlockNumber>::read_from(source)?;
        Ok(NoteInfo { header, details_bytes, after_block_num })
    }
}

/// The pre-`after_block_num` on-disk layout of [`NoteInfo`] (header + details only). Used only to
/// read relay-outbox blobs written by an earlier client version so a pending relay survives an
/// upgrade; see [`Client::load_relay_outbox`].
struct LegacyNoteInfo {
    header: NoteHeader,
    details_bytes: Vec<u8>,
}

impl Deserializable for LegacyNoteInfo {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let header = NoteHeader::read_from(source)?;
        let details_bytes = Vec::<u8>::read_from(source)?;
        Ok(LegacyNoteInfo { header, details_bytes })
    }
}

impl From<LegacyNoteInfo> for NoteInfo {
    fn from(legacy: LegacyNoteInfo) -> Self {
        NoteInfo {
            header: legacy.header,
            details_bytes: legacy.details_bytes,
            after_block_num: None,
        }
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
    // The transport wire format only carries `NoteHeader` + serialized `NoteDetails`, not the
    // attachments collection. We rejoin with empty attachments; this matches the original note
    // only when it had no attachments in the first place.
    let partial_metadata = *header.metadata().partial_metadata();
    Ok(Note::new(
        details.assets().clone(),
        partial_metadata,
        details.recipient().clone(),
    ))
}

#[cfg(test)]
mod tests {
    use miden_protocol::Felt;
    use miden_protocol::crypto::rand::RandomCoin;
    use miden_protocol::note::NoteType;
    use miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE;
    use miden_protocol::utils::serde::Deserializable;
    use miden_standards::testing::note::NoteBuilder;

    use super::*;

    /// A relay-outbox blob written before `after_block_num` was added (header + details only) must
    /// still be readable so a pending relay survives a client upgrade instead of being dropped.
    #[test]
    fn legacy_outbox_blob_is_recovered_with_no_hint() {
        let account_id = ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE.try_into().unwrap();
        let note = NoteBuilder::new(
            account_id,
            RandomCoin::new([1, 2, 3, 4].map(Felt::new_unchecked).into()),
        )
        .note_type(NoteType::Private)
        .build()
        .unwrap();
        let header = *note.header();
        let details_bytes = NoteDetails::from(note).to_bytes();

        // The pre-`after_block_num` layout is header + details with the same `Vec` framing, which a
        // `(NoteHeader, Vec<u8>)` tuple reproduces byte for byte.
        let old_bytes = vec![(header, details_bytes.clone())].to_bytes();

        // The current decoder rejects the missing trailing field...
        assert!(Vec::<NoteInfo>::read_from_bytes(&old_bytes).is_err());

        // ...but the legacy fallback reads it, defaulting the hint to `None`.
        let recovered: Vec<NoteInfo> = Vec::<LegacyNoteInfo>::read_from_bytes(&old_bytes)
            .unwrap()
            .into_iter()
            .map(NoteInfo::from)
            .collect();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].header.id(), header.id());
        assert_eq!(recovered[0].details_bytes, details_bytes);
        assert_eq!(recovered[0].after_block_num, None);
    }
}
