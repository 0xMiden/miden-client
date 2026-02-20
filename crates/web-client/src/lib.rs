#![cfg_attr(feature = "wasm", no_std)]

extern crate alloc;

#[cfg(all(feature = "wasm", feature = "napi"))]
compile_error!("Features `wasm` and `napi` are mutually exclusive.");

#[cfg(not(any(feature = "wasm", feature = "napi")))]
compile_error!("One of `wasm` or `napi` features must be enabled.");

// Shared imports
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt::Write;

#[cfg(feature = "wasm")]
use core::cell::RefCell;

#[cfg(feature = "napi")]
use std::path::PathBuf;

use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::note_transport::NoteTransportClient;
use miden_client::note_transport::grpc::GrpcNoteTransportClient;
use miden_client::rpc::{Endpoint, GrpcClient, NodeRpcClient};
use miden_client::{Client, DebugMode, Felt};
use models::code_builder::CodeBuilder;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[cfg(feature = "wasm")]
use idxdb_store::WebStore;
#[cfg(feature = "wasm")]
use js_sys::{Function, Reflect};
#[cfg(feature = "wasm")]
use miden_client::testing::mock::MockRpcApi;
#[cfg(feature = "wasm")]
use miden_client::testing::note_transport::MockNoteTransportApi;
#[cfg(feature = "wasm")]
use miden_client::{ClientError, ErrorHint};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "napi")]
use miden_client::keystore::FilesystemKeyStore;
#[cfg(feature = "napi")]
use miden_client::store::Store;
#[cfg(feature = "napi")]
use miden_client_sqlite_store::SqliteStore;
#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "napi")]
use tokio::sync::Mutex;

// ================================================================================================
// INTERNAL MACROS
// ================================================================================================

/// Acquires a lock on the inner client. Returns a guard (RefMut for wasm, MutexGuard for napi).
/// Use `.as_mut()` on the returned guard to get `Option<&mut Client<Authenticator>>`.
macro_rules! lock_client {
    ($self:expr) => {{
        #[cfg(feature = "wasm")]
        let guard = $self.inner.borrow_mut();
        #[cfg(feature = "napi")]
        let guard = $self.inner.lock().await;
        guard
    }};
}

/// Clones the store from the client.
macro_rules! lock_store {
    ($self:expr) => {{
        #[cfg(feature = "wasm")]
        let store = $self.store.clone();
        #[cfg(feature = "napi")]
        let store = $self.store.lock().await.clone();
        store.ok_or_else(|| crate::platform::error_from_string("Store not initialized"))
    }};
}

/// Clones the keystore from the client.
macro_rules! lock_keystore {
    ($self:expr) => {{
        #[cfg(feature = "wasm")]
        let ks = $self.keystore.clone();
        #[cfg(feature = "napi")]
        let ks = $self.keystore.lock().await.clone();
        ks.ok_or_else(|| crate::platform::error_from_string("Keystore not initialized"))
    }};
}

// WASM-only macro module (must be before models so the macro is in scope)
#[cfg(feature = "wasm")]
#[macro_use]
pub(crate) mod miden_array;

// Shared modules
pub mod account;
pub mod conversions;
pub mod export;
pub mod helpers;
pub mod import;
pub mod models;
pub mod new_account;
pub mod new_transactions;
pub mod note_transport;
pub mod notes;
pub(crate) mod platform;
pub mod prelude;
pub mod rpc_client;
pub mod settings;
pub mod sync;
pub mod tags;
pub mod transactions;
#[cfg(feature = "wasm")]
pub mod mock;
#[cfg(feature = "wasm")]
pub mod utils;
#[cfg(feature = "wasm")]
mod web_keystore;
#[cfg(feature = "wasm")]
mod web_keystore_callbacks;
#[cfg(feature = "wasm")]
pub use web_keystore::WebKeyStore;

// ================================================================================================
// TYPE ALIASES
// ================================================================================================

#[cfg(feature = "wasm")]
pub(crate) type Authenticator = WebKeyStore<RpoRandomCoin>;
#[cfg(feature = "napi")]
pub(crate) type Authenticator = FilesystemKeyStore;

// ================================================================================================
// WebClient
// ================================================================================================

#[cfg(feature = "wasm")]
const BASE_STORE_NAME: &str = "WebClientDB";
#[cfg(feature = "napi")]
const DEFAULT_DB_PATH: &str = "miden_client.db";
#[cfg(feature = "napi")]
const DEFAULT_KEYS_DIR: &str = "miden_keys";

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "napi", napi)]
pub struct WebClient {
    #[cfg(feature = "wasm")]
    inner: RefCell<Option<Client<Authenticator>>>,
    #[cfg(feature = "napi")]
    inner: Mutex<Option<Client<Authenticator>>>,

    #[cfg(feature = "wasm")]
    pub(crate) store: Option<Arc<WebStore>>,
    #[cfg(feature = "napi")]
    store: Mutex<Option<Arc<dyn Store>>>,

    #[cfg(feature = "wasm")]
    pub(crate) keystore: Option<WebKeyStore<RpoRandomCoin>>,
    #[cfg(feature = "napi")]
    keystore: Mutex<Option<Arc<FilesystemKeyStore>>>,

    #[cfg(feature = "wasm")]
    pub(crate) mock_rpc_api: Option<Arc<MockRpcApi>>,
    #[cfg(feature = "wasm")]
    pub(crate) mock_note_transport_api: Option<Arc<MockNoteTransportApi>>,

    debug_mode: bool,
}

impl Default for WebClient {
    fn default() -> Self {
        Self::new()
    }
}

// Shared constructor and simple methods
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "napi", napi)]
impl WebClient {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    #[cfg_attr(feature = "napi", napi(constructor))]
    pub fn new() -> Self {
        #[cfg(feature = "wasm")]
        console_error_panic_hook::set_once();

        WebClient {
            #[cfg(feature = "wasm")]
            inner: RefCell::new(None),
            #[cfg(feature = "napi")]
            inner: Mutex::new(None),

            #[cfg(feature = "wasm")]
            store: None,
            #[cfg(feature = "napi")]
            store: Mutex::new(None),

            #[cfg(feature = "wasm")]
            keystore: None,
            #[cfg(feature = "napi")]
            keystore: Mutex::new(None),

            #[cfg(feature = "wasm")]
            mock_rpc_api: None,
            #[cfg(feature = "wasm")]
            mock_note_transport_api: None,

            debug_mode: false,
        }
    }

    /// Sets the debug mode for transaction execution.
    ///
    /// When enabled, the transaction executor will record additional information useful for
    /// debugging (the values on the VM stack and the state of the advice provider). This is
    /// disabled by default since it adds overhead.
    ///
    /// Must be called before `createClient`.
    #[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "setDebugMode"))]
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }
}

// ================================================================================================
// WebClient methods (wasm)
// ================================================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WebClient {
    /// Creates a new client instance with the specified configuration.
    ///
    /// # Arguments
    /// * `node_url`: The URL of the node RPC endpoint. If `None`, defaults to the testnet endpoint.
    /// * `node_note_transport_url`: Optional URL of the note transport service.
    /// * `seed`: Optional seed for account initialization.
    /// * `store_name`: Optional name for the web store. If `None`, the store name defaults to
    ///   `WebClientDB_{network_id}`, where `network_id` is derived from the `node_url`.
    ///   Explicitly setting this allows for creating multiple isolated clients.
    #[wasm_bindgen(js_name = "createClient")]
    pub async fn create_client(
        &mut self,
        node_url: Option<String>,
        node_note_transport_url: Option<String>,
        seed: Option<Vec<u8>>,
        store_name: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let endpoint = node_url.map_or(Ok(Endpoint::testnet()), |url| {
            Endpoint::try_from(url.as_str())
                .map_err(|_| platform::error_from_string("Invalid node URL"))
        })?;

        let web_rpc_client = Arc::new(GrpcClient::new(&endpoint, 0));

        let note_transport_client = node_note_transport_url
            .map(|url| Arc::new(GrpcNoteTransportClient::new(url)) as Arc<dyn NoteTransportClient>);

        let store_name =
            store_name.unwrap_or(format!("{}_{}", BASE_STORE_NAME, endpoint.to_network_id()));

        self.setup_client(
            web_rpc_client,
            store_name,
            note_transport_client,
            seed,
            None,
            None,
            None,
        )
        .await?;

        Ok(JsValue::from_str("Client created successfully"))
    }

    /// Creates a new client instance with external keystore callbacks.
    ///
    /// # Arguments
    /// * `node_url`: The URL of the node RPC endpoint. If `None`, defaults to the testnet endpoint.
    /// * `node_note_transport_url`: Optional URL of the note transport service.
    /// * `seed`: Optional seed for account initialization.
    /// * `store_name`: Optional name for the web store. If `None`, the store name defaults to
    ///   `WebClientDB_{network_id}`, where `network_id` is derived from the `node_url`.
    ///   Explicitly setting this allows for creating multiple isolated clients.
    /// * `get_key_cb`: Callback to retrieve the secret key bytes for a given public key.
    /// * `insert_key_cb`: Callback to persist a secret key.
    /// * `sign_cb`: Callback to produce serialized signature bytes for the provided inputs.
    #[wasm_bindgen(js_name = "createClientWithExternalKeystore")]
    pub async fn create_client_with_external_keystore(
        &mut self,
        node_url: Option<String>,
        node_note_transport_url: Option<String>,
        seed: Option<Vec<u8>>,
        store_name: Option<String>,
        get_key_cb: Option<Function>,
        insert_key_cb: Option<Function>,
        sign_cb: Option<Function>,
    ) -> Result<JsValue, JsValue> {
        let endpoint = node_url.map_or(Ok(Endpoint::testnet()), |url| {
            Endpoint::try_from(url.as_str())
                .map_err(|_| platform::error_from_string("Invalid node URL"))
        })?;

        let web_rpc_client = Arc::new(GrpcClient::new(&endpoint, 0));

        let note_transport_client = node_note_transport_url
            .map(|url| Arc::new(GrpcNoteTransportClient::new(url)) as Arc<dyn NoteTransportClient>);

        let store_name =
            store_name.unwrap_or(format!("{}_{}", BASE_STORE_NAME, endpoint.to_network_id()));

        self.setup_client(
            web_rpc_client,
            store_name,
            note_transport_client,
            seed,
            get_key_cb,
            insert_key_cb,
            sign_cb,
        )
        .await?;

        Ok(JsValue::from_str("Client created successfully"))
    }

    async fn setup_client(
        &mut self,
        rpc_client: Arc<dyn NodeRpcClient>,
        store_name: String,
        note_transport_client: Option<Arc<dyn NoteTransportClient>>,
        seed: Option<Vec<u8>>,
        get_key_cb: Option<Function>,
        insert_key_cb: Option<Function>,
        sign_cb: Option<Function>,
    ) -> Result<(), JsValue> {
        let mut rng = match seed {
            Some(seed_bytes) => {
                if seed_bytes.len() == 32 {
                    let mut seed_array = [0u8; 32];
                    seed_array.copy_from_slice(&seed_bytes);
                    StdRng::from_seed(seed_array)
                } else {
                    return Err(platform::error_from_string("Seed must be exactly 32 bytes"));
                }
            },
            None => StdRng::from_os_rng(),
        };
        let coin_seed: [u64; 4] = rng.random();

        let rng = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

        let web_store = Arc::new(
            WebStore::new(store_name.clone())
                .await
                .map_err(|_| platform::error_from_string("Failed to initialize WebStore"))?,
        );

        let keystore =
            WebKeyStore::new_with_callbacks(rng, store_name, get_key_cb, insert_key_cb, sign_cb);

        let mut builder = ClientBuilder::new()
            .rpc(rpc_client)
            .rng(Box::new(rng))
            .store(web_store.clone())
            .authenticator(Arc::new(keystore.clone()))
            .in_debug_mode(if self.debug_mode {
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
            .map_err(|err| platform::error_with_context(err, "Failed to create client"))?;

        // Ensure genesis block is fetched and stored in IndexedDB.
        // This is important for web workers that create their own client instances -
        // they will read the genesis from the shared IndexedDB and automatically
        // set the genesis commitment on their RPC client.
        client
            .ensure_genesis_in_place()
            .await
            .map_err(|err| platform::error_with_context(err, "Failed to ensure genesis in place"))?;

        *self.inner.get_mut() = Some(client);
        self.store = Some(web_store);
        self.keystore = Some(keystore);

        Ok(())
    }

    #[wasm_bindgen(js_name = "createCodeBuilder")]
    pub fn create_code_builder(&self) -> Result<CodeBuilder, JsValue> {
        let guard = self.inner.borrow();
        let Some(client) = guard.as_ref() else {
            return Err("client was not initialized before instancing CodeBuilder".into());
        };
        Ok(CodeBuilder::from_source_manager(client.code_builder().source_manager().clone()))
    }
}

// ================================================================================================
// WebClient methods (napi)
// ================================================================================================

#[cfg(feature = "napi")]
#[napi]
impl WebClient {
    /// Creates a new client instance with the specified configuration.
    ///
    /// # Arguments
    /// * `node_url`: The URL of the node RPC endpoint. If not provided, defaults to the testnet.
    /// * `note_transport_url`: Optional URL of the note transport service.
    /// * `seed`: Optional 32-byte seed for RNG initialization.
    /// * `db_path`: Optional path for the SQLite database file. Defaults to "miden_client.db".
    /// * `keystore_path`: Optional directory path for the filesystem keystore. Defaults to
    ///   "miden_keys".
    pub async fn create_client(
        &self,
        node_url: Option<String>,
        note_transport_url: Option<String>,
        seed: Option<Buffer>,
        db_path: Option<String>,
        keystore_path: Option<String>,
    ) -> napi::Result<String> {
        let endpoint = node_url.map_or(Ok(Endpoint::testnet()), |url| {
            Endpoint::try_from(url.as_str())
                .map_err(|e| platform::error_from_string(&format!("Invalid node URL: {e}")))
        })?;

        let rpc_client = Arc::new(GrpcClient::new(&endpoint, 10_000));

        let note_transport_client = match note_transport_url {
            Some(url) => {
                let client = GrpcNoteTransportClient::connect(url, 10_000)
                    .await
                    .map_err(|e| {
                        platform::error_with_context(e, "Failed to connect note transport")
                    })?;
                Some(Arc::new(client) as Arc<dyn NoteTransportClient>)
            },
            None => None,
        };

        let db_path = PathBuf::from(db_path.unwrap_or_else(|| DEFAULT_DB_PATH.to_string()));

        let keystore_dir =
            PathBuf::from(keystore_path.unwrap_or_else(|| DEFAULT_KEYS_DIR.to_string()));
        let keystore = FilesystemKeyStore::new(keystore_dir)
            .map_err(|e| platform::error_with_context(e, "Failed to create keystore"))?;

        self.setup_client(rpc_client, db_path, note_transport_client, seed, keystore)
            .await?;

        Ok("Client created successfully".to_string())
    }

    async fn setup_client(
        &self,
        rpc_client: Arc<dyn NodeRpcClient>,
        db_path: PathBuf,
        note_transport_client: Option<Arc<dyn NoteTransportClient>>,
        seed: Option<Buffer>,
        keystore: FilesystemKeyStore,
    ) -> napi::Result<()> {
        let mut rng = match seed {
            Some(seed_bytes) => {
                if seed_bytes.len() == 32 {
                    let mut seed_array = [0u8; 32];
                    seed_array.copy_from_slice(&seed_bytes);
                    StdRng::from_seed(seed_array)
                } else {
                    return Err(platform::error_from_string("Seed must be exactly 32 bytes"));
                }
            },
            None => StdRng::from_os_rng(),
        };
        let coin_seed: [u64; 4] = rng.random();
        let rng = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

        let sqlite_store = Arc::new(
            SqliteStore::new(db_path)
                .await
                .map_err(|err| {
                    platform::error_with_context(err, "Failed to create SQLite store")
                })?,
        );

        let keystore = Arc::new(keystore);

        let debug_mode = self.debug_mode;
        let mut client = build_client(
            rpc_client,
            sqlite_store.clone(),
            rng,
            keystore.clone(),
            note_transport_client,
            debug_mode,
        )
        .await
        .map_err(|err| platform::error_with_context(err, "Failed to create client"))?;

        client
            .ensure_genesis_in_place()
            .await
            .map_err(|err| {
                platform::error_with_context(err, "Failed to ensure genesis in place")
            })?;

        *self.keystore.lock().await = Some(keystore);
        *self.store.lock().await = Some(sqlite_store as Arc<dyn Store>);
        *self.inner.lock().await = Some(client);

        Ok(())
    }

    pub async fn create_code_builder(&self) -> napi::Result<CodeBuilder> {
        let guard = self.inner.lock().await;
        let client = guard.as_ref().ok_or_else(|| {
            platform::error_from_string("client was not initialized before instancing CodeBuilder")
        })?;
        Ok(CodeBuilder::from_source_manager(client.code_builder().source_manager().clone()))
    }
}

// ================================================================================================
// ERROR HANDLING HELPERS
// ================================================================================================

#[cfg(feature = "wasm")]
pub(crate) fn js_error_with_context<T>(err: T, context: &str) -> JsValue
where
    T: Error + 'static,
{
    let mut error_string = context.to_string();
    let mut source = Some(&err as &dyn Error);
    while let Some(err) = source {
        write!(error_string, ": {err}").expect("writing to string should always succeed");
        source = err.source();
    }

    let help = hint_from_error(&err);
    let js_error: JsValue = JsError::new(&error_string).into();

    if let Some(help) = help {
        let _ = Reflect::set(&js_error, &JsValue::from_str("help"), &JsValue::from_str(&help));
    }

    js_error
}

#[cfg(feature = "wasm")]
fn hint_from_error(err: &(dyn Error + 'static)) -> Option<String> {
    if let Some(client_error) = err.downcast_ref::<ClientError>() {
        return Option::<ErrorHint>::from(client_error).map(ErrorHint::into_help_message);
    }

    err.source().and_then(hint_from_error)
}

#[cfg(feature = "napi")]
pub(crate) fn napi_error_with_context<T: Error>(err: T, context: &str) -> napi::Error {
    let mut error_string = context.to_string();
    let mut source = Some(&err as &dyn Error);
    while let Some(e) = source {
        write!(error_string, ": {e}").expect("writing to string should always succeed");
        source = e.source();
    }
    napi::Error::from_reason(error_string)
}

// ================================================================================================
// CLIENT BUILDER HELPER (napi)
// ================================================================================================

#[cfg(feature = "napi")]
async fn build_client(
    rpc_client: Arc<dyn NodeRpcClient>,
    store: Arc<SqliteStore>,
    rng: RpoRandomCoin,
    keystore: Arc<FilesystemKeyStore>,
    note_transport_client: Option<Arc<dyn NoteTransportClient>>,
    debug_mode: bool,
) -> std::result::Result<Client<FilesystemKeyStore>, miden_client::ClientError> {
    let fut = async move {
        let mut builder = ClientBuilder::new()
            .rpc(rpc_client)
            .rng(Box::new(rng))
            .store(store as Arc<dyn Store>)
            .authenticator(keystore)
            .in_debug_mode(if debug_mode {
                DebugMode::Enabled
            } else {
                DebugMode::Disabled
            });

        if let Some(transport) = note_transport_client {
            builder = builder.note_transport(transport);
        }

        builder.build().await
    };

    // SAFETY: The ClientBuilder only contains Send types in practice — we use
    // StoreBuilder::Store (not Factory), and all other fields are Send.
    // The non-Send StoreFactory variant is never populated.
    unsafe { assert_send(fut).await }
}

/// Wraps a future in a Send assertion.
///
/// # Safety
/// The caller must ensure the future is actually safe to send between threads.
#[cfg(feature = "napi")]
pub(crate) unsafe fn assert_send<T>(
    fut: impl std::future::Future<Output = T>,
) -> impl std::future::Future<Output = T> + Send {
    struct AssertSend<F>(F);
    unsafe impl<F> Send for AssertSend<F> {}

    impl<F: std::future::Future> std::future::Future for AssertSend<F> {
        type Output = F::Output;

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            // SAFETY: We're just forwarding the poll. The outer unsafe contract
            // ensures this is only called when the future is actually Send-safe.
            unsafe { self.map_unchecked_mut(|s| &mut s.0).poll(cx) }
        }
    }

    AssertSend(fut)
}
