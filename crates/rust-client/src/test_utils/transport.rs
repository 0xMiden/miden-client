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

use crate::transport::{NoteInfo, NoteStream, NoteTransportClient, NoteTransportError};

/// Mock Note Transport Node
///
/// Simulates the functionality of the transport layer node.
#[derive(Clone)]
pub struct MockNoteTransportNode {
    notes: BTreeMap<NoteTag, Vec<(NoteInfo, u64)>>,
}

impl MockNoteTransportNode {
    pub fn new() -> Self {
        Self { notes: BTreeMap::default() }
    }

    pub fn add_note(&mut self, header: NoteHeader, details_bytes: Vec<u8>) {
        let info = NoteInfo { header, details_bytes };
        let cursor = Utc::now().timestamp_micros().try_into().unwrap();
        self.notes.entry(header.metadata().tag()).or_default().push((info, cursor));
    }

    pub fn get_notes(&self, tags: &[NoteTag], cursor: u64) -> (Vec<NoteInfo>, u64) {
        let mut notes = vec![];
        let mut rcursor = 0;
        for tag in tags {
            // Assumes stored notes are ordered by cursor
            let tnotes = self
                .notes
                .get(tag)
                .map(|pg_notes| {
                    // Find first element after cursor
                    if let Some(pos) = pg_notes.iter().position(|(_, tcursor)| *tcursor > cursor) {
                        &pg_notes[pos..]
                    } else {
                        &[]
                    }
                })
                .map(Vec::from)
                .unwrap_or_default();
            rcursor = rcursor.max(tnotes.iter().map(|(_, cursor)| *cursor).max().unwrap_or(0));
            notes.extend(tnotes.into_iter().map(|(note, _)| note).collect::<Vec<_>>());
        }
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
/// Simulates communications with the transport layer node.
pub struct MockNoteTransportApi {
    mock_node: Arc<RwLock<MockNoteTransportNode>>,
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

    pub fn fetch_notes(&self, tags: &[NoteTag], cursor: u64) -> (Vec<NoteInfo>, u64) {
        self.mock_node.read().get_notes(tags, cursor)
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
        cursor: u64,
    ) -> Result<(Vec<NoteInfo>, u64), NoteTransportError> {
        Ok(self.fetch_notes(tags, cursor))
    }

    async fn stream_notes(
        &self,
        _tag: NoteTag,
        _cursor: u64,
    ) -> Result<Box<dyn NoteStream>, NoteTransportError> {
        Ok(Box::new(DummyNoteStream {}))
    }
}
