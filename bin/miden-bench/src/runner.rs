use crate::benchmarks::transaction;
use crate::config::BenchConfig;
use crate::metrics::BenchmarkResult;

/// Benchmark runner that orchestrates benchmark execution
pub struct BenchmarkRunner {
    config: BenchConfig,
}

impl BenchmarkRunner {
    /// Creates a new benchmark runner with the given configuration
    pub fn new(config: BenchConfig) -> Self {
        Self { config }
    }

    /// Runs transaction benchmarks (requires node)
    pub async fn run_transaction_benchmarks(
        &mut self,
        account_id: String,
        seed: Option<[u8; 32]>,
        entries_per_map: Option<usize>,
    ) -> anyhow::Result<Vec<BenchmarkResult>> {
        println!("Network: {}", self.config.network);
        println!("Account ID: {account_id}");
        println!();

        Box::pin(transaction::run_transaction_benchmarks(
            &self.config,
            account_id,
            seed,
            entries_per_map,
        ))
        .await
    }
}
