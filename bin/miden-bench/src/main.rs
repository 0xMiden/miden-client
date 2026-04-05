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
mod masm;
mod metrics;
mod report;

use config::{BenchConfig, DEFAULT_STORE_DIR};

const DEFAULT_ITERATION_COUNT: usize = 5;

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

    /// Path to the persistent store directory. All commands share this directory
    /// for the `SQLite` database and filesystem keystore.
    #[arg(long, global = true, default_value = DEFAULT_STORE_DIR)]
    store: String,
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

// MAIN
// ================================================================================================

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    let store_flag = if args.store == DEFAULT_STORE_DIR {
        String::new()
    } else {
        format!(" --store {}", args.store)
    };

    let store_path = PathBuf::from(&args.store);
    let endpoint = args.network.to_endpoint();

    // Initialize persistent store directory and client
    std::fs::create_dir_all(&store_path).expect("Failed to create store directory");

    println!("Network: {endpoint}");
    println!("Store directory: {}", store_path.display());

    let mut client = config::create_client(&endpoint, &store_path)
        .await
        .expect("Failed to create client");

    println!("Connecting to node at {endpoint}...");
    client.sync_state().await.expect("Failed to sync with node");
    let chain_height = client.get_sync_height().await.expect("Failed to get sync height");
    println!("Connected successfully. Chain height: {chain_height}");

    match args.command {
        Command::Deploy(deploy_args) => {
            let result =
                Box::pin(deploy::deploy_account(&mut client, &store_path, deploy_args.maps)).await;

            match result {
                Ok(account_id) => {
                    println!();
                    println!("Expand storage with:");
                    println!(
                        "  miden-bench{store_flag} expand --account-id {account_id} --map-idx 0 --offset 0 --count 100"
                    );
                },
                Err(e) => {
                    panic!("Deploy failed: {e:?}");
                },
            }
        },
        Command::Expand(expand_args) => {
            let result = Box::pin(expand::expand_storage(
                &mut client,
                &expand_args.account_id,
                expand_args.map_idx,
                expand_args.offset,
                expand_args.count,
            ))
            .await;

            match result {
                Ok(()) => {
                    println!();
                    println!("Run benchmarks with:");
                    println!(
                        "  miden-bench{store_flag} transaction --account-id {} --iterations {DEFAULT_ITERATION_COUNT}",
                        expand_args.account_id
                    );
                },
                Err(e) => {
                    panic!("Expand failed: {e:?}");
                },
            }
        },
        Command::Transaction(ref tx_args) => {
            let start_time = Instant::now();
            let config = BenchConfig::new(endpoint, tx_args.iterations, store_path);
            let results = Box::pin(benchmarks::transaction::run_transaction_benchmarks(
                &mut client,
                &config,
                tx_args.account_id.clone(),
                tx_args.reads,
            ))
            .await;
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

// HELPERS
// ================================================================================================

fn parse_maps(s: &str) -> Result<usize, String> {
    let n: usize = s.parse().map_err(|e| format!("{e}"))?;
    if (1..=100).contains(&n) {
        Ok(n)
    } else {
        Err(format!("storage map count must be between 1 and 100, got {n}"))
    }
}
