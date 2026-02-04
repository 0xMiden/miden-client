use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser;
use miden_client::rpc::Endpoint;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::tests::config::ClientConfig;

mod generated_tests;
mod tests;

// MAIN
// ================================================================================================

/// Entry point for the integration test binary.
///
/// Parses command line arguments, filters tests based on provided criteria, and runs the selected
/// tests in parallel. Exits with code 1 if any tests fail.
fn main() {
    let args = Args::parse();

    let all_tests = generated_tests::get_all_tests();
    let filtered_tests = filter_tests(all_tests, &args);

    if args.list {
        list_tests(&filtered_tests);
        return;
    }

    if filtered_tests.is_empty() {
        println!("No tests match the specified filters.");
        return;
    }

    let base_config = match BaseConfig::try_from(args.clone()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: Failed to create configuration: {e}");
            std::process::exit(1);
        },
    };
    let start_time = Instant::now();

    let results = run_tests_parallel(filtered_tests, base_config, args.jobs, args.verbose);

    let total_duration = start_time.elapsed();
    print_summary(&results, total_duration);

    // Exit with error code if any tests failed
    let failed_count = results.iter().filter(|r| !r.passed).count();
    if failed_count > 0 {
        std::process::exit(1);
    }
}

// ARGS
// ================================================================================================

/// Command line arguments for the integration test binary.
#[derive(Parser, Clone)]
#[command(
    name = "miden-client-integration-tests",
    about = "Integration tests for the Miden client library",
    version
)]
struct Args {
    /// The URL of the RPC endpoint to use.
    ///
    /// The network to use. Options are `devnet`, `testnet`, `localhost` or a custom RPC endpoint.
    #[arg(short, long, default_value = "localhost", env = "TEST_MIDEN_NETWORK")]
    network: Network,

    /// Timeout for the RPC requests in milliseconds.
    #[arg(short, long, default_value = "10000")]
    timeout: u64,

    /// Number of tests to run in parallel. Set to 1 for sequential execution.
    #[arg(short, long, default_value_t = num_cpus::get())]
    jobs: usize,

    /// Filter tests by name (supports regex patterns).
    #[arg(short, long)]
    filter: Option<String>,

    /// List all available tests without running them.
    #[arg(long)]
    list: bool,

    /// Show verbose output including individual test timings.
    #[arg(short, long)]
    verbose: bool,

    /// Only run tests whose names contain this substring.
    #[arg(long)]
    contains: Option<String>,

    /// Exclude tests whose names match this pattern (supports regex).
    #[arg(long)]
    exclude: Option<String>,
}

/// Base configuration derived from command line arguments.
#[derive(Clone)]
struct BaseConfig {
    rpc_endpoint: Endpoint,
    timeout: u64,
}

impl TryFrom<Args> for BaseConfig {
    type Error = anyhow::Error;

    /// Creates a BaseConfig from command line arguments.
    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let endpoint = Endpoint::try_from(args.network.to_rpc_endpoint().as_str())
            .map_err(|e| anyhow::anyhow!("Invalid network: {:?}: {}", args.network, e))?;

        let timeout_ms = args.timeout;

        Ok(BaseConfig {
            rpc_endpoint: endpoint,
            timeout: timeout_ms,
        })
    }
}

// TYPE ALIASES
// ================================================================================================

/// Type alias for a test function that takes a ClientConfig and returns a boxed future
type TestFunction = Box<
    dyn Fn(ClientConfig) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>>>> + Send + Sync,
>;

// TEST CASE
// ================================================================================================

/// Represents a single test case with its name, category, and associated function.
struct TestCase {
    name: String,
    category: TestCategory,
    function: TestFunction,
}

impl TestCase {
    /// Creates a new TestCase with the given name, category, and function.
    fn new<F, Fut>(name: &str, category: TestCategory, func: F) -> Self
    where
        F: Fn(ClientConfig) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), anyhow::Error>> + 'static,
    {
        Self {
            name: name.to_string(),
            category,
            function: Box::new(move |config| Box::pin(func(config))),
        }
    }
}

impl std::fmt::Debug for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestCase")
            .field("name", &self.name)
            .field("category", &self.category)
            .field("function", &"<function>")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum TestCategory {
    Client,
    ClientService,
    CustomTransaction,
    Fpi,
    NetworkTransaction,
    Onchain,
    PassThrough,
    SwapTransaction,
    Transport,
}

impl AsRef<str> for TestCategory {
    fn as_ref(&self) -> &str {
        match self {
            TestCategory::Client => "client",
            TestCategory::ClientService => "client_service",
            TestCategory::CustomTransaction => "custom_transaction",
            TestCategory::Fpi => "fpi",
            TestCategory::NetworkTransaction => "network_transaction",
            TestCategory::Onchain => "onchain",
            TestCategory::PassThrough => "pass_through",
            TestCategory::SwapTransaction => "swap_transaction",
            TestCategory::Transport => "transport",
        }
    }
}

/// Represents the result of executing a test case.
#[derive(Debug)]
struct TestResult {
    name: String,
    category: TestCategory,
    passed: bool,
    duration: Duration,
    error_message: Option<String>,
}

impl TestResult {
    /// Creates a TestResult for a passed test.
    fn passed(name: String, category: TestCategory, duration: Duration) -> Self {
        Self {
            name,
            category,
            passed: true,
            duration,
            error_message: None,
        }
    }

    /// Creates a TestResult for a failed test with an error message.
    fn failed(name: String, category: TestCategory, duration: Duration, error: String) -> Self {
        Self {
            name,
            category,
            passed: false,
            duration,
            error_message: Some(error),
        }
    }
}

/// Filters the list of tests based on command line arguments.
///
/// Applies regex patterns, substring matching, and exclusion filters to select which tests should
/// be executed.
fn filter_tests(tests: Vec<TestCase>, args: &Args) -> Vec<TestCase> {
    let mut filtered_tests = tests;

    // Apply filter (regex pattern on test names)
    if let Some(ref filter_pattern) = args.filter {
        if let Ok(regex) = Regex::new(filter_pattern) {
            filtered_tests.retain(|test| regex.is_match(&test.name));
        } else {
            eprintln!("Warning: Invalid regex pattern in filter: {filter_pattern}");
        }
    }

    // Apply contains filter
    if let Some(ref contains) = args.contains {
        filtered_tests.retain(|test| test.name.contains(contains));
    }

    // Apply exclude filter
    if let Some(ref exclude_pattern) = args.exclude {
        if let Ok(regex) = Regex::new(exclude_pattern) {
            filtered_tests.retain(|test| !regex.is_match(&test.name));
        } else {
            eprintln!("Warning: Invalid regex pattern in exclude: {exclude_pattern}");
        }
    }

    filtered_tests
}

/// Prints all available tests organized by category.
///
/// Used when the --list flag is provided to show what tests are available without actually running
/// them.
fn list_tests(tests: &[TestCase]) {
    println!("Available tests:");
    println!("================");

    let mut tests_by_category: BTreeMap<TestCategory, Vec<&TestCase>> = BTreeMap::new();
    for test in tests {
        tests_by_category.entry(test.category.clone()).or_default().push(test);
    }

    for (category, tests) in tests_by_category {
        println!("\n{}:", category.as_ref().to_uppercase());
        for test in tests {
            println!("  - {}", test.name);
        }
    }

    println!("\nTotal: {} tests", tests.len());
}

/// Executes a single test and returns its result.
///
/// Creates a new Tokio runtime for the test, handles panics, and measures execution time. Each
/// test gets its own isolated configuration.
fn run_single_test(test_case: &TestCase, base_config: &BaseConfig) -> TestResult {
    let start_time = Instant::now();

    // Create a new runtime for this test
    let rt = tokio::runtime::Runtime::new().unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rt.block_on(async {
            // Create a unique ClientConfig for this test (with unique temporary directories)
            let client_config =
                ClientConfig::new(base_config.rpc_endpoint.clone(), base_config.timeout);

            // Call the stored test function directly
            (test_case.function)(client_config).await
        })
    }));

    let duration = start_time.elapsed();

    match result {
        Ok(Ok(_)) => {
            TestResult::passed(test_case.name.clone(), test_case.category.clone(), duration)
        },
        Ok(Err(e)) => {
            let error_msg = format_error_report(e);
            TestResult::failed(
                test_case.name.clone(),
                test_case.category.clone(),
                duration,
                error_msg,
            )
        },
        Err(panic_info) => {
            let error_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".into()
            };
            TestResult::failed(
                test_case.name.clone(),
                test_case.category.clone(),
                duration,
                error_msg,
            )
        },
    }
}

/// Formats an error with its full chain
fn format_error_report(error: anyhow::Error) -> String {
    let mut output = String::new();
    let mut first = true;

    for err in error.chain() {
        if !first {
            output.push_str("\n  Caused by: ");
        }
        output.push_str(&format!("{err}"));
        first = false;
    }

    output
}

/// Runs multiple tests in parallel using a specified number of worker threads.
///
/// Uses a shared work queue to distribute tests among worker threads. Provides real-time progress
/// updates and collects results from all workers.
fn run_tests_parallel(
    tests: Vec<TestCase>,
    base_config: BaseConfig,
    jobs: usize,
    verbose: bool,
) -> Vec<TestResult> {
    let total_tests = tests.len();
    println!("Running {total_tests} tests with {jobs} parallel jobs...");
    println!("==========================================================");
    println!("Using:");
    println!(" - RPC endpoint: {}", base_config.rpc_endpoint);
    println!(" - Timeout: {}ms", base_config.timeout);
    println!("==========================================================");

    let results = Arc::new(Mutex::new(Vec::new()));
    let completed_count = Arc::new(Mutex::new(0usize));

    // Use Arc<Mutex<>> to share the work queue
    let work_queue = Arc::new(Mutex::new(tests));

    // Spawn worker threads
    let mut handles = Vec::new();
    for worker_id in 0..jobs {
        let work_queue = Arc::clone(&work_queue);
        let base_config = base_config.clone();
        let results = Arc::clone(&results);
        let completed_count = Arc::clone(&completed_count);

        let handle = thread::spawn(move || {
            loop {
                // Get the next test to run
                let test = {
                    let mut queue = work_queue.lock().unwrap();
                    if queue.is_empty() {
                        break; // No more work
                    }
                    queue.pop().unwrap()
                };

                let test_name = test.name.clone();

                if verbose {
                    println!("[Worker {worker_id}] Starting test: {test_name}");
                }

                let result = run_single_test(&test, &base_config);

                let status = if result.passed { "PASSED" } else { "FAILED" };
                let duration_str = if result.duration.as_secs() > 0 {
                    format!("{:.2}s", result.duration.as_secs_f64())
                } else {
                    format!("{}ms", result.duration.as_millis())
                };

                if verbose {
                    println!(
                        "[Worker {}] {} - {}: {} ({})",
                        worker_id,
                        test_name,
                        result.category.as_ref(),
                        status,
                        duration_str
                    );
                } else {
                    println!(
                        " - {} ({}): {} ({})",
                        test_name,
                        result.category.as_ref(),
                        status,
                        duration_str
                    );
                }

                if !result.passed
                    && let Some(ref error) = result.error_message
                {
                    println!("   Error: {error}");
                }

                // Update results
                results.lock().unwrap().push(result);

                // Update and print progress
                let mut count = completed_count.lock().unwrap();
                *count += 1;
                let progress = *count;
                drop(count);

                if !verbose {
                    println!("   Progress: {progress}/{total_tests}");
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Extract results
    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}

/// Prints a comprehensive summary of test execution results.
///
/// Shows pass/fail counts, failed test details, and timing statistics including average, median,
/// min, and max execution times.
fn print_summary(results: &[TestResult], total_duration: Duration) {
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;

    println!("\n=== TEST SUMMARY ===");
    println!("Total: {} tests", results.len());
    println!("Passed: {passed} tests");
    println!("Failed: {failed} tests");
    println!("Total time: {:.2}s", total_duration.as_secs_f64());

    if failed > 0 {
        println!("\nFailed tests:");
        for result in results.iter().filter(|r| !r.passed) {
            println!("  - {} ({})", result.name, result.category.as_ref());
            if let Some(ref error) = result.error_message {
                println!("    Error: {error}");
            }
        }
    }

    // Print timing statistics
    if results.len() > 1 {
        let mut durations: Vec<_> = results.iter().map(|r| r.duration).collect();
        durations.sort();

        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let median_duration = durations[durations.len() / 2];
        let min_duration = durations[0];
        let max_duration = durations[durations.len() - 1];

        println!("\nTiming statistics:");
        println!("  Average: {:.2}s", avg_duration.as_secs_f64());
        println!("  Median:  {:.2}s", median_duration.as_secs_f64());
        println!("  Min:     {:.2}s", min_duration.as_secs_f64());
        println!("  Max:     {:.2}s", max_duration.as_secs_f64());
    }
}

// NETWORK
// ================================================================================================

/// Represents the network to which the client connects. It is used to determine the RPC endpoint
/// and network ID for the CLI.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Network {
    Custom(String),
    Devnet,
    Localhost,
    Testnet,
}

impl FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "devnet" => Ok(Network::Devnet),
            "localhost" => Ok(Network::Localhost),
            "testnet" => Ok(Network::Testnet),
            custom => Ok(Network::Custom(custom.to_string())),
        }
    }
}

impl Network {
    /// Converts the Network variant to its corresponding RPC endpoint string
    #[allow(dead_code)]
    pub fn to_rpc_endpoint(&self) -> String {
        match self {
            Network::Custom(custom) => custom.clone(),
            Network::Devnet => Endpoint::devnet().to_string(),
            Network::Localhost => Endpoint::default().to_string(),
            Network::Testnet => Endpoint::testnet().to_string(),
        }
    }
}
