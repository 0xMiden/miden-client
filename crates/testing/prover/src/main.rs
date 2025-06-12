use miden_proving_service::api::{ProverType, RpcListener};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

const DEFAULT_PROVER_PORT: u16 = 50051;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)?;

    let addr = format!("127.0.0.1:{DEFAULT_PROVER_PORT}");
    let rpc = RpcListener::new(TcpListener::bind(&addr).await?, ProverType::Transaction);

    println!("Proving service listening on {}", rpc.listener.local_addr()?);

    tonic::transport::Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(rpc.api_service))
        .add_service(tonic_web::enable(rpc.status_service))
        .serve_with_incoming(TcpListenerStream::new(rpc.listener))
        .await?;

    Ok(())
}
