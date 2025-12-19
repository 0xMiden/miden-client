use alloc::sync::Arc;
use core::time::Duration;

use miden_client::RemoteTransactionProver;
use miden_client::transaction::{
    LocalTransactionProver,
    ProvingOptions,
    TransactionProver as TransactionProverTrait,
};
use wasm_bindgen::prelude::*;

/// Wrapper over local or remote transaction proving backends.
#[wasm_bindgen]
#[derive(Clone)]
pub struct TransactionProver {
    prover: Arc<dyn TransactionProverTrait + Send + Sync>,
    endpoint: Option<String>,
    timeout: Option<Duration>,
}

#[wasm_bindgen]
impl TransactionProver {
    /// Creates a prover that uses the local proving backend.
    #[wasm_bindgen(js_name = "newLocalProver")]
    pub fn new_local_prover() -> TransactionProver {
        let local_prover = LocalTransactionProver::new(ProvingOptions::default());
        TransactionProver {
            prover: Arc::new(local_prover),
            endpoint: None,
            timeout: None,
        }
    }

    /// Creates a new remote transaction prover.
    ///
    /// Arguments:
    /// - `endpoint`: The URL of the remote prover.
    /// - `timeout_ms`: The timeout in milliseconds for the remote prover.
    #[wasm_bindgen(js_name = "newRemoteProver")]
    pub fn new_remote_prover(endpoint: &str, timeout_ms: Option<u64>) -> TransactionProver {
        let mut remote_prover = RemoteTransactionProver::new(endpoint);

        let timeout = if let Some(timeout) = timeout_ms {
            let timeout = Duration::from_millis(timeout);
            remote_prover = remote_prover.with_timeout(timeout);
            Some(timeout)
        } else {
            None
        };

        TransactionProver {
            prover: Arc::new(remote_prover),
            endpoint: Some(endpoint.to_string()),
            timeout,
        }
    }

    /// Serializes the prover configuration into a string descriptor.
    pub fn serialize(&self) -> String {
        match (&self.endpoint, &self.timeout) {
            (Some(ep), Some(timeout)) => {
                format!("remote:{ep}")
                    + &format!(
                        ":{}",
                        u64::try_from(timeout.as_millis())
                            .expect("timeout was created from u64 milliseconds")
                    )
            },
            (Some(ep), None) => format!("remote:{ep}"),
            (None, _) => "local".to_string(),
        }
    }

    /// Reconstructs a prover from its serialized descriptor.
    pub fn deserialize(
        prover_type: &str,
        endpoint: Option<String>,
        timeout_ms: Option<u64>,
    ) -> Result<TransactionProver, JsValue> {
        match prover_type {
            "local" => Ok(TransactionProver::new_local_prover()),
            "remote" => {
                if let Some(ep) = endpoint {
                    Ok(TransactionProver::new_remote_prover(&ep, timeout_ms))
                } else {
                    Err(JsValue::from_str("Remote prover requires an endpoint"))
                }
            },
            _ => Err(JsValue::from_str("Invalid prover type")),
        }
    }

    /// Returns the endpoint if this is a remote prover.
    pub fn endpoint(&self) -> Option<String> {
        self.endpoint.clone()
    }
}

impl TransactionProver {
    /// Returns the underlying proving trait object.
    pub fn get_prover(&self) -> Arc<dyn TransactionProverTrait + Send + Sync> {
        self.prover.clone()
    }
}

impl From<Arc<dyn TransactionProverTrait + Send + Sync>> for TransactionProver {
    fn from(prover: Arc<dyn TransactionProverTrait + Send + Sync>) -> Self {
        TransactionProver { prover, endpoint: None, timeout: None }
    }
}
