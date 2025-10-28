# miden-client

`miden-client` is the std-aware runtime that layers orchestration capabilities on top of
[`miden-client-core`](../miden-client-core/README.md). It provides utilities for running the
light client in a background task, coordinating continuous sync, sequencing transactions, and
exposing event handlers that react to on-chain updates.

## Crate layout

- `miden-client-core` exposes the portable building blocks that work in both `no_std` and WASM
  environments.
- `miden-client` (this crate) re-exports the full public API of the core crate and augments it with
  std-only features such as the async service runtime located under [`service`](./src/service).

## Getting started

Add the crate to your `Cargo.toml`:

```toml
miden-client = { version = "0.12", features = ["tonic"] }
```

Instantiate a client service and obtain a handle:

```rust
use std::sync::Arc;
use miden_client::service::{ClientService, ClientServiceConfig};
use miden_client::{builder::ClientBuilder, transaction::TransactionAuthenticator};

async fn start_client<AUTH>(authenticator: Arc<AUTH>)
where
    AUTH: TransactionAuthenticator + Send + Sync + 'static,
{
    let client = ClientBuilder::new()
        .authenticator(authenticator)
        // configure RPC, store, RNG, etc.
        .build()
        .await
        .expect("client");

    let service = ClientService::start(
        client,
        ClientServiceConfig {
            max_parallel_proofs: 4,
            ..Default::default()
        },
    );

    let handle = service.handle();

    // trigger an immediate sync whenever you need fresh data
    let _summary = handle.sync_now().await.expect("sync");

    // submit a transaction request
    let tx_job = handle
        .transaction_service()
        .submit_transaction(account_id, request)
        .await
        .expect("job");

    let executed = tx_job.execution.await.expect("execution finished")?;
    println!("Executed against block {}", executed.block_num());

    // submission completes once the transaction is proven and applied locally in order
    tx_job.completion.await.expect("submission finished")?;

    service.shutdown().await.expect("shutdown");
}
```

> Note: the service runs on `tokio::task::spawn_local`, so make sure it lives inside a `LocalSet`
> (e.g. `tokio::task::LocalSet::run_until`) or a current-thread runtime. Reads can use
> `ClientServiceHandle::store()` to access the shared store without waiting on the command queue.
> Transactions execute sequentially, proofs are processed in parallel, and submissions are
> serialized to keep the client state consistent.

The `service` module is still evolving. Expect additional APIs for handler registration and sync
callbacks to filter notes on arrival.
