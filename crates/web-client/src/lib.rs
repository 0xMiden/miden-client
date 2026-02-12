extern crate alloc;
use alloc::sync::Arc;
use core::error::Error;
use core::fmt::Write;

use idxdb_store::WebStore;
use js_sys::{Function, Reflect};
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::note_transport::NoteTransportClient;
use miden_client::note_transport::grpc::GrpcNoteTransportClient;
use miden_client::rpc::{Endpoint, GrpcClient, NodeRpcClient};
use miden_client::store::Store;
use miden_client::testing::mock::MockRpcApi;
use miden_client::testing::note_transport::MockNoteTransportApi;
use miden_client::{Client, ClientError, DebugMode, ErrorHint, Felt};
use miden_client_sqlite_store::SqliteStore;
use models::code_builder::CodeBuilder;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
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
pub mod rpc_client;
pub mod settings;
pub mod sync;
pub mod tags;
pub mod transactions;
pub mod utils;

mod web_keystore;
mod web_keystore_callbacks;
pub use web_keystore::WebKeyStore;

const BASE_STORE_NAME: &str = "MidenClientDB";

#[wasm_bindgen]
pub struct WebClient {
    store: Option<Arc<dyn Store>>,
    /// Kept separately for IndexedDB-specific operations (export/import store).
    /// Only set when using the `IndexedDB` backend (`createClient`).
    idxdb_store: Option<Arc<WebStore>>,
    keystore: Option<WebKeyStore<RpoRandomCoin>>,
    inner: Option<Client<WebKeyStore<RpoRandomCoin>>>,
    mock_rpc_api: Option<Arc<MockRpcApi>>,
    mock_note_transport_api: Option<Arc<MockNoteTransportApi>>,
    debug_mode: bool,
}

impl Default for WebClient {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        WebClient {
            inner: None,
            store: None,
            idxdb_store: None,
            keystore: None,
            mock_rpc_api: None,
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
    #[wasm_bindgen(js_name = "setDebugMode")]
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }

    pub(crate) fn get_mut_inner(&mut self) -> Option<&mut Client<WebKeyStore<RpoRandomCoin>>> {
        self.inner.as_mut()
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
    #[wasm_bindgen(js_name = "createClient")]
    pub async fn create_client(
        &mut self,
        node_url: Option<String>,
        node_note_transport_url: Option<String>,
        seed: Option<Vec<u8>>,
        store_name: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let endpoint = node_url.map_or(Ok(Endpoint::testnet()), |url| {
            Endpoint::try_from(url.as_str()).map_err(|_| JsValue::from_str("Invalid node URL"))
        })?;

        let web_rpc_client = Arc::new(GrpcClient::new(&endpoint, 0));

        let note_transport_client = node_note_transport_url
            .map(|url| Arc::new(GrpcNoteTransportClient::new(url)) as Arc<dyn NoteTransportClient>);

        let store_name =
            store_name.unwrap_or(format!("{}_{}", BASE_STORE_NAME, endpoint.to_network_id()));

        let web_store = Arc::new(
            WebStore::new(store_name.clone())
                .await
                .map_err(|_| JsValue::from_str("Failed to initialize WebStore"))?,
        );

        self.idxdb_store = Some(web_store.clone());

        self.setup_client_with_store(
            web_rpc_client,
            web_store,
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

    /// Creates a new `WebClient` instance backed by `SQLite` (via WASM `SQLite` adapter).
    ///
    /// This is suitable for Node.js environments where `IndexedDB` is not available.
    /// The `SQLite` database is accessed through a JavaScript adapter layer.
    ///
    /// # Arguments
    /// * `node_url`: The URL of the node RPC endpoint. If `None`, defaults to the testnet endpoint.
    /// * `node_note_transport_url`: Optional URL of the note transport service.
    /// * `seed`: Optional seed for account initialization.
    /// * `store_name`: Optional name for the database. If `None`, defaults to
    ///   `MidenClientDB_{network_id}`.
    #[wasm_bindgen(js_name = "createClientWithSqlite")]
    pub async fn create_client_with_sqlite(
        &mut self,
        node_url: Option<String>,
        node_note_transport_url: Option<String>,
        seed: Option<Vec<u8>>,
        store_name: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let endpoint = node_url.map_or(Ok(Endpoint::testnet()), |url| {
            Endpoint::try_from(url.as_str()).map_err(|_| JsValue::from_str("Invalid node URL"))
        })?;

        let web_rpc_client = Arc::new(GrpcClient::new(&endpoint, 0));

        let note_transport_client = node_note_transport_url
            .map(|url| Arc::new(GrpcNoteTransportClient::new(url)) as Arc<dyn NoteTransportClient>);

        let store_name =
            store_name.unwrap_or(format!("{}_{}", BASE_STORE_NAME, endpoint.to_network_id()));

        let sqlite_store = Arc::new(
            SqliteStore::new(store_name.clone())
                .await
                .map_err(|_| JsValue::from_str("Failed to initialize SqliteStore"))?,
        );

        self.setup_client_with_store(
            web_rpc_client,
            sqlite_store,
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
            Endpoint::try_from(url.as_str()).map_err(|_| JsValue::from_str("Invalid node URL"))
        })?;

        let web_rpc_client = Arc::new(GrpcClient::new(&endpoint, 0));

        let note_transport_client = node_note_transport_url
            .map(|url| Arc::new(GrpcNoteTransportClient::new(url)) as Arc<dyn NoteTransportClient>);

        let store_name =
            store_name.unwrap_or(format!("{}_{}", BASE_STORE_NAME, endpoint.to_network_id()));

        let web_store = Arc::new(
            WebStore::new(store_name.clone())
                .await
                .map_err(|_| JsValue::from_str("Failed to initialize WebStore"))?,
        );

        self.idxdb_store = Some(web_store.clone());

        self.setup_client_with_store(
            web_rpc_client,
            web_store,
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

    #[allow(clippy::too_many_arguments)]
    async fn setup_client_with_store(
        &mut self,
        rpc_client: Arc<dyn NodeRpcClient>,
        store: Arc<dyn Store>,
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
                    return Err(JsValue::from_str("Seed must be exactly 32 bytes"));
                }
            },
            None => StdRng::from_os_rng(),
        };
        let coin_seed: [u64; 4] = rng.random();

        let rng = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

        let keystore =
            WebKeyStore::new_with_callbacks(rng, store_name, get_key_cb, insert_key_cb, sign_cb);

        let mut builder = ClientBuilder::new()
            .rpc(rpc_client)
            .rng(Box::new(rng))
            .store(store.clone())
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
            .map_err(|err| js_error_with_context(err, "Failed to create client"))?;

        // Ensure genesis block is fetched and stored.
        client
            .ensure_genesis_in_place()
            .await
            .map_err(|err| js_error_with_context(err, "Failed to ensure genesis in place"))?;

        self.inner = Some(client);
        self.store = Some(store);
        self.keystore = Some(keystore);

        Ok(())
    }

    #[wasm_bindgen(js_name = "createCodeBuilder")]
    pub fn create_code_builder(&self) -> Result<CodeBuilder, JsValue> {
        let Some(client) = &self.inner else {
            return Err("client was not initialized before instancing CodeBuilder".into());
        };
        Ok(CodeBuilder::from_source_manager(client.code_builder().source_manager().clone()))
    }
}

// ERROR HANDLING HELPERS
// ================================================================================================

fn js_error_with_context<T>(err: T, context: &str) -> JsValue
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

fn hint_from_error(err: &(dyn Error + 'static)) -> Option<String> {
    if let Some(client_error) = err.downcast_ref::<ClientError>() {
        return Option::<ErrorHint>::from(client_error).map(ErrorHint::into_help_message);
    }

    err.source().and_then(hint_from_error)
}
