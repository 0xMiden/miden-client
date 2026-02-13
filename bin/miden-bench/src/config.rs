use std::path::PathBuf;

use miden_client::rpc::Endpoint;

/// Default store directory name, created in the current working directory.
pub const DEFAULT_STORE_DIR: &str = "miden-bench-store";

/// Configuration for benchmark execution
#[derive(Clone)]
pub struct BenchConfig {
    /// RPC endpoint for network benchmarks
    pub network: Endpoint,
    /// Number of benchmark iterations
    pub iterations: usize,
    /// Persistent store directory. Deploy saves the account and keystore here;
    /// transaction and expand commands reuse the same directory.
    pub store_path: PathBuf,
}

impl BenchConfig {
    /// Creates a new benchmark configuration
    pub fn new(network: Endpoint, iterations: usize, store_path: PathBuf) -> Self {
        Self { network, iterations, store_path }
    }
}
