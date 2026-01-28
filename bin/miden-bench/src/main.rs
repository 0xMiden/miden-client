use std::str::FromStr;
use std::time::Instant;

use clap::{Args, Parser, Subcommand};
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
            Box::pin(deploy::deploy_account(
                &args.network.to_endpoint(),
                deploy_args.maps,
                deploy_args.entries_per_map,
            ))
            .await
        });

        match result {
            Ok((account_id, seed)) => {
                let seed_hex = hex::encode(seed);
                println!("Account ID: {account_id}");
                println!("Seed: {seed_hex}");
                println!();
                println!("Run benchmarks with:");
                println!(
                    "  miden-bench transaction --account-id {account_id} --seed {seed_hex} --maps {} --entries-per-map {}",
                    deploy_args.maps, deploy_args.entries_per_map
                );
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
        Command::Transaction(tx_args) => {
            let config = BenchConfig::new(
                args.network.to_endpoint(),
                args.iterations,
                tx_args.maps,
                tx_args.entries_per_map,
            );
            let mut runner = BenchmarkRunner::new(config);
            let seed = parse_seed_hex(&tx_args.seed)?;
            Box::pin(runner.run_transaction_benchmarks(tx_args.account_id.clone(), seed)).await
        },
        Command::Deploy(_) => unreachable!("Deploy is handled separately"),
    }
}

/// Parses a hex-encoded 32-byte seed string into a byte array
fn parse_seed_hex(hex_str: &str) -> anyhow::Result<[u8; 32]> {
    let bytes = hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Invalid seed hex: {e}"))?;
    bytes
        .try_into()
        .map_err(|v: Vec<u8>| anyhow::anyhow!("Seed must be 32 bytes, got {}", v.len()))
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

/// Storage configuration options for benchmarks
#[derive(Args, Clone)]
struct StorageArgs {
    /// Number of storage maps in the account
    #[arg(short, long, default_value = "1")]
    maps: usize,

    /// Number of key/value entries per storage map
    #[arg(short, long, default_value = "10")]
    entries_per_map: usize,
}

/// Transaction benchmark options
#[derive(Args, Clone)]
struct TransactionArgs {
    /// Public account ID to benchmark (hex format, e.g., 0x...)
    #[arg(short, long)]
    account_id: String,

    /// Account seed for signing (hex, output by the deploy command)
    #[arg(short, long)]
    seed: String,

    /// Number of storage maps in the account (must match deploy config)
    #[arg(short, long, default_value = "1")]
    maps: usize,

    /// Number of key/value entries per storage map (must match deploy config)
    #[arg(short, long, default_value = "10")]
    entries_per_map: usize,
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Benchmark transaction operations: read all storage entries from account (requires node)
    Transaction(TransactionArgs),
    /// Deploy a public wallet with configurable storage to the network (requires node)
    Deploy(StorageArgs),
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
