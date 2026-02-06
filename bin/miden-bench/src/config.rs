use std::path::PathBuf;

use miden_client::rpc::Endpoint;

/// Configuration for benchmark execution
#[derive(Clone)]
pub struct BenchConfig {
    /// RPC endpoint for network benchmarks
    pub network: Endpoint,
    /// Number of benchmark iterations
    pub iterations: usize,
}

impl BenchConfig {
    /// Creates a new benchmark configuration
    pub fn new(network: Endpoint, iterations: usize) -> Self {
        Self { network, iterations }
    }

    /// Returns a temporary directory for benchmark artifacts
    #[allow(clippy::unused_self)]
    pub fn temp_dir(&self) -> PathBuf {
        std::env::temp_dir()
    }
}
