// All `#[js_export]` methods must accept arguments by value (not by reference) because
// wasm_bindgen and napi-rs require owned parameters for JS interop. Suppress the lint
// crate-wide rather than annotating every individual function.
#![allow(clippy::needless_pass_by_value)]

extern crate alloc;

#[cfg(all(feature = "browser", feature = "nodejs"))]
compile_error!("Features `browser` and `nodejs` are mutually exclusive. Enable only one.");

#[cfg(not(any(feature = "browser", feature = "nodejs")))]
compile_error!("Either `browser` or `nodejs` feature must be enabled.");

use alloc::sync::Arc;
use core::error::Error;
use core::fmt::Write;

#[cfg(feature = "browser")]
use idxdb_store::IdxdbStore;
use js_export_macro::js_export;
#[cfg(feature = "browser")]
use js_sys::{Function, Reflect};
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
#[cfg(feature = "nodejs")]
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note_transport::NoteTransportClient;
use miden_client::note_transport::grpc::GrpcNoteTransportClient;
use miden_client::rpc::{Endpoint, GrpcClient, NodeRpcClient};
use miden_client::store::Store;
use miden_client::testing::mock::MockRpcApi;
use miden_client::testing::note_transport::MockNoteTransportApi;
use miden_client::{Client, ClientError, DebugMode, ErrorHint, Felt};
use models::code_builder::CodeBuilder;
#[cfg(feature = "nodejs")]
use napi_derive::napi;
#[cfg(feature = "nodejs")]
use platform::maybe_wrap_send;
use platform::{AsyncCell, ClientAuth, JsErr, from_str_err};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
#[cfg(feature = "browser")]
use tracing::Level;
#[cfg(feature = "browser")]
use tracing_subscriber::layer::SubscriberExt;
#[cfg(feature = "browser")]
use wasm_bindgen::prelude::*;

pub mod account;
pub mod export;
pub mod helpers;
pub mod import;
#[macro_use]
pub(crate) mod miden_array;
pub mod mock;
pub mod models;
pub mod new_account;
pub mod new_transactions;
pub mod note_transport;
pub mod notes;
pub(crate) mod platform;
pub mod rpc_client;
pub mod settings;
pub mod sync;
pub mod tags;
pub mod transactions;
pub mod utils;

#[cfg(feature = "browser")]
pub mod keystore_api;
#[cfg(feature = "browser")]
mod web_keystore;
#[cfg(feature = "browser")]
mod web_keystore_callbacks;
#[cfg(feature = "browser")]
mod web_keystore_db;
#[cfg(feature = "browser")]
pub use web_keystore::WebKeyStore;

#[cfg(feature = "browser")]
const BASE_STORE_NAME: &str = "MidenClientDB";

/// Initializes the `tracing` subscriber that routes Rust log output to the
/// browser console via `console.log` / `console.warn` / `console.error`.
///
/// `log_level` must be one of `"error"`, `"warn"`, `"info"`, `"debug"`,
/// `"trace"`, `"off"`, or `"none"` (no logging). Unknown values are treated
/// as "off".
///
/// This is a **per-thread global** — call it once on the main thread and, if
/// you use a Web Worker, once inside the worker. Subsequent calls on the same
/// thread are harmless no-ops.
#[cfg(feature = "browser")]
#[wasm_bindgen(js_name = "setupLogging")]
pub fn setup_logging(log_level: &str) {
    let level = match log_level.to_lowercase().as_str() {
        "error" => Some(Level::ERROR),
        "warn" => Some(Level::WARN),
        "info" => Some(Level::INFO),
        "debug" => Some(Level::DEBUG),
        "trace" => Some(Level::TRACE),
        _ => None,
    };

    if let Some(level) = level {
        let config = tracing_wasm::WASMLayerConfigBuilder::new().set_max_level(level).build();
        // `set_as_global_default_with_config` panics on double-init, so replicate
        // its logic with `set_global_default` which returns a `Result` instead.
        let _ = tracing::subscriber::set_global_default(
            tracing_subscriber::registry().with(tracing_wasm::WASMLayer::new(config)),
        );
    }
}

#[js_export]
pub struct WebClient {
    inner: AsyncCell<Option<Client<ClientAuth>>>,
    mock_rpc_api: AsyncCell<Option<Arc<MockRpcApi>>>,
    mock_note_transport_api: AsyncCell<Option<Arc<MockNoteTransportApi>>>,
}

// SAFETY: napi-rs with `tokio_rt` uses a multi-threaded tokio runtime, so async napi
// functions run on worker threads. This is sound because the concrete types behind
// trait objects (`SqliteStore`, `GrpcClient`, `FilesystemKeyStore`) are all Send + Sync
// — only the `dyn Trait` bounds lack Send. All mutable state is behind `AsyncCell`
// (tokio::sync::Mutex), which serializes access.
#[cfg(feature = "nodejs")]
unsafe impl Send for WebClient {}
#[cfg(feature = "nodejs")]
unsafe impl Sync for WebClient {}

// Prevent deadpool's Drop from panicking on Node.js process exit.
// When napi's Tokio runtime shuts down, the SQLite connection pool tries to
// spawn_blocking during Drop, but the runtime is already gone. This causes a
// panic-in-panic (SIGABRT). Since the OS reclaims all resources on exit anyway,
// we intentionally leak the client to avoid the crash.
#[cfg(feature = "nodejs")]
impl Drop for WebClient {
    fn drop(&mut self) {
        // Take ownership of the inner Mutex contents and forget them,
        // preventing the deadpool Drop chain from running.
        let inner = std::mem::replace(&mut self.inner, AsyncCell::new(None));
        std::mem::forget(inner);
    }
}

impl Default for WebClient {
    fn default() -> Self {
        Self::new()
    }
}

// Common methods shared between browser and Node.js
#[js_export]
impl WebClient {
    #[js_export(constructor)]
    pub fn new() -> Self {
        #[cfg(feature = "browser")]
        console_error_panic_hook::set_once();

        WebClient {
            inner: AsyncCell::new(None),
            mock_rpc_api: AsyncCell::new(None),
            mock_note_transport_api: AsyncCell::new(None),
        }
    }

    /// Returns the identifier of the underlying store (e.g. `IndexedDB` database name, file path).
    #[js_export(js_name = "storeIdentifier")]
    pub async fn store_identifier(&self) -> Result<String, JsErr> {
        let guard = self.inner.lock().await;
        let client = guard.as_ref().ok_or_else(|| from_str_err("Client not initialized"))?;
        Ok(client.store_identifier().to_string())
    }

    #[js_export(js_name = "createCodeBuilder")]
    pub async fn create_code_builder(&self) -> Result<CodeBuilder, JsErr> {
        let guard = self.inner.lock().await;
        let client = guard.as_ref().ok_or_else(|| {
            from_str_err("client was not initialized before instancing CodeBuilder")
        })?;
        Ok(CodeBuilder::from_source_manager(client.code_builder().source_manager().clone()))
    }
}

// Internal helpers
impl WebClient {
    pub(crate) async fn get_mut_inner(
        &self,
    ) -> impl core::ops::DerefMut<Target = Option<Client<ClientAuth>>> + '_ {
        self.inner.lock().await
    }

    pub(crate) async fn get_keystore(&self) -> Result<Arc<ClientAuth>, JsErr> {
        let guard = self.inner.lock().await;
        guard
            .as_ref()
            .and_then(|c| c.authenticator())
            .cloned()
            .ok_or_else(|| from_str_err("Client not initialized"))
    }
}

// Browser-specific client creation
#[cfg(feature = "browser")]
#[wasm_bindgen]
impl WebClient {
    /// Returns a `WebKeystoreApi` handle for managing secret keys.
    ///
    /// The returned object can be used from JavaScript as `client.keystore`.
    #[wasm_bindgen(getter)]
    pub fn keystore(&self) -> Result<keystore_api::WebKeystoreApi, JsValue> {
        let guard = self.inner.borrow();
        let ks = guard
            .as_ref()
            .and_then(|c| c.authenticator())
            .cloned()
            .ok_or_else(|| JsValue::from_str("Client not initialized"))?;
        Ok(keystore_api::WebKeystoreApi::new(ks))
    }

    /// Creates a new `WebClient` instance with the specified configuration.
    ///
    /// # Arguments
    /// * `node_url`: The URL of the node RPC endpoint. If `None`, defaults to the testnet endpoint.
    /// * `node_note_transport_url`: Optional URL of the note transport service.
    /// * `seed`: Optional seed for account initialization.
    /// * `store_name`: Optional name for the web store. If `None`, the store name defaults to
    ///   `MidenClientDB_{network_id}`, where `network_id` is derived from the `node_url`.
    ///   Explicitly setting this allows for creating multiple isolated clients.
    /// * `debug_mode`: Optional flag to enable debug mode for transaction execution. When enabled,
    ///   the transaction executor records additional information useful for debugging. Defaults to
    ///   disabled.
    #[wasm_bindgen(js_name = "createClient")]
    pub async fn create_client(
        &self,
        node_url: Option<String>,
        node_note_transport_url: Option<String>,
        seed: Option<Vec<u8>>,
        store_name: Option<String>,
        debug_mode: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        let endpoint = node_url.map_or(Ok(Endpoint::testnet()), |url| {
            Endpoint::try_from(url.as_str()).map_err(|_| JsValue::from_str("Invalid node URL"))
        })?;

        let web_rpc_client = Arc::new(GrpcClient::new(&endpoint, 0));

        let note_transport_client = node_note_transport_url
            .map(|url| Arc::new(GrpcNoteTransportClient::new(url)) as Arc<dyn NoteTransportClient>);

        let store_name =
            store_name.unwrap_or(format!("{}_{}", BASE_STORE_NAME, endpoint.to_network_id()));

        let rng = create_rng(seed)?;
        let store: Arc<dyn Store> = Arc::new(
            IdxdbStore::new(store_name.clone())
                .await
                .map_err(|_| JsValue::from_str("Failed to initialize IdxdbStore"))?,
        );
        let keystore = WebKeyStore::new_with_callbacks(rng, store_name.clone(), None, None, None);

        self.setup_client(web_rpc_client, store, keystore, rng, note_transport_client, debug_mode)
            .await?;

        Ok(JsValue::from_str("Client created successfully"))
    }

    /// Creates a new `WebClient` instance with external keystore callbacks.
    ///
    /// # Arguments
    /// * `node_url`: The URL of the node RPC endpoint. If `None`, defaults to the testnet endpoint.
    /// * `node_note_transport_url`: Optional URL of the note transport service.
    /// * `seed`: Optional seed for account initialization.
    /// * `store_name`: Optional name for the web store. If `None`, the store name defaults to
    ///   `MidenClientDB_{network_id}`, where `network_id` is derived from the `node_url`.
    ///   Explicitly setting this allows for creating multiple isolated clients.
    /// * `get_key_cb`: Callback to retrieve the secret key bytes for a given public key.
    /// * `insert_key_cb`: Callback to persist a secret key.
    /// * `sign_cb`: Callback to produce serialized signature bytes for the provided inputs.
    /// * `debug_mode`: Optional flag to enable debug mode for transaction execution. Defaults to
    ///   disabled.
    #[wasm_bindgen(js_name = "createClientWithExternalKeystore")]
    #[allow(clippy::too_many_arguments)]
    pub async fn create_client_with_external_keystore(
        &self,
        node_url: Option<String>,
        node_note_transport_url: Option<String>,
        seed: Option<Vec<u8>>,
        store_name: Option<String>,
        get_key_cb: Option<Function>,
        insert_key_cb: Option<Function>,
        sign_cb: Option<Function>,
        debug_mode: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        let endpoint = node_url.map_or(Ok(Endpoint::testnet()), |url| {
            Endpoint::try_from(url.as_str()).map_err(|_| JsValue::from_str("Invalid node URL"))
        })?;

        let web_rpc_client = Arc::new(GrpcClient::new(&endpoint, 0));

        let note_transport_client = node_note_transport_url
            .map(|url| Arc::new(GrpcNoteTransportClient::new(url)) as Arc<dyn NoteTransportClient>);

        let store_name =
            store_name.unwrap_or(format!("{}_{}", BASE_STORE_NAME, endpoint.to_network_id()));

        let rng = create_rng(seed)?;
        let store: Arc<dyn Store> = Arc::new(
            IdxdbStore::new(store_name.clone())
                .await
                .map_err(|_| JsValue::from_str("Failed to initialize IdxdbStore"))?,
        );
        let keystore =
            WebKeyStore::new_with_callbacks(rng, store_name, get_key_cb, insert_key_cb, sign_cb);

        self.setup_client(web_rpc_client, store, keystore, rng, note_transport_client, debug_mode)
            .await?;

        Ok(JsValue::from_str("Client created successfully"))
    }

    async fn setup_client(
        &self,
        rpc_client: Arc<dyn NodeRpcClient>,
        store: Arc<dyn Store>,
        keystore: WebKeyStore<RpoRandomCoin>,
        rng: RpoRandomCoin,
        note_transport_client: Option<Arc<dyn NoteTransportClient>>,
        debug_mode: Option<bool>,
    ) -> Result<(), JsValue> {
        let mut builder = ClientBuilder::new()
            .rpc(rpc_client)
            .rng(Box::new(rng))
            .store(store)
            .authenticator(Arc::new(keystore))
            .in_debug_mode(if debug_mode.unwrap_or(false) {
                DebugMode::Enabled
            } else {
                DebugMode::Disabled
            });

        if let Some(transport) = note_transport_client {
            builder = builder.note_transport(transport);
        }

        let mut client = builder
            .build()
            .await
            .map_err(|err| js_error_with_context(err, "Failed to create client"))?;

        client
            .ensure_genesis_in_place()
            .await
            .map_err(|err| js_error_with_context(err, "Failed to ensure genesis in place"))?;

        *self.inner.lock().await = Some(client);

        Ok(())
    }
}

// Node.js-specific client creation
#[cfg(feature = "nodejs")]
#[napi]
impl WebClient {
    /// Creates a new `WebClient` instance with the specified configuration.
    ///
    /// # Arguments
    /// * `node_url`: The URL of the node RPC endpoint. If `None`, defaults to the testnet endpoint.
    /// * `node_note_transport_url`: Optional URL of the note transport service.
    /// * `seed`: Optional seed for account initialization.
    /// * `db_path`: Path to the SQLite database file.
    /// * `keystore_path`: Path to the directory for storing keys.
    /// * `debug_mode`: Optional flag to enable debug mode for transaction execution. Defaults to
    ///   disabled.
    #[napi(js_name = "createClient")]
    pub async fn create_client(
        &self,
        node_url: Option<String>,
        node_note_transport_url: Option<String>,
        seed: Option<Vec<u8>>,
        db_path: String,
        keystore_path: String,
        debug_mode: Option<bool>,
    ) -> Result<String, JsErr> {
        let endpoint = node_url.map_or(Ok(Endpoint::testnet()), |url| {
            Endpoint::try_from(url.as_str()).map_err(|_| from_str_err("Invalid node URL"))
        })?;

        let rpc_client = Arc::new(GrpcClient::new(&endpoint, 10_000));

        let note_transport_client = if let Some(url) = node_note_transport_url {
            let client = GrpcNoteTransportClient::connect(url, 10_000)
                .await
                .map_err(|e| from_str_err(&format!("Failed to connect note transport: {e}")))?;
            Some(Arc::new(client) as Arc<dyn NoteTransportClient>)
        } else {
            None
        };

        let rng = create_rng(seed)?;

        let store: Arc<dyn Store> = Arc::new(
            miden_client_sqlite_store::SqliteStore::new(db_path.into())
                .await
                .map_err(|e| from_str_err(&format!("Failed to initialize SqliteStore: {e}")))?,
        );

        let keystore = FilesystemKeyStore::new(keystore_path.into())
            .map_err(|e| from_str_err(&format!("Failed to initialize keystore: {e}")))?;

        self.setup_client(rpc_client, store, keystore, rng, note_transport_client, debug_mode)
            .await?;

        Ok("Client created successfully".to_string())
    }

    async fn setup_client(
        &self,
        rpc_client: Arc<dyn NodeRpcClient>,
        store: Arc<dyn Store>,
        keystore: FilesystemKeyStore,
        rng: RpoRandomCoin,
        note_transport_client: Option<Arc<dyn NoteTransportClient>>,
        debug_mode: Option<bool>,
    ) -> Result<(), JsErr> {
        let client = maybe_wrap_send(async move {
            let mut builder = ClientBuilder::new()
                .rpc(rpc_client)
                .rng(Box::new(rng))
                .store(store)
                .authenticator(Arc::new(keystore))
                .in_debug_mode(if debug_mode.unwrap_or(false) {
                    DebugMode::Enabled
                } else {
                    DebugMode::Disabled
                });

            if let Some(transport) = note_transport_client {
                builder = builder.note_transport(transport);
            }

            let mut client = builder
                .build()
                .await
                .map_err(|err| js_error_with_context(err, "Failed to create client"))?;

            client
                .ensure_genesis_in_place()
                .await
                .map_err(|err| js_error_with_context(err, "Failed to ensure genesis in place"))?;

            Ok::<_, JsErr>(client)
        })
        .await?;

        *self.inner.lock().await = Some(client);

        Ok(())
    }
}

pub(crate) fn create_rng(seed: Option<Vec<u8>>) -> Result<RpoRandomCoin, JsErr> {
    let mut rng = match seed {
        Some(seed_bytes) => {
            if seed_bytes.len() == 32 {
                let mut seed_array = [0u8; 32];
                seed_array.copy_from_slice(&seed_bytes);
                StdRng::from_seed(seed_array)
            } else {
                return Err(from_str_err("Seed must be exactly 32 bytes"));
            }
        },
        None => StdRng::from_os_rng(),
    };
    let coin_seed: [u64; 4] = rng.random();
    Ok(RpoRandomCoin::new(coin_seed.map(Felt::new).into()))
}

// ERROR HANDLING HELPERS
// ================================================================================================

pub(crate) fn js_error_with_context<T>(err: T, context: &str) -> JsErr
where
    T: Error + 'static,
{
    let error_message = build_error_chain(context, &err);
    let help = hint_from_error(&err);

    #[cfg(feature = "browser")]
    {
        let js_error: JsValue = JsError::new(&error_message).into();
        if let Some(help) = help {
            let _ = Reflect::set(&js_error, &JsValue::from_str("help"), &JsValue::from_str(&help));
        }
        js_error
    }

    #[cfg(feature = "nodejs")]
    {
        let message = match help {
            Some(help) => format!("{error_message} [help: {help}]"),
            None => error_message,
        };
        napi::Error::from_reason(message)
    }
}

/// Walks the error chain and builds a `context: err1: err2: ...` message.
fn build_error_chain(context: &str, err: &(dyn Error + 'static)) -> String {
    let mut msg = context.to_string();
    let mut source = Some(err);
    while let Some(e) = source {
        write!(msg, ": {e}").expect("writing to string should always succeed");
        source = e.source();
    }
    msg
}

fn hint_from_error(err: &(dyn Error + 'static)) -> Option<String> {
    if let Some(client_error) = err.downcast_ref::<ClientError>() {
        return Option::<ErrorHint>::from(client_error).map(ErrorHint::into_help_message);
    }

    err.source().and_then(hint_from_error)
}
