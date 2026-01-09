use alloc::sync::Arc;
use core::time::Duration;

use miden_client::RemoteTransactionProver;
use miden_client::transaction::{
    LocalTransactionProver, ProvingOptions, TransactionProver as TransactionProverTrait,
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
    ///
    /// Format:
    /// - `"local"` for local prover
    /// - `"remote|{endpoint}"` for remote prover without timeout
    /// - `"remote|{endpoint}|{timeout_ms}"` for remote prover with timeout
    ///
    /// Uses `|` as delimiter since it's not a valid URL character.
    pub fn serialize(&self) -> String {
        match (&self.endpoint, &self.timeout) {
            (Some(ep), Some(timeout)) => {
                let timeout_ms = u64::try_from(timeout.as_millis())
                    .expect("timeout was created from u64 milliseconds");
                format!("remote|{ep}|{timeout_ms}")
            },
            (Some(ep), None) => format!("remote|{ep}"),
            (None, _) => "local".to_string(),
        }
    }

    /// Reconstructs a prover from its serialized descriptor.
    ///
    /// Parses the format produced by `serialize()`:
    /// - `"local"` for local prover
    /// - `"remote|{endpoint}"` for remote prover without timeout
    /// - `"remote|{endpoint}|{timeout_ms}"` for remote prover with timeout
    pub fn deserialize(payload: &str) -> Result<TransactionProver, JsValue> {
        if payload == "local" {
            return Ok(TransactionProver::new_local_prover());
        }

        if let Some(rest) = payload.strip_prefix("remote|") {
            if rest.is_empty() {
                return Err(JsValue::from_str("Remote prover requires an endpoint"));
            }

            // Split on last `|` to extract optional timeout
            if let Some(last_pipe) = rest.rfind('|') {
                let endpoint = &rest[..last_pipe];
                let timeout_str = &rest[last_pipe + 1..];

                // Check if the suffix is a valid integer (timeout)
                if let Ok(timeout_ms) = timeout_str.parse::<u64>() {
                    return Ok(TransactionProver::new_remote_prover(endpoint, Some(timeout_ms)));
                }
            }

            // No valid timeout found, entire rest is the endpoint
            return Ok(TransactionProver::new_remote_prover(rest, None));
        }

        Err(JsValue::from_str(&format!("Invalid prover payload: {payload}")))
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
