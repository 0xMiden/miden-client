use alloc::sync::Arc;

use miden_client::testing::MockChain;
use miden_client::testing::mock::MockRpcApi;
use miden_client::testing::note_transport::{MockNoteTransportApi, MockNoteTransportNode};
use miden_client::utils::{Deserializable, RwLock, Serializable};
use wasm_bindgen::prelude::*;

use crate::{WebClient, js_error_with_context};

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

        self.setup_client(
            mock_rpc_api.clone(),
            "MockClientDB".to_string(),
            Some(mock_note_transport_api.clone()),
            seed,
            None,
            None,
            None,
        )
        .await?;

        self.mock_rpc_api = Some(mock_rpc_api);
        self.mock_note_transport_api = Some(mock_note_transport_api);

        Ok(JsValue::from_str("Client created successfully"))
    }

    /// Returns the inner serialized mock chain if it exists.
    #[wasm_bindgen(js_name = "serializeMockChain")]
    pub fn serialize_mock_chain(&mut self) -> Result<Vec<u8>, JsValue> {
        self.mock_rpc_api
            .as_ref()
            .map(|api| api.mock_chain.read().to_bytes())
            .ok_or_else(|| {
                JsValue::from_str("Mock chain is not initialized. Create a mock client first.")
            })
    }

    /// Returns the inner serialized mock note transport node if it exists.
    #[wasm_bindgen(js_name = "serializeMockNoteTransportNode")]
    pub fn serialize_mock_note_transport_node(&mut self) -> Result<Vec<u8>, JsValue> {
        self.mock_note_transport_api
            .as_ref()
            .map(|api| api.mock_node.read().to_bytes())
            .ok_or_else(|| {
                JsValue::from_str(
                    "Mock note transport node is not initialized. Create a mock client first.",
                )
            })
    }

    #[wasm_bindgen(js_name = "proveBlock")]
    pub fn prove_block(&mut self) -> Result<(), JsValue> {
        match self.mock_rpc_api.as_ref() {
            Some(api) => {
                api.prove_block();
                Ok(())
            },
            None => Err(JsValue::from_str("WebClient does not have a mock chain.")),
        }
    }

    #[wasm_bindgen(js_name = "usesMockChain")]
    pub fn uses_mock_chain(&self) -> bool {
        self.mock_rpc_api.is_some()
    }
}
