extern crate alloc;
use alloc::sync::Arc;
use std::fmt::Write;

use miden_client::keystore::WebKeyStore;
use miden_client::rpc::{Endpoint, NodeRpcClient, TonicRpcClient};
use miden_client::store::web_store::WebStore;
use miden_client::testing::mock::MockRpcApi;
use miden_client::{Client, ExecutionOptions};
use miden_objects::crypto::rand::RpoRandomCoin;
use miden_objects::{Felt, MAX_TX_EXECUTION_CYCLES, MIN_TX_EXECUTION_CYCLES};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use wasm_bindgen::prelude::*;

pub mod account;
pub mod export;
pub mod helpers;
pub mod import;
pub mod mock;
pub mod models;
pub mod new_account;
pub mod new_transactions;
pub mod notes;
pub mod sync;
pub mod tags;
pub mod transactions;
pub mod utils;

#[wasm_bindgen]
pub struct WebClient {
    store: Option<Arc<WebStore>>,
    keystore: Option<WebKeyStore<RpoRandomCoin>>,
    inner: Option<Client<WebKeyStore<RpoRandomCoin>>>,
    mock_rpc_api: Option<Arc<MockRpcApi>>,
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
        WebClient {
            inner: None,
            store: None,
            keystore: None,
            mock_rpc_api: None,
        }
    }

    pub(crate) fn get_mut_inner(&mut self) -> Option<&mut Client<WebKeyStore<RpoRandomCoin>>> {
        self.inner.as_mut()
    }

    /// Creates a new client with the given node URL and optional seed.
    /// If `node_url` is `None`, it defaults to the testnet endpoint.
    #[wasm_bindgen(js_name = "createClient")]
    pub async fn create_client(
        &mut self,
        node_url: Option<String>,
        seed: Option<Vec<u8>>,
    ) -> Result<JsValue, JsValue> {
        let endpoint = node_url.map_or(Ok(Endpoint::testnet()), |url| {
            Endpoint::try_from(url.as_str()).map_err(|_| JsValue::from_str("Invalid node URL"))
        })?;

        let web_rpc_client = Arc::new(TonicRpcClient::new(&endpoint, 0));

        self.setup_client(web_rpc_client, seed).await?;

        Ok(JsValue::from_str("Client created successfully"))
    }

    /// Initializes the inner client and components with the given RPC client and optional seed.
    async fn setup_client(
        &mut self,
        rpc_client: Arc<dyn NodeRpcClient>,
        seed: Option<Vec<u8>>,
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

        let web_store = Arc::new(
            WebStore::new()
                .await
                .map_err(|_| JsValue::from_str("Failed to initialize WebStore"))?,
        );

        let keystore = WebKeyStore::new(rng);

        self.inner = Some(
            Client::new(
                rpc_client,
                Box::new(rng),
                web_store.clone(),
                Some(Arc::new(keystore.clone())),
                ExecutionOptions::new(
                    Some(MAX_TX_EXECUTION_CYCLES),
                    MIN_TX_EXECUTION_CYCLES,
                    false,
                    false,
                )
                .expect("Default executor's options should always be valid"),
                None,
                None,
            )
            .await
            .map_err(|err| js_error_with_context(err, "Failed to create client"))?,
        );

        self.store = Some(web_store);
        self.keystore = Some(keystore);

        Ok(())
    }
}

// ERROR HANDLING HELPERS
// ================================================================================================

fn js_error_with_context<T>(err: T, context: &str) -> JsValue
where
    T: core::error::Error,
{
    let mut error_string = context.to_string();
    let mut source = Some(&err as &dyn core::error::Error);
    while let Some(err) = source {
        write!(error_string, ": {err}").expect("writing to string should always succeeds");
        source = err.source();
    }
    JsValue::from(error_string)
}
