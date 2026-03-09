use miden_node_proto::generated::remote_prover::{
    self as proto,
    api_server,
    worker_status_api_server,
};
use miden_node_utils::cors::cors_for_grpc_web_layer;
use miden_protocol::utils::{Deserializable, Serializable};
use miden_tx::LocalTransactionProver;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic_web::GrpcWebLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

const DEFAULT_PROVER_PORT: u16 = 50051;

// PROVER SERVICE
// ================================================================================================

struct TransactionProverService {
    prover: tokio::sync::Mutex<LocalTransactionProver>,
}

impl TransactionProverService {
    fn new() -> Self {
        Self {
            prover: tokio::sync::Mutex::new(LocalTransactionProver::default()),
        }
    }
}

#[async_trait::async_trait]
impl api_server::Api for TransactionProverService {
    async fn prove(
        &self,
        request: tonic::Request<proto::ProofRequest>,
    ) -> Result<tonic::Response<proto::Proof>, tonic::Status> {
        let request = request.into_inner();

        let inputs =
            miden_protocol::transaction::TransactionInputs::read_from_bytes(&request.payload)
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("failed to decode request: {e}"))
                })?;

        let prover = self.prover.lock().await;
        let proven_tx = tokio::task::block_in_place(|| prover.prove(inputs))
            .map_err(|e| tonic::Status::internal(format!("failed to prove transaction: {e}")))?;

        Ok(tonic::Response::new(proto::Proof { payload: proven_tx.to_bytes() }))
    }
}

// STATUS SERVICE
// ================================================================================================

struct StatusService;

#[async_trait::async_trait]
impl worker_status_api_server::WorkerStatusApi for StatusService {
    async fn status(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::WorkerStatus>, tonic::Status> {
        Ok(tonic::Response::new(proto::WorkerStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            supported_proof_type: proto::ProofType::Transaction as i32,
        }))
    }
}

// MAIN
// ================================================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)?;

    let addr = format!("127.0.0.1:{DEFAULT_PROVER_PORT}");
    let listener = TcpListener::bind(&addr).await?;
    println!("Remote prover listening on {}", listener.local_addr()?);

    let api_service = api_server::ApiServer::new(TransactionProverService::new());
    let status_service = worker_status_api_server::WorkerStatusApiServer::new(StatusService);

    tonic::transport::Server::builder()
        .accept_http1(true)
        .layer(cors_for_grpc_web_layer())
        .layer(GrpcWebLayer::new())
        .add_service(api_service)
        .add_service(status_service)
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await?;

    Ok(())
}
