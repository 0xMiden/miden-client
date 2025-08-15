use alloc::sync::Arc;

use miden_client::testing::MockChain;
use miden_client::testing::mock::MockRpcApi;
use miden_lib::utils::{Deserializable, Serializable};
use wasm_bindgen::prelude::*;

use crate::WebClient;

#[wasm_bindgen]
impl WebClient {
    /// Creates a new client with a mock RPC API. Useful for testing purposes and proof-of-concept
    /// applications as it uses a mock chain that simulates the behavior of a real node.
    #[wasm_bindgen(js_name = "createMockClient")]
    pub async fn create_mock_client(
        &mut self,
        seed: Option<Vec<u8>>,
        serialized_mock_chain: Option<Vec<u8>>,
    ) -> Result<JsValue, JsValue> {
        let mock_rpc_api = match serialized_mock_chain {
            Some(chain) => {
                Arc::new(MockRpcApi::new(MockChain::read_from_bytes(&chain).map_err(|err| {
                    JsValue::from_str(&format!("Failed to deserialize mock chain: {err}"))
                })?))
            },
            None => Arc::new(MockRpcApi::default()),
        };

        self.setup_client(mock_rpc_api.clone(), seed).await?;

        self.mock_rpc_api = Some(mock_rpc_api);

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
