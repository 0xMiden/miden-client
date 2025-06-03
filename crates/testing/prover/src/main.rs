use anyhow::Error;
use miden_proving_service::{ProverType, RpcListener, setup_tracing};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;

const DEFAULT_PROVER_PORT: u16 = 50051;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing().map_err(|err| Error::msg(err.to_string()))?;

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
