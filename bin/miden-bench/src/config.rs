use std::path::PathBuf;

use miden_client::rpc::Endpoint;

/// Configuration for benchmark execution
#[derive(Clone)]
pub struct BenchConfig {
    /// RPC endpoint for network benchmarks
    pub network: Endpoint,
    /// Number of benchmark iterations
    pub iterations: usize,
    /// Optional persistent store directory. When set, deploy saves the store here
    /// and transaction reuses it instead of creating temporary directories.
    pub store_path: Option<PathBuf>,
}

impl BenchConfig {
    /// Creates a new benchmark configuration
    pub fn new(network: Endpoint, iterations: usize, store_path: Option<PathBuf>) -> Self {
        Self { network, iterations, store_path }
    }

    /// Returns a temporary directory for benchmark artifacts
    #[allow(clippy::unused_self)]
    pub fn temp_dir(&self) -> PathBuf {
        std::env::temp_dir()
    }
}
