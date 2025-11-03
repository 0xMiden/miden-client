# miden-client

`miden-client` provides a std-enabled orchestration layer around the `miden-client-core`
functionality. It exposes the background service, handler registration, and other utilities for
running the client in long-lived server applications.

The crate re-exports everything from `miden-client-core`, so existing consumers can migrate
incrementally while gaining access to the new service abstractions.

## Quick start

```rust
use std::sync::Arc;
use std::time::Duration;

use miden_client::builder::ClientBuilder;
use miden_client::service::{ClientRuntime, ClientServiceConfig, ClientServiceError, SyncEvent};
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::NoteTag;
use miden_client_sqlite_store::ClientBuilderSqliteExt;

# #[tokio::main]
async fn main() -> Result<(), ClientServiceError> {
    // Build the core client as usual.
    let client = ClientBuilder::<FilesystemKeyStore<_>>::new()
        .sqlite_store("./client.sqlite3")
        .filesystem_keystore("./keys")
        .build()
        .await?;

    // Start the background runtime with a 5 second polling interval.
    let runtime = ClientRuntime::start(client, ClientServiceConfig::default()).await?;
    let handle = runtime.handle();

    // Register a blocking handler that logs new notes.
    handle
        .register_blocking_handler(|_handle, event: SyncEvent| async move {
            if !event.summary.received_notes.is_empty() {
                tracing::info!(?event.summary.received_notes, "new notes received");
            }
            Ok(())
        })
        .await;

    // Trigger an immediate sync in addition to the background interval.
    handle.trigger_sync();

    // Update tracked note tags without blocking reads from other tasks.
    handle
        .add_note_tag(NoteTag::from_word(miden_client::Word::default()))
        .await?;

    // Application code can now execute transactions while the background service keeps the
    // client in sync.

    // When shutting down, drop the runtime or call `runtime.shutdown().await` explicitly.
    runtime.shutdown().await;

    Ok(())
}
```

See the [`service` module documentation](https://docs.rs/miden-client) for more details on the
available APIs, including transaction queues, handler management, and manual sync controls.
