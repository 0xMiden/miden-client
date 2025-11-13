use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::pin::Pin;
use core::task::{Context, Poll};

use chrono::Utc;
use futures::Stream;
use miden_objects::note::{NoteHeader, NoteTag};
use miden_tx::utils::sync::RwLock;
use miden_tx::utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable};

use crate::note_transport::{
    NoteInfo,
    NoteStream,
    NoteTransportClient,
    NoteTransportCursor,
    NoteTransportError,
};

/// Mock Note Transport Node
///
/// Simulates the functionality of the note transport node.
#[derive(Clone)]
pub struct MockNoteTransportNode {
    notes: BTreeMap<NoteTag, Vec<(NoteInfo, NoteTransportCursor)>>,
}

impl MockNoteTransportNode {
    pub fn new() -> Self {
        Self { notes: BTreeMap::default() }
    }

    pub fn add_note(&mut self, header: NoteHeader, details_bytes: Vec<u8>) {
        let info = NoteInfo { header, details_bytes };
        let cursor = u64::try_from(Utc::now().timestamp_micros()).unwrap();
        self.notes
            .entry(header.metadata().tag())
            .or_default()
            .push((info, cursor.into()));
    }

    pub fn get_notes(
        &self,
        tags: &[NoteTag],
        cursor: NoteTransportCursor,
        limit: Option<u32>,
    ) -> (Vec<NoteInfo>, NoteTransportCursor) {
        let mut notesc_unlimited = Vec::new();

        for tag in tags {
            // Assumes stored notes are ordered by cursor
            if let Some(tag_notes) = self.notes.get(tag) {
                // Find first element after cursor
                if let Some(pos) = tag_notes.iter().position(|(_, tcursor)| *tcursor > cursor) {
                    for (note, note_cursor) in &tag_notes[pos..] {
                        notesc_unlimited.push((note.clone(), *note_cursor));
                    }
                }
            }
        }

        // Sort mixed-tagged notes by cursor
        notesc_unlimited.sort_by_key(|(_, cursor)| *cursor);

        // Apply limit if specified
        let limit_usize = limit.map(|l| l as usize);
        let notesc_limited: Vec<_> =
            notesc_unlimited.iter().take(limit_usize.unwrap_or(usize::MAX)).collect();

        let rcursor = notesc_limited.last().map(|(_, cursor)| *cursor).unwrap_or(cursor);

        // Extract notes
        let notes: Vec<NoteInfo> = notesc_limited.iter().map(|(note, _)| note.clone()).collect();

        (notes, rcursor)
    }
}

impl Default for MockNoteTransportNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock Note Transport API
///
/// Simulates communications with the note transport node.
#[derive(Clone, Default)]
pub struct MockNoteTransportApi {
    pub mock_node: Arc<RwLock<MockNoteTransportNode>>,
}

impl MockNoteTransportApi {
    pub fn new(mock_node: Arc<RwLock<MockNoteTransportNode>>) -> Self {
        Self { mock_node }
    }
}

impl MockNoteTransportApi {
    pub fn send_note(&self, header: NoteHeader, details_bytes: Vec<u8>) {
        self.mock_node.write().add_note(header, details_bytes);
    }

    pub fn fetch_notes(
        &self,
        tags: &[NoteTag],
        cursor: NoteTransportCursor,
        limit: Option<u32>,
    ) -> (Vec<NoteInfo>, NoteTransportCursor) {
        self.mock_node.read().get_notes(tags, cursor, limit)
    }
}

pub struct DummyNoteStream {}
impl Stream for DummyNoteStream {
    type Item = Result<Vec<NoteInfo>, NoteTransportError>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(None)
    }
}
impl NoteStream for DummyNoteStream {}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl NoteTransportClient for MockNoteTransportApi {
    async fn send_note(
        &self,
        header: NoteHeader,
        details: Vec<u8>,
    ) -> Result<(), NoteTransportError> {
        self.send_note(header, details);
        Ok(())
    }

    async fn fetch_notes(
        &self,
        tags: &[NoteTag],
        cursor: NoteTransportCursor,
        limit: Option<u32>,
    ) -> Result<(Vec<NoteInfo>, NoteTransportCursor), NoteTransportError> {
        Ok(self.fetch_notes(tags, cursor, limit))
    }

    async fn stream_notes(
        &self,
        _tag: NoteTag,
        _cursor: NoteTransportCursor,
    ) -> Result<Box<dyn NoteStream>, NoteTransportError> {
        Ok(Box::new(DummyNoteStream {}))
    }
}

// SERIALIZATION
// ================================================================================================

impl Serializable for MockNoteTransportNode {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.notes.write_into(target);
    }
}

impl Deserializable for MockNoteTransportNode {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let notes = BTreeMap::<NoteTag, Vec<(NoteInfo, NoteTransportCursor)>>::read_from(source)?;

        Ok(Self { notes })
    }
}
