use std::path::PathBuf;

use miden_client::rpc::Endpoint;

/// Configuration for benchmark execution
#[derive(Clone)]
pub struct BenchConfig {
    /// RPC endpoint for network benchmarks
    pub network: Endpoint,
    /// Number of benchmark iterations
    pub iterations: usize,
    /// Number of storage maps in the account
    pub maps: usize,
    /// Number of key/value entries per storage map
    pub entries_per_map: usize,
}

impl BenchConfig {
    /// Creates a new benchmark configuration
    pub fn new(network: Endpoint, iterations: usize, maps: usize, entries_per_map: usize) -> Self {
        Self {
            network,
            iterations,
            maps,
            entries_per_map,
        }
    }

    /// Returns a temporary directory for benchmark artifacts
    #[allow(clippy::unused_self)]
    pub fn temp_dir(&self) -> PathBuf {
        std::env::temp_dir()
    }
}
