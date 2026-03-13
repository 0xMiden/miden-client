use alloc::sync::Arc;

#[cfg(feature = "browser")]
use idxdb_store::IdxdbStore;
use js_export_macro::js_export;
#[cfg(feature = "browser")]
use miden_client::store::Store;
#[cfg(feature = "browser")]
use miden_client::testing::MockChain;
use miden_client::testing::mock::MockRpcApi;
use miden_client::testing::note_transport::MockNoteTransportApi;
#[cfg(feature = "browser")]
use miden_client::testing::note_transport::MockNoteTransportNode;
use miden_client::utils::Serializable;
#[cfg(feature = "browser")]
use miden_client::utils::{Deserializable, RwLock};
#[cfg(feature = "browser")]
use wasm_bindgen::prelude::*;

use crate::platform::{JsErr, from_str_err};
use crate::{WebClient, js_error_with_context};
#[cfg(feature = "browser")]
use crate::{WebKeyStore, create_rng};

#[cfg(feature = "browser")]
#[wasm_bindgen]
impl WebClient {
    /// Creates a new client with a mock RPC API. Useful for testing purposes and proof-of-concept
    /// applications as it uses a mock chain that simulates the behavior of a real node.
    #[wasm_bindgen(js_name = "createMockClient")]
    pub async fn create_mock_client(
        &mut self,
        seed: Option<Vec<u8>>,
        serialized_mock_chain: Option<Vec<u8>>,
        serialized_mock_note_transport_node: Option<Vec<u8>>,
    ) -> Result<JsValue, JsValue> {
        let mock_rpc_api = match serialized_mock_chain {
            Some(chain) => {
                Arc::new(MockRpcApi::new(MockChain::read_from_bytes(&chain).map_err(|err| {
                    js_error_with_context(err, "failed to deserialize mock chain")
                })?))
            },
            None => Arc::new(MockRpcApi::default()),
        };

        let mock_note_transport_api = match serialized_mock_note_transport_node {
            Some(node_bytes) => {
                let node = MockNoteTransportNode::read_from_bytes(&node_bytes).map_err(|err| {
                    js_error_with_context(err, "failed to deserialize mock note transport node")
                })?;
                Arc::new(MockNoteTransportApi::new(Arc::new(RwLock::new(node))))
            },
            None => Arc::new(MockNoteTransportApi::default()),
        };

        let store_name = "mock_client_db".to_owned();
        let rng = create_rng(seed)?;
        let store: Arc<dyn Store> = Arc::new(
            IdxdbStore::new(store_name.clone())
                .await
                .map_err(|_| JsValue::from_str("Failed to initialize IdxdbStore"))?,
        );
        let keystore = WebKeyStore::new_with_callbacks(rng, store_name, None, None, None);

        self.setup_client(
            mock_rpc_api.clone(),
            store,
            keystore,
            rng,
            Some(mock_note_transport_api.clone()),
            None,
        )
        .await?;

        self.mock_rpc_api = Some(mock_rpc_api);
        self.mock_note_transport_api = Some(mock_note_transport_api);

        Ok(JsValue::from_str("Client created successfully"))
    }
}

#[js_export]
impl WebClient {
    /// Returns the inner serialized mock chain if it exists.
    #[js_export(js_name = "serializeMockChain")]
    pub fn serialize_mock_chain(&self) -> Result<Vec<u8>, JsErr> {
        self.mock_rpc_api
            .as_ref()
            .map(|api| api.mock_chain.read().to_bytes())
            .ok_or_else(|| {
                from_str_err("Mock chain is not initialized. Create a mock client first.")
            })
    }

    /// Returns the inner serialized mock note transport node if it exists.
    #[js_export(js_name = "serializeMockNoteTransportNode")]
    pub fn serialize_mock_note_transport_node(&self) -> Result<Vec<u8>, JsErr> {
        self.mock_note_transport_api
            .as_ref()
            .map(|api| api.mock_node.read().to_bytes())
            .ok_or_else(|| {
                from_str_err(
                    "Mock note transport node is not initialized. Create a mock client first.",
                )
            })
    }

    #[js_export(js_name = "proveBlock")]
    pub fn prove_block(&self) -> Result<(), JsErr> {
        match self.mock_rpc_api.as_ref() {
            Some(api) => {
                api.prove_block();
                Ok(())
            },
            None => Err(from_str_err("WebClient does not have a mock chain.")),
        }
    }

    #[js_export(js_name = "usesMockChain")]
    pub fn uses_mock_chain(&self) -> bool {
        self.mock_rpc_api.is_some()
    }
}
