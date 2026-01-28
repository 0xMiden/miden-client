use std::str::FromStr;
use std::time::Instant;

use clap::{Args, Parser, Subcommand, ValueEnum};
use miden_client::rpc::Endpoint;

mod benchmarks;
mod config;
mod deploy;
mod generators;
mod metrics;
mod report;
mod runner;
mod spinner;

use config::BenchConfig;
use metrics::BenchmarkResult;
use runner::BenchmarkRunner;

// MAIN
// ================================================================================================

fn main() {
    let args = CliArgs::parse();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    // Handle deploy command separately (not a benchmark)
    if let Command::Deploy(deploy_args) = &args.command {
        let result = rt.block_on(async {
            deploy::deploy_account(&args.network.to_endpoint(), deploy_args.size).await
        });

        match result {
            Ok(account_id) => {
                println!("Account ID: {account_id}");
            },
            Err(e) => {
                eprintln!("Deploy failed: {e:?}");
                std::process::exit(1);
            },
        }
        return;
    }

    let start_time = Instant::now();
    let results = rt.block_on(async { Box::pin(run_benchmarks(&args)).await });

    let total_duration = start_time.elapsed();

    let title = match &args.command {
        Command::Export(_) => "Export Benchmark",
        Command::Sync(_) => "Sync Benchmark",
        Command::Transaction(_) => "Transaction Benchmark",
        Command::Deploy(_) => unreachable!(),
    };

    match results {
        Ok(results) => {
            report::print_results(&results, title, total_duration);
        },
        Err(e) => {
            eprintln!("Benchmark failed: {e:?}");
            std::process::exit(1);
        },
    }
}

async fn run_benchmarks(args: &CliArgs) -> anyhow::Result<Vec<BenchmarkResult>> {
    match &args.command {
        Command::Export(size_args) => {
            let config =
                BenchConfig::new(args.network.to_endpoint(), args.iterations, size_args.size);
            let mut runner = BenchmarkRunner::new(config);
            runner.run_export_benchmarks().await
        },
        Command::Sync(sync_args) => {
            let config = BenchConfig::new(
                args.network.to_endpoint(),
                args.iterations,
                AccountSize::default(),
            );
            let mut runner = BenchmarkRunner::new(config);
            runner.run_sync_benchmarks(sync_args.account_id.clone()).await
        },
        Command::Transaction(size_args) => {
            let config =
                BenchConfig::new(args.network.to_endpoint(), args.iterations, size_args.size);
            let mut runner = BenchmarkRunner::new(config);
            Box::pin(runner.run_transaction_benchmarks()).await
        },
        Command::Deploy(_) => unreachable!("Deploy is handled separately"),
    }
}

// ARGS
// ================================================================================================

/// Benchmarks for the Miden client library
#[derive(Parser)]
#[command(name = "miden-bench", about = "Benchmarks for the Miden client library", version)]
struct CliArgs {
    #[command(subcommand)]
    command: Command,

    /// Network environment: localhost, devnet, testnet, or a custom RPC URL
    #[arg(short, long, default_value = "localhost", env = "MIDEN_NETWORK", global = true)]
    network: Network,

    /// Number of benchmark iterations
    #[arg(short, long, default_value = "5", global = true)]
    iterations: usize,
}

/// Account size options for benchmarks
#[derive(Args, Clone)]
struct SizeArgs {
    /// Account size (controls number of storage entries: small=10, medium=100, large=1000,
    /// very-large=50000)
    #[arg(short, long, default_value = "medium")]
    size: AccountSize,
}

/// Sync benchmark options
#[derive(Args, Clone)]
struct SyncArgs {
    /// Public account ID to import from the network (hex format, e.g., 0x...)
    #[arg(short, long)]
    account_id: Option<String>,
}

/// Deploy command options
#[derive(Args, Clone)]
struct DeployArgs {
    /// Account size (controls number of storage entries: small=10, medium=100, large=1000,
    /// very-large=50000)
    #[arg(short, long, default_value = "medium")]
    size: AccountSize,
}

/// Transaction benchmark options
#[derive(Args, Clone)]
struct TransactionArgs {
    /// Transaction size (controls number of output notes: small=5, medium=50, large=100,
    /// very-large=1000)
    #[arg(short, long, default_value = "medium")]
    size: AccountSize,
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Benchmark account export/import (offline)
    Export(SizeArgs),
    /// Benchmark sync operations: no account tracking, import public account (requires node)
    Sync(SyncArgs),
    /// Benchmark transaction operations: execute, prove, full (requires node)
    Transaction(TransactionArgs),
    /// Deploy a public wallet with configurable storage to the network (requires node)
    Deploy(DeployArgs),
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum AccountSize {
    Small,
    #[default]
    Medium,
    Large,
    VeryLarge,
}

// NETWORK
// ================================================================================================

/// Network environment for benchmarks
#[derive(Debug, Clone)]
pub enum Network {
    /// Local development node (`http://localhost:57291`)
    Localhost,
    /// Miden devnet (`https://rpc.devnet.miden.io`)
    Devnet,
    /// Miden testnet (`https://rpc.testnet.miden.io`)
    Testnet,
    /// Custom RPC endpoint URL
    Custom(String),
}

impl Network {
    /// Converts the network to an RPC endpoint
    pub fn to_endpoint(&self) -> Endpoint {
        match self {
            Network::Localhost => Endpoint::default(),
            Network::Devnet => Endpoint::devnet(),
            Network::Testnet => Endpoint::testnet(),
            Network::Custom(url) => {
                Endpoint::try_from(url.as_str()).expect("Invalid custom endpoint URL")
            },
        }
    }
}

impl FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "localhost" | "local" => Ok(Network::Localhost),
            "devnet" | "dev" => Ok(Network::Devnet),
            "testnet" | "test" => Ok(Network::Testnet),
            // Treat anything else as a custom URL
            custom => Ok(Network::Custom(custom.to_string())),
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Localhost => write!(f, "localhost"),
            Network::Devnet => write!(f, "devnet"),
            Network::Testnet => write!(f, "testnet"),
            Network::Custom(url) => write!(f, "{url}"),
        }
    }
}
