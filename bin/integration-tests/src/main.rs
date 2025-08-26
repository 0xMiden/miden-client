use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::anyhow;
use clap::Parser;
use miden_client::rpc::Endpoint;
use regex::Regex;
use url::Url;

use crate::tests::client::*;
use crate::tests::config::ClientConfig;
use crate::tests::custom_transaction::*;
use crate::tests::fpi::*;
use crate::tests::network_transaction::*;
use crate::tests::onchain::*;
use crate::tests::swap_transaction::*;

mod tests;

// MAIN
// ================================================================================================

/// Entry point for the integration test binary.
///
/// Parses command line arguments, filters tests based on provided criteria, and runs the selected
/// tests in parallel. Exits with code 1 if any tests fail.
fn main() {
    let args = Args::parse();

    let all_tests = get_all_tests();
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
            eprintln!("Error: Failed to create configuration: {}", e);
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
    #[arg(
        short,
        long,
        default_value = "http://localhost:57291",
        env = "TEST_MIDEN_RPC_ENDPOINT"
    )]
    rpc_endpoint: Url,

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
        let host = args
            .rpc_endpoint
            .host_str()
            .ok_or_else(|| anyhow!("RPC endpoint URL is missing a host"))?
            .to_string();

        let port = args
            .rpc_endpoint
            .port()
            .ok_or_else(|| anyhow!("RPC endpoint URL is missing a port"))?;

        let endpoint = Endpoint::new(args.rpc_endpoint.scheme().to_string(), host, Some(port));
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
    CustomTransaction,
    Fpi,
    NetworkTransaction,
    Onchain,
    SwapTransaction,
}

impl AsRef<str> for TestCategory {
    fn as_ref(&self) -> &str {
        match self {
            TestCategory::Client => "client",
            TestCategory::CustomTransaction => "custom_transaction",
            TestCategory::Fpi => "fpi",
            TestCategory::NetworkTransaction => "network_transaction",
            TestCategory::Onchain => "onchain",
            TestCategory::SwapTransaction => "swap_transaction",
        }
    }
}

/// Returns all available test cases organized by category.
///
/// This function defines the complete list of integration tests available in the test suite,
/// categorized by functionality area.
fn get_all_tests() -> Vec<TestCase> {
    vec![
        // CLIENT tests
        TestCase::new(
            "client_builder_initializes_client_with_endpoint",
            TestCategory::Client,
            client_builder_initializes_client_with_endpoint,
        ),
        TestCase::new("multiple_tx_on_same_block", TestCategory::Client, multiple_tx_on_same_block),
        TestCase::new("import_expected_notes", TestCategory::Client, import_expected_notes),
        TestCase::new(
            "import_expected_note_uncommitted",
            TestCategory::Client,
            import_expected_note_uncommitted,
        ),
        TestCase::new(
            "import_expected_notes_from_the_past_as_committed",
            TestCategory::Client,
            import_expected_notes_from_the_past_as_committed,
        ),
        TestCase::new("get_account_update", TestCategory::Client, get_account_update),
        TestCase::new("sync_detail_values", TestCategory::Client, sync_detail_values),
        TestCase::new(
            "multiple_transactions_can_be_committed_in_different_blocks_without_sync",
            TestCategory::Client,
            multiple_transactions_can_be_committed_in_different_blocks_without_sync,
        ),
        TestCase::new(
            "consume_multiple_expected_notes",
            TestCategory::Client,
            consume_multiple_expected_notes,
        ),
        TestCase::new(
            "import_consumed_note_with_proof",
            TestCategory::Client,
            import_consumed_note_with_proof,
        ),
        TestCase::new(
            "import_consumed_note_with_id",
            TestCategory::Client,
            import_consumed_note_with_id,
        ),
        TestCase::new("import_note_with_proof", TestCategory::Client, import_note_with_proof),
        TestCase::new("discarded_transaction", TestCategory::Client, discarded_transaction),
        TestCase::new("custom_transaction_prover", TestCategory::Client, custom_transaction_prover),
        TestCase::new("locked_account", TestCategory::Client, locked_account),
        TestCase::new("expired_transaction_fails", TestCategory::Client, expired_transaction_fails),
        TestCase::new("unused_rpc_api", TestCategory::Client, unused_rpc_api),
        TestCase::new("ignore_invalid_notes", TestCategory::Client, ignore_invalid_notes),
        TestCase::new("output_only_note", TestCategory::Client, output_only_note),
        // CUSTOM TRANSACTION tests
        TestCase::new("merkle_store", TestCategory::CustomTransaction, merkle_store),
        TestCase::new(
            "onchain_notes_sync_with_tag",
            TestCategory::CustomTransaction,
            onchain_notes_sync_with_tag,
        ),
        TestCase::new("transaction_request", TestCategory::CustomTransaction, transaction_request),
        // FPI tests
        TestCase::new("standard_fpi_public", TestCategory::Fpi, standard_fpi_public),
        TestCase::new("standard_fpi_private", TestCategory::Fpi, standard_fpi_private),
        TestCase::new("fpi_execute_program", TestCategory::Fpi, fpi_execute_program),
        TestCase::new("nested_fpi_calls", TestCategory::Fpi, nested_fpi_calls),
        // NETWORK TRANSACTION tests
        TestCase::new(
            "counter_contract_ntx",
            TestCategory::NetworkTransaction,
            counter_contract_ntx,
        ),
        TestCase::new(
            "recall_note_before_ntx_consumes_it",
            TestCategory::NetworkTransaction,
            recall_note_before_ntx_consumes_it,
        ),
        // ONCHAIN tests
        TestCase::new("import_account_by_id", TestCategory::Onchain, import_account_by_id),
        TestCase::new("onchain_accounts", TestCategory::Onchain, onchain_accounts),
        TestCase::new("onchain_notes_flow", TestCategory::Onchain, onchain_notes_flow),
        TestCase::new("incorrect_genesis", TestCategory::Onchain, incorrect_genesis),
        // SWAP TRANSACTION tests
        TestCase::new("swap_fully_onchain", TestCategory::SwapTransaction, swap_fully_onchain),
        TestCase::new("swap_private", TestCategory::SwapTransaction, swap_private),
    ]
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
            eprintln!("Warning: Invalid regex pattern in filter: {}", filter_pattern);
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
            eprintln!("Warning: Invalid regex pattern in exclude: {}", exclude_pattern);
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
        output.push_str(&format!("{}", err));
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
    println!("Running {} tests with {} parallel jobs...", total_tests, jobs);
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
                    println!("[Worker {}] Starting test: {}", worker_id, test_name);
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
                    println!("   Error: {}", error);
                }

                // Update results
                results.lock().unwrap().push(result);

                // Update and print progress
                let mut count = completed_count.lock().unwrap();
                *count += 1;
                let progress = *count;
                drop(count);

                if !verbose {
                    println!("   Progress: {}/{}", progress, total_tests);
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
    println!("Passed: {} tests", passed);
    println!("Failed: {} tests", failed);
    println!("Total time: {:.2}s", total_duration.as_secs_f64());

    if failed > 0 {
        println!("\nFailed tests:");
        for result in results.iter().filter(|r| !r.passed) {
            println!("  - {} ({})", result.name, result.category.as_ref());
            if let Some(ref error) = result.error_message {
                println!("    Error: {}", error);
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
