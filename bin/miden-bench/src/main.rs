use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use clap::{Args, Parser, Subcommand};
use miden_client::rpc::Endpoint;

mod benchmarks;
mod config;
mod deploy;
mod expand;
mod generators;
mod metrics;
mod report;
mod runner;

use config::BenchConfig;
use metrics::BenchmarkResult;
use runner::BenchmarkRunner;

const DEFAULT_ITERATION_COUNT: usize = 5;

// MAIN
// ================================================================================================

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    match args.command {
        Command::Deploy(deploy_args) => {
            let result = Box::pin(deploy::deploy_account(
                &args.network.to_endpoint(),
                deploy_args.maps,
                args.store.as_deref(),
            ))
            .await;

            match result {
                Ok((account_id, seed)) => {
                    let seed_hex = hex::encode(seed);
                    let store_flag =
                        args.store.as_ref().map_or(String::new(), |s| format!(" --store {s}"));
                    println!();
                    println!("Expand storage with:");
                    println!(
                        "  miden-bench{store_flag} expand --account-id {account_id} --seed {seed_hex} --map-idx 0 --offset 0 --count 100"
                    );
                },
                Err(e) => {
                    panic!("Deploy failed: {e:?}");
                },
            }
        },
        Command::Expand(expand_args) => {
            let result = Box::pin(expand::expand_storage(
                &args.network.to_endpoint(),
                &expand_args.account_id,
                &expand_args.seed,
                expand_args.map_idx,
                expand_args.offset,
                expand_args.count,
                args.store.as_deref(),
            ))
            .await;

            match result {
                Ok(()) => {
                    println!();
                    println!("Run benchmarks with:");
                    let store_flag =
                        args.store.as_ref().map_or(String::new(), |s| format!(" --store {s}"));
                    println!(
                        "  miden-bench{store_flag} transaction --account-id {} --seed {} --iterations {DEFAULT_ITERATION_COUNT}",
                        expand_args.account_id, expand_args.seed
                    );
                },
                Err(e) => {
                    panic!("Expand failed: {e:?}");
                },
            }
        },
        Command::Transaction(ref tx_args) => {
            let start_time = Instant::now();
            let store_path = args.store.as_ref().map(PathBuf::from);
            let results = run_benchmarks(tx_args, &args.network, store_path).await;
            let total_duration = start_time.elapsed();

            match results {
                Ok(results) => {
                    report::print_results(&results, "Transaction Benchmark", total_duration);
                },
                Err(e) => {
                    panic!("Benchmark failed: {e:?}");
                },
            }
        },
    }
}

async fn run_benchmarks(
    tx_args: &TransactionArgs,
    network: &Network,
    store_path: Option<PathBuf>,
) -> anyhow::Result<Vec<BenchmarkResult>> {
    let config = BenchConfig::new(network.to_endpoint(), tx_args.iterations, store_path);
    let mut runner = BenchmarkRunner::new(config);
    let seed = tx_args.seed.as_deref().map(parse_seed_hex).transpose()?;
    Box::pin(runner.run_transaction_benchmarks(tx_args.account_id.clone(), seed, tx_args.reads))
        .await
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

    /// Path to a persistent store directory. When provided, deploy saves the store here
    /// and transaction reuses it. When omitted, temporary directories are used.
    #[arg(long, global = true)]
    store: Option<String>,
}

fn parse_maps(s: &str) -> Result<usize, String> {
    let n: usize = s.parse().map_err(|e| format!("{e}"))?;
    if (1..=100).contains(&n) {
        Ok(n)
    } else {
        Err(format!("storage map count must be between 1 and 100, got {n}"))
    }
}

/// Storage configuration options for benchmarks
#[derive(Args, Clone)]
struct StorageArgs {
    /// Number of storage maps in the account (1-100)
    #[arg(short, long, default_value = "1", value_parser = parse_maps)]
    maps: usize,
}

/// Transaction benchmark options
#[derive(Args, Clone)]
struct TransactionArgs {
    /// Public account ID to benchmark (hex format, e.g., 0x...)
    #[arg(short, long)]
    account_id: String,

    /// Account seed for signing (hex, output by the deploy command).
    /// When omitted, only execution is benchmarked (no proving or submission).
    #[arg(short, long)]
    seed: Option<String>,

    /// Maximum storage reads per transaction. When total entries exceed this limit,
    /// reads are split across multiple transactions per benchmark iteration.
    /// Each iteration's time is the sum across all transactions.
    /// When omitted, all entries are read in a single transaction.
    #[arg(short, long)]
    reads: Option<usize>,

    /// Number of benchmark iterations
    #[arg(short, long, default_value_t = DEFAULT_ITERATION_COUNT)]
    iterations: usize,
}

/// Expand storage: fill entries in a specific map of a deployed account
#[derive(Args, Clone)]
struct ExpandArgs {
    /// Public account ID to expand (hex format)
    #[arg(short, long)]
    account_id: String,

    /// Account seed for signing (hex, output by the deploy command)
    #[arg(short, long)]
    seed: String,

    /// Storage map index to fill (0-based, matches deploy --maps count)
    #[arg(short, long)]
    map_idx: usize,

    /// Starting entry offset (0-based)
    #[arg(short, long)]
    offset: usize,

    /// Number of entries to add starting from offset
    #[arg(short, long)]
    count: usize,
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Benchmark transaction operations: read all storage entries from account (requires node)
    Transaction(TransactionArgs),
    /// Deploy a public wallet with configurable storage to the network (requires node)
    Deploy(StorageArgs),
    /// Expand storage: fill entries in a specific map of a deployed account (requires node)
    Expand(ExpandArgs),
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
