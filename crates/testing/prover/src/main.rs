use miden_node_utils::cors::cors_for_grpc_web_layer;
use miden_remote_prover::api::{ProofType, RpcListener};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic_web::GrpcWebLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

const DEFAULT_PROVER_PORT: u16 = 50051;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)?;

    let addr = format!("127.0.0.1:{DEFAULT_PROVER_PORT}");
    let rpc = RpcListener::new(TcpListener::bind(&addr).await?, ProofType::Transaction);

    println!("Remote prover listening on {}", rpc.listener.local_addr()?);

    tonic::transport::Server::builder()
        .accept_http1(true)
        .layer(cors_for_grpc_web_layer())
        .layer(GrpcWebLayer::new())
        .add_service(rpc.api_service)
        .add_service(rpc.status_service)
        .serve_with_incoming(TcpListenerStream::new(rpc.listener))
        .await?;

    Ok(())
}
