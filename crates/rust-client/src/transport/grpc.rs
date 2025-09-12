use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::pin::Pin;
use core::task::{Context, Poll};

use anyhow::anyhow as err;
use futures::Stream;
use miden_objects::note::{NoteHeader, NoteTag};
use miden_objects::utils::{Deserializable, Serializable};
use miden_private_transport_proto::miden_private_transport::miden_private_transport_client::MidenPrivateTransportClient;
use miden_private_transport_proto::miden_private_transport::{
    FetchNotesRequest,
    SendNoteRequest,
    StreamNotesRequest,
    StreamNotesUpdate,
    TransportNote,
};
use tonic::{Request, Streaming};
use tonic_health::pb::HealthCheckRequest;
use tonic_health::pb::health_client::HealthClient;
#[cfg(not(target_arch = "wasm32"))]
use {
    std::time::Duration,
    tonic::transport::{Channel, ClientTlsConfig},
    tower::timeout::Timeout,
};

use super::{NoteInfo, NoteStream, TransportError};

#[cfg(not(target_arch = "wasm32"))]
type Service = Timeout<Channel>;
#[cfg(target_arch = "wasm32")]
type Service = tonic_web_wasm_client::Client;

/// gRPC client
#[derive(Clone)]
pub struct CanonicalNoteTransportClient {
    client: MidenPrivateTransportClient<Service>,
    health_client: HealthClient<Service>,
}

impl CanonicalNoteTransportClient {
    /// gRPC client constructor
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn connect(endpoint: String, timeout_ms: u64) -> Result<Self, TransportError> {
        let tls = ClientTlsConfig::new().with_native_roots();
        let channel = Channel::from_shared(endpoint.clone())
            .map_err(|e| err!("Failed to create channel: {e}"))?
            .tls_config(tls)
            .map_err(|e| err!("Failed to setup TLS: {e}"))?
            .connect()
            .await
            .map_err(|e| err!("Failed to connect: {e}"))?;
        let timeout = Duration::from_millis(timeout_ms);
        let timeout_channel = Timeout::new(channel, timeout);
        let health_client = HealthClient::new(timeout_channel.clone());
        let client = MidenPrivateTransportClient::new(timeout_channel);

        Ok(Self { client, health_client })
    }

    /// gRPC client (WASM) constructor
    #[cfg(target_arch = "wasm32")]
    pub async fn connect(endpoint: String, _timeout_ms: u64) -> Result<Self, TransportError> {
        let wasm_client = tonic_web_wasm_client::Client::new(endpoint);
        let health_client = HealthClient::new(wasm_client.clone());
        let client = MidenPrivateTransportClient::new(wasm_client);

        Ok(Self { client, health_client })
    }

    /// Send a note
    ///
    /// Pushes a note to the transport layer.
    /// While the note header goes in plaintext, the provided note details can be encrypted.
    pub async fn send_note(
        &mut self,
        header: NoteHeader,
        details: Vec<u8>,
    ) -> Result<(), TransportError> {
        let request = SendNoteRequest {
            note: Some(TransportNote { header: header.to_bytes(), details }),
        };

        self.client
            .send_note(Request::new(request))
            .await
            .map_err(|e| err!("Send note failed: {e:?}"))?;

        Ok(())
    }

    /// Fetch notes
    ///
    /// Downloads notes for a given tag.
    /// Only notes labelled after the provided cursor are returned.
    pub async fn fetch_notes(
        &mut self,
        tag: NoteTag,
        cursor: u64,
    ) -> Result<Vec<NoteInfo>, TransportError> {
        let request = FetchNotesRequest { tag: tag.as_u32(), cursor };

        let response = self
            .client
            .fetch_notes(Request::new(request))
            .await
            .map_err(|e| err!("Fetch notes failed: {e:?}"))?;

        let response = response.into_inner();

        // Convert protobuf notes to internal format and track the most recent received timestamp
        let mut notes = Vec::new();

        for pg_note in response.notes {
            let note = pg_note.note.ok_or_else(|| err!("Fetched note has no data".to_string()))?;
            let header = NoteHeader::read_from_bytes(&note.header)
                .map_err(|e| err!("Invalid note header: {e:?}"))?;

            notes.push(NoteInfo {
                header,
                details_bytes: note.details,
                cursor: pg_note.cursor,
            });
        }

        Ok(notes)
    }

    /// Stream notes
    ///
    /// Subscribes to a given tag.
    /// New notes are received periodically.
    pub async fn stream_notes(
        &mut self,
        tag: NoteTag,
        cursor: u64,
    ) -> Result<NoteStreamAdapter, TransportError> {
        let request = StreamNotesRequest { tag: tag.as_u32(), cursor };

        let response = self
            .client
            .stream_notes(request)
            .await
            .map_err(|e| err!("Stream notes failed: {e:?}"))?;
        Ok(NoteStreamAdapter::new(response.into_inner()))
    }

    /// gRPC-standardized server health-check
    pub async fn health_check(&mut self) -> Result<(), TransportError> {
        let request = tonic::Request::new(HealthCheckRequest {
            service: String::new(), // empty string -> whole server
        });

        let response = self
            .health_client
            .check(request)
            .await
            .map_err(|e| err!("Health check failed: {e}"))?
            .into_inner();

        let serving = matches!(
            response.status(),
            tonic_health::pb::health_check_response::ServingStatus::Serving
        );

        serving.then_some(()).ok_or_else(|| err!("Service is not serving").into())
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl super::NoteTransportClient for CanonicalNoteTransportClient {
    async fn send_note(
        &mut self,
        header: NoteHeader,
        details: Vec<u8>,
    ) -> Result<(), TransportError> {
        self.send_note(header, details).await
    }

    async fn fetch_notes(
        &mut self,
        tag: NoteTag,
        cursor: u64,
    ) -> Result<Vec<NoteInfo>, TransportError> {
        self.fetch_notes(tag, cursor).await
    }

    async fn stream_notes(
        &mut self,
        tag: NoteTag,
        cursor: u64,
    ) -> Result<Box<dyn NoteStream>, TransportError> {
        let stream = self.stream_notes(tag, cursor).await?;
        Ok(Box::new(stream))
    }
}

/// Convert from `tonic::Streaming<StreamNotesUpdate>` to [`NoteStream`]
pub struct NoteStreamAdapter {
    inner: Streaming<StreamNotesUpdate>,
}

impl NoteStreamAdapter {
    /// Create a new [`NoteStreamAdapter`]
    pub fn new(stream: Streaming<StreamNotesUpdate>) -> Self {
        Self { inner: stream }
    }
}

impl Stream for NoteStreamAdapter {
    type Item = Result<Vec<NoteInfo>, TransportError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(update))) => {
                // Convert StreamNotesUpdate to Vec<NoteInfo>
                let mut notes = Vec::new();
                for pg_note in update.notes {
                    if let Some(note) = pg_note.note {
                        let header = NoteHeader::read_from_bytes(&note.header)
                            .map_err(|e| err!("Invalid note header: {e:?}"))?;

                        notes.push(NoteInfo {
                            header,
                            details_bytes: note.details,
                            cursor: pg_note.cursor,
                        });
                    }
                }
                Poll::Ready(Some(Ok(notes)))
            },
            Poll::Ready(Some(Err(status))) => {
                Poll::Ready(Some(Err(err!("tonic status: {status}").into())))
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl NoteStream for NoteStreamAdapter {}
