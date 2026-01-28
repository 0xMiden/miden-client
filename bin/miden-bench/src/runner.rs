use crate::benchmarks::{account_export, sync, transaction};
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

    /// Runs account export/import benchmarks
    pub async fn run_export_benchmarks(&mut self) -> anyhow::Result<Vec<BenchmarkResult>> {
        println!("Account size: {:?}", self.config.size);
        println!();

        account_export::run_export_benchmarks(&self.config).await
    }

    /// Runs sync benchmarks (requires node)
    pub async fn run_sync_benchmarks(
        &mut self,
        account_id: Option<String>,
    ) -> anyhow::Result<Vec<BenchmarkResult>> {
        println!("Network: {}", self.config.network);
        if let Some(ref id) = account_id {
            println!("Account ID: {id}");
        }
        println!();

        sync::run_sync_benchmarks(&self.config, account_id).await
    }

    /// Runs transaction benchmarks (requires node)
    pub async fn run_transaction_benchmarks(&mut self) -> anyhow::Result<Vec<BenchmarkResult>> {
        println!("Network: {}", self.config.network);
        println!();

        Box::pin(transaction::run_transaction_benchmarks(&self.config)).await
    }
}
