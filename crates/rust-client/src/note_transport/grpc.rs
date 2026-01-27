use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::pin::Pin;
use core::task::{Context, Poll};

use futures::Stream;
use miden_protocol::note::{NoteHeader, NoteTag};
use miden_protocol::utils::{Deserializable, Serializable};
use miden_tx::utils::sync::RwLock;
use tonic::{Request, Streaming};
use tonic_health::pb::HealthCheckRequest;
use tonic_health::pb::health_client::HealthClient;
#[cfg(all(not(target_arch = "wasm32"), feature = "std"))]
use {
    std::time::Duration,
    tonic::transport::{Channel, ClientTlsConfig},
};

use super::generated::miden_note_transport::miden_note_transport_client::MidenNoteTransportClient;
use super::generated::miden_note_transport::{
    FetchNotesRequest,
    SendNoteRequest,
    StreamNotesRequest,
    StreamNotesUpdate,
    TransportNote,
};
use super::{NoteInfo, NoteStream, NoteTransportCursor, NoteTransportError};

#[cfg(all(not(target_arch = "wasm32"), feature = "std"))]
type Service = Channel;
#[cfg(any(target_arch = "wasm32", not(feature = "std")))]
type Service = tonic_web_wasm_client::Client;

/// gRPC client
pub struct GrpcNoteTransportClient {
    client: RwLock<MidenNoteTransportClient<Service>>,
    health_client: RwLock<HealthClient<Service>>,
}

impl GrpcNoteTransportClient {
    /// gRPC client constructor
    #[cfg(all(not(target_arch = "wasm32"), feature = "std"))]
    pub async fn connect(endpoint: String, timeout_ms: u64) -> Result<Self, NoteTransportError> {
        let endpoint = tonic::transport::Endpoint::try_from(endpoint)
            .map_err(|e| NoteTransportError::Connection(Box::new(e)))?
            .timeout(Duration::from_millis(timeout_ms));
        let tls = ClientTlsConfig::new().with_native_roots();
        let channel = endpoint
            .tls_config(tls)
            .map_err(|e| NoteTransportError::Connection(Box::new(e)))?
            .connect()
            .await
            .map_err(|e| NoteTransportError::Connection(Box::new(e)))?;
        let health_client = HealthClient::new(channel.clone());
        let client = MidenNoteTransportClient::new(channel);

        Ok(Self {
            client: RwLock::new(client),
            health_client: RwLock::new(health_client),
        })
    }

    /// gRPC client (WASM or no-std) constructor
    #[cfg(any(target_arch = "wasm32", not(feature = "std")))]
    pub fn new(endpoint: String) -> Self {
        let wasm_client = tonic_web_wasm_client::Client::new(endpoint);
        let health_client = HealthClient::new(wasm_client.clone());
        let client = MidenNoteTransportClient::new(wasm_client);

        Self {
            client: RwLock::new(client),
            health_client: RwLock::new(health_client),
        }
    }

    /// Get a lock to the main client
    fn api(&self) -> MidenNoteTransportClient<Service> {
        self.client.read().clone()
    }

    /// Get a lock to the health client
    fn health_api(&self) -> HealthClient<Service> {
        self.health_client.read().clone()
    }

    /// Pushes a note to the note transport network.
    ///
    /// While the note header goes in plaintext, the provided note details can be encrypted.
    pub async fn send_note(
        &self,
        header: NoteHeader,
        details: Vec<u8>,
    ) -> Result<(), NoteTransportError> {
        let request = SendNoteRequest {
            note: Some(TransportNote { header: header.to_bytes(), details }),
        };

        self.api()
            .send_note(Request::new(request))
            .await
            .map_err(|e| NoteTransportError::Network(format!("Send note failed: {e:?}")))?;

        Ok(())
    }

    /// Downloads notes for given tags from the note transport network.
    ///
    /// Returns notes labeled after the provided cursor (pagination), and an updated cursor.
    pub async fn fetch_notes(
        &self,
        tags: &[NoteTag],
        cursor: NoteTransportCursor,
    ) -> Result<(Vec<NoteInfo>, NoteTransportCursor), NoteTransportError> {
        let tags_int = tags.iter().map(NoteTag::as_u32).collect();
        let request = FetchNotesRequest { tags: tags_int, cursor: cursor.value() };

        let response = self
            .api()
            .fetch_notes(Request::new(request))
            .await
            .map_err(|e| NoteTransportError::Network(format!("Fetch notes failed: {e:?}")))?;

        let response = response.into_inner();

        // Convert protobuf notes to internal format and track the most recent received timestamp
        let mut notes = Vec::new();

        for pnote in response.notes {
            let header = NoteHeader::read_from_bytes(&pnote.header)?;

            notes.push(NoteInfo { header, details_bytes: pnote.details });
        }

        Ok((notes, response.cursor.into()))
    }

    /// Stream notes from the note transport network.
    ///
    /// Subscribes to a given tag.
    /// New notes are received periodically.
    pub async fn stream_notes(
        &self,
        tag: NoteTag,
        cursor: NoteTransportCursor,
    ) -> Result<NoteStreamAdapter, NoteTransportError> {
        let request = StreamNotesRequest {
            tag: tag.as_u32(),
            cursor: cursor.value(),
        };

        let response = self
            .api()
            .stream_notes(request)
            .await
            .map_err(|e| NoteTransportError::Network(format!("Stream notes failed: {e:?}")))?;
        Ok(NoteStreamAdapter::new(response.into_inner()))
    }

    /// gRPC-standardized server health-check.
    ///
    /// Checks if the note transport node and respective gRPC services are serving requests.
    /// Currently the grPC server operates only one service labelled `MidenNoteTransport`.
    pub async fn health_check(&mut self) -> Result<(), NoteTransportError> {
        let request = tonic::Request::new(HealthCheckRequest {
            service: String::new(), // empty string -> whole server
        });

        let response = self
            .health_api()
            .check(request)
            .await
            .map_err(|e| NoteTransportError::Network(format!("Health check failed: {e}")))?
            .into_inner();

        let serving = matches!(
            response.status(),
            tonic_health::pb::health_check_response::ServingStatus::Serving
        );

        serving
            .then_some(())
            .ok_or_else(|| NoteTransportError::Network("Service is not serving".into()))
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl super::NoteTransportClient for GrpcNoteTransportClient {
    async fn send_note(
        &self,
        header: NoteHeader,
        details: Vec<u8>,
    ) -> Result<(), NoteTransportError> {
        self.send_note(header, details).await
    }

    async fn fetch_notes(
        &self,
        tags: &[NoteTag],
        cursor: NoteTransportCursor,
    ) -> Result<(Vec<NoteInfo>, NoteTransportCursor), NoteTransportError> {
        self.fetch_notes(tags, cursor).await
    }

    async fn stream_notes(
        &self,
        tag: NoteTag,
        cursor: NoteTransportCursor,
    ) -> Result<Box<dyn NoteStream>, NoteTransportError> {
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
    type Item = Result<Vec<NoteInfo>, NoteTransportError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(update))) => {
                // Convert StreamNotesUpdate to Vec<NoteInfo>
                let mut notes = Vec::new();
                for pnote in update.notes {
                    let header = NoteHeader::read_from_bytes(&pnote.header)?;

                    notes.push(NoteInfo { header, details_bytes: pnote.details });
                }
                Poll::Ready(Some(Ok(notes)))
            },
            Poll::Ready(Some(Err(status))) => Poll::Ready(Some(Err(NoteTransportError::Network(
                format!("tonic status: {status}"),
            )))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl NoteStream for NoteStreamAdapter {}
