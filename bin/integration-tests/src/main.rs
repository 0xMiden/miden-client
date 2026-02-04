use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use miden_client::rpc::Endpoint;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

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

    // If running as a subprocess for a single test, execute it and exit
    if let Some(ref test_name) = args.internal_run_test {
        run_single_test_subprocess(&args, test_name);
        return;
    }

    // Initialize tracing from RUST_LOG if set
    init_tracing();

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

    // Run initial test pass
    let results = run_tests_parallel(
        filtered_tests,
        base_config.clone(),
        args.jobs,
        args.verbose,
        args.output_format,
        false,
    );

    // Determine retry configuration
    let retry_enabled = !args.no_retry && args.retry_count > 0;
    let mut retry_results: Vec<TestResult> = Vec::new();

    if retry_enabled {
        let failed_test_names: Vec<String> =
            results.iter().filter(|r| !r.passed).map(|r| r.name.clone()).collect();

        if !failed_test_names.is_empty() {
            for retry_attempt in 1..=args.retry_count {
                if !matches!(args.output_format, OutputFormat::Json) {
                    println!("\n=== RETRY ATTEMPT {}/{} ===", retry_attempt, args.retry_count);
                    println!(
                        "Retrying {} failed test(s) with reduced parallelism...",
                        failed_test_names.len()
                    );
                }

                // Get fresh test cases for the failed tests
                let all_tests = generated_tests::get_all_tests();
                let tests_to_retry: Vec<TestCase> =
                    all_tests.into_iter().filter(|t| failed_test_names.contains(&t.name)).collect();

                if tests_to_retry.is_empty() {
                    break;
                }

                // Retry with reduced parallelism (half the jobs, minimum 1)
                let retry_jobs = (args.jobs / 2).max(1);

                let current_retry_results = run_tests_parallel(
                    tests_to_retry,
                    base_config.clone(),
                    retry_jobs,
                    args.verbose,
                    args.output_format,
                    true,
                );

                // Check if all retries passed
                let all_passed = current_retry_results.iter().all(|r| r.passed);
                retry_results.extend(current_retry_results);

                if all_passed {
                    break;
                }
            }
        }
    }

    let total_duration = start_time.elapsed();
    print_summary(&results, &retry_results, total_duration, args.output_format);

    // Exit with error code if any tests failed after retries
    let final_failed_count = if retry_results.is_empty() {
        results.iter().filter(|r| !r.passed).count()
    } else {
        // For tests that were retried, only count as failed if they failed in ALL retries
        let retried_test_names: std::collections::HashSet<_> =
            retry_results.iter().map(|r| &r.name).collect();

        let non_retried_failures = results
            .iter()
            .filter(|r| !r.passed && !retried_test_names.contains(&r.name))
            .count();

        let retry_failures = retry_results.iter().filter(|r| !r.passed).count();

        non_retried_failures + retry_failures
    };

    if final_failed_count > 0 {
        std::process::exit(1);
    }
}

/// Initializes tracing from RUST_LOG environment variable.
fn init_tracing() {
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer().with_target(true))
            .init();
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

    /// Show verbose output including worker IDs. Use RUST_LOG env var for tracing.
    #[arg(short, long)]
    verbose: bool,

    /// Only run tests whose names contain this substring.
    #[arg(long)]
    contains: Option<String>,

    /// Exclude tests whose names match this pattern (supports regex).
    #[arg(long)]
    exclude: Option<String>,

    /// Number of times to retry failed tests. Set to 0 to disable retries.
    #[arg(long, default_value = "1")]
    retry_count: usize,

    /// Disable automatic retry of failed tests.
    #[arg(long)]
    no_retry: bool,

    /// Output format for test results.
    #[arg(long, default_value = "human", value_enum)]
    output_format: OutputFormat,

    /// Internal: run a single test by name and exit (hidden from help).
    /// Used by the test runner to spawn subprocesses for parallel execution.
    #[arg(long, hide = true)]
    internal_run_test: Option<String>,
}

/// Output format for test results.
#[derive(Debug, Clone, Copy, Default, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum OutputFormat {
    /// Human-readable output format.
    #[default]
    Human,
    /// JSON output format for machine parsing.
    Json,
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
#[derive(Debug, Clone, Serialize)]
struct TestResult {
    name: String,
    category: String,
    passed: bool,
    #[serde(serialize_with = "serialize_duration")]
    duration: Duration,
    error_message: Option<String>,
    /// Indicates whether this result is from a retry attempt.
    is_retry: bool,
    /// Captured stdout from the test (only shown for failed tests).
    #[serde(skip_serializing_if = "Option::is_none")]
    captured_output: Option<String>,
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64(duration.as_secs_f64())
}

// SUBPROCESS RESULT
// ================================================================================================

/// Result type serialized by subprocess and parsed by parent process.
/// Uses f64 for duration (seconds) to avoid custom serde implementations.
#[derive(Debug, Serialize, Deserialize)]
struct SubprocessResult {
    name: String,
    category: String,
    passed: bool,
    duration_secs: f64,
    error_message: Option<String>,
}

impl SubprocessResult {
    fn passed(name: &str, category: &TestCategory, duration: Duration) -> Self {
        Self {
            name: name.to_string(),
            category: category.as_ref().to_string(),
            passed: true,
            duration_secs: duration.as_secs_f64(),
            error_message: None,
        }
    }

    fn failed(name: &str, category: &TestCategory, duration: Duration, error: &str) -> Self {
        Self {
            name: name.to_string(),
            category: category.as_ref().to_string(),
            passed: false,
            duration_secs: duration.as_secs_f64(),
            error_message: Some(error.to_string()),
        }
    }

    fn error(name: &str, error: &str) -> Self {
        Self {
            name: name.to_string(),
            category: "unknown".to_string(),
            passed: false,
            duration_secs: 0.0,
            error_message: Some(error.to_string()),
        }
    }
}

/// Runs a single test in subprocess mode.
///
/// This function is called when the binary is invoked with `--internal-run-test`.
/// It executes the named test, captures the result, and outputs it as JSON to stdout.
/// All other stdout from the test is preserved and will be captured by the parent process.
fn run_single_test_subprocess(args: &Args, test_name: &str) {
    let all_tests = generated_tests::get_all_tests();
    let test = all_tests.into_iter().find(|t| t.name == test_name);

    let Some(test) = test else {
        let result = SubprocessResult::error(test_name, "Test not found");
        println!("{}", serde_json::to_string(&result).unwrap());
        std::process::exit(1);
    };

    let base_config = match BaseConfig::try_from(args.clone()) {
        Ok(c) => c,
        Err(e) => {
            let result = SubprocessResult::error(test_name, &e.to_string());
            println!("{}", serde_json::to_string(&result).unwrap());
            std::process::exit(1);
        },
    };

    let start = Instant::now();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rt.block_on(async {
            let config = ClientConfig::new(base_config.rpc_endpoint.clone(), base_config.timeout);
            (test.function)(config).await
        })
    }));

    let duration = start.elapsed();
    let subprocess_result = match result {
        Ok(Ok(_)) => SubprocessResult::passed(test_name, &test.category, duration),
        Ok(Err(e)) => {
            SubprocessResult::failed(test_name, &test.category, duration, &format_error_report(e))
        },
        Err(panic) => {
            let msg = panic
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "Unknown panic".into());
            SubprocessResult::failed(test_name, &test.category, duration, &msg)
        },
    };

    // Output result as JSON to stdout (will be captured by parent)
    println!("{}", serde_json::to_string(&subprocess_result).unwrap());
    std::process::exit(if subprocess_result.passed { 0 } else { 1 });
}

impl TestResult {
    /// Marks this result as coming from a retry attempt.
    fn with_retry(mut self) -> Self {
        self.is_retry = true;
        self
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

/// Runs multiple tests in parallel using subprocess execution.
///
/// Each test is spawned as a separate subprocess, enabling true OS-level parallelism.
/// Stdout/stderr from each subprocess is captured automatically and associated with
/// the test result.
fn run_tests_parallel(
    tests: Vec<TestCase>,
    base_config: BaseConfig,
    jobs: usize,
    _verbose: bool,
    output_format: OutputFormat,
    is_retry: bool,
) -> Vec<TestResult> {
    let total_tests = tests.len();
    let is_json = matches!(output_format, OutputFormat::Json);
    let current_exe = std::env::current_exe().expect("Failed to get current executable");

    if !is_json {
        println!();
        let run_type = if is_retry { "Retrying" } else { "Starting" };
        println!("{run_type} {total_tests} tests across {jobs} workers");
        if !is_retry {
            println!("─────────────────────────────────────────────────────────");
            println!("  RPC endpoint: {}", base_config.rpc_endpoint);
            println!("  Timeout:      {}ms", base_config.timeout);
            println!("─────────────────────────────────────────────────────────");
        }
        println!();
    }

    // Convert tests to (name, category) pairs for subprocess spawning
    let test_info: Vec<(String, String)> = tests
        .iter()
        .map(|t| (t.name.clone(), t.category.as_ref().to_string()))
        .collect();

    let results = Arc::new(Mutex::new(Vec::new()));
    let completed_count = Arc::new(AtomicUsize::new(0));

    // Use Arc<Mutex<>> to share the work queue
    let work_queue = Arc::new(Mutex::new(test_info));

    // Mutex for serializing output to prevent interleaved lines
    let output_mutex = Arc::new(Mutex::new(()));

    // Get network endpoint string for passing to subprocess
    let network_endpoint = base_config.rpc_endpoint.to_string();
    let timeout = base_config.timeout;

    // Spawn worker threads (each spawns subprocesses)
    let mut handles = Vec::new();
    for _worker_id in 0..jobs {
        let work_queue = Arc::clone(&work_queue);
        let results = Arc::clone(&results);
        let completed_count = Arc::clone(&completed_count);
        let output_mutex = Arc::clone(&output_mutex);
        let current_exe = current_exe.clone();
        let network_endpoint = network_endpoint.clone();

        let handle = thread::spawn(move || {
            loop {
                // Get the next test to run
                let test = {
                    let mut queue = work_queue.lock().unwrap();
                    queue.pop()
                };

                let Some((test_name, test_category)) = test else {
                    break; // No more work
                };

                // Print "START" message
                if !is_json {
                    let _lock = output_mutex.lock().unwrap();
                    println!("        START  {}::{}", test_category, test_name);
                }

                // Spawn subprocess for this test
                let output = Command::new(&current_exe)
                    .arg("--internal-run-test")
                    .arg(&test_name)
                    .arg("--network")
                    .arg(&network_endpoint)
                    .arg("--timeout")
                    .arg(timeout.to_string())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output();

                let progress = completed_count.fetch_add(1, Ordering::SeqCst) + 1;

                let result = match output {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        // Parse the JSON result from the last line of stdout
                        let subprocess_result: Option<SubprocessResult> =
                            stdout.lines().last().and_then(|line| serde_json::from_str(line).ok());

                        // Captured output is everything except the last JSON line
                        let stdout_lines: Vec<&str> = stdout.lines().collect();
                        let captured_stdout: String = if stdout_lines.len() > 1 {
                            stdout_lines[..stdout_lines.len() - 1].join("\n")
                        } else {
                            String::new()
                        };

                        let captured_output =
                            if captured_stdout.trim().is_empty() && stderr.trim().is_empty() {
                                None
                            } else if captured_stdout.trim().is_empty() {
                                Some(stderr.to_string())
                            } else if stderr.trim().is_empty() {
                                Some(captured_stdout)
                            } else {
                                Some(format!("{}\n{}", captured_stdout, stderr))
                            };

                        match subprocess_result {
                            Some(sr) => {
                                let mut res = TestResult {
                                    name: sr.name,
                                    category: sr.category,
                                    passed: sr.passed,
                                    duration: Duration::from_secs_f64(sr.duration_secs),
                                    error_message: sr.error_message,
                                    is_retry: false,
                                    captured_output,
                                };
                                if is_retry {
                                    res = res.with_retry();
                                }
                                res
                            },
                            None => {
                                let mut res = TestResult {
                                    name: test_name.clone(),
                                    category: test_category.clone(),
                                    passed: false,
                                    duration: Duration::ZERO,
                                    error_message: Some(format!(
                                        "Failed to parse subprocess output: {}",
                                        stdout
                                    )),
                                    is_retry: false,
                                    captured_output: Some(stderr.to_string()),
                                };
                                if is_retry {
                                    res = res.with_retry();
                                }
                                res
                            },
                        }
                    },
                    Err(e) => {
                        let mut res = TestResult {
                            name: test_name.clone(),
                            category: test_category.clone(),
                            passed: false,
                            duration: Duration::ZERO,
                            error_message: Some(format!("Failed to spawn subprocess: {}", e)),
                            is_retry: false,
                            captured_output: None,
                        };
                        if is_retry {
                            res = res.with_retry();
                        }
                        res
                    },
                };

                // Print result
                if is_json {
                    if let Ok(json) = serde_json::to_string(&result) {
                        let _lock = output_mutex.lock().unwrap();
                        println!("{json}");
                    }
                } else {
                    let _lock = output_mutex.lock().unwrap();

                    let status = if result.passed { "PASS" } else { "FAIL" };
                    let retry_marker = if is_retry { " (retry)" } else { "" };
                    let duration_str = format_duration(result.duration);

                    println!(
                        "[{:>3}/{:>3}] {:>4}{}  {:>8}  {}::{}",
                        progress,
                        total_tests,
                        status,
                        retry_marker,
                        duration_str,
                        result.category,
                        result.name,
                    );

                    if !result.passed
                        && let Some(ref error) = result.error_message
                    {
                        println!("            Error: {error}");
                    }
                }

                results.lock().unwrap().push(result);
            }
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}

/// Formats a duration in a human-readable way, similar to nextest.
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs_f64();
    if secs >= 60.0 {
        let mins = (secs / 60.0).floor() as u64;
        let remaining_secs = secs - (mins as f64 * 60.0);
        format!("{}m {:.1}s", mins, remaining_secs)
    } else if secs >= 1.0 {
        format!("{:.2}s", secs)
    } else {
        format!("{}ms", duration.as_millis())
    }
}

/// Prints a comprehensive summary of test execution results.
///
/// Shows pass/fail counts, failed test details, retry statistics, and timing statistics including
/// average, median, min, and max execution times.
fn print_summary(
    initial_results: &[TestResult],
    retry_results: &[TestResult],
    total_duration: Duration,
    output_format: OutputFormat,
) {
    if matches!(output_format, OutputFormat::Json) {
        print_summary_json(initial_results, retry_results, total_duration);
    } else {
        print_summary_human(initial_results, retry_results, total_duration);
    }
}

/// Prints a JSON summary of test results.
fn print_summary_json(
    initial_results: &[TestResult],
    retry_results: &[TestResult],
    total_duration: Duration,
) {
    let initial_passed = initial_results.iter().filter(|r| r.passed).count();
    let initial_failed = initial_results.len() - initial_passed;

    let retry_passed = retry_results.iter().filter(|r| r.passed).count();
    let retry_failed = retry_results.len() - retry_passed;

    // Find flaky tests
    let initial_failed_names: std::collections::HashSet<_> =
        initial_results.iter().filter(|r| !r.passed).map(|r| &r.name).collect();

    let flaky_tests: Vec<_> = retry_results
        .iter()
        .filter(|r| r.passed && initial_failed_names.contains(&r.name))
        .map(|r| r.name.clone())
        .collect();

    let persistent_failures: Vec<_> =
        retry_results.iter().filter(|r| !r.passed).map(|r| r.name.clone()).collect();

    let summary = serde_json::json!({
        "summary": {
            "initial_run": {
                "total": initial_results.len(),
                "passed": initial_passed,
                "failed": initial_failed,
            },
            "retries": {
                "total": retry_results.len(),
                "passed": retry_passed,
                "failed": retry_failed,
            },
            "flaky_tests": flaky_tests,
            "persistent_failures": persistent_failures,
            "total_duration_secs": total_duration.as_secs_f64(),
        }
    });

    println!("{}", serde_json::to_string_pretty(&summary).unwrap_or_default());
}

/// Prints a human-readable summary of test results.
/// Shows captured stdout only for failed tests.
fn print_summary_human(
    initial_results: &[TestResult],
    retry_results: &[TestResult],
    total_duration: Duration,
) {
    let initial_passed = initial_results.iter().filter(|r| r.passed).count();
    let initial_failed = initial_results.len() - initial_passed;

    println!();
    println!("─────────────────────────────────────────────────────────");
    println!("  Summary");
    println!("─────────────────────────────────────────────────────────");

    // Report retry statistics if there were retries
    if !retry_results.is_empty() {
        let retry_passed = retry_results.iter().filter(|r| r.passed).count();

        // Find flaky tests: failed initially but passed on retry
        let initial_failed_names: std::collections::HashSet<_> =
            initial_results.iter().filter(|r| !r.passed).map(|r| &r.name).collect();

        let flaky_tests: Vec<_> = retry_results
            .iter()
            .filter(|r| r.passed && initial_failed_names.contains(&r.name))
            .collect();

        // Persistent failures: failed in initial run and in all retries
        let persistent_failures: Vec<_> = retry_results.iter().filter(|r| !r.passed).collect();

        // Final counts after retries
        let final_passed = initial_passed + retry_passed;
        let final_failed = persistent_failures.len();

        println!(
            "  {} passed, {} failed, {} flaky in {}",
            final_passed,
            final_failed,
            flaky_tests.len(),
            format_duration(total_duration)
        );

        if !flaky_tests.is_empty() {
            println!();
            println!("  Flaky tests (passed on retry):");
            for result in &flaky_tests {
                println!("    {}::{}", result.category, result.name);
            }
        }

        if !persistent_failures.is_empty() {
            println!();
            println!("  Failures:");
            for result in &persistent_failures {
                println!("    FAIL {}::{}", result.category, result.name);
                if let Some(ref error) = result.error_message {
                    println!("         Error: {error}");
                }
                // Show captured output for failed tests
                if let Some(ref output) = result.captured_output {
                    println!("         ─── Captured stdout ───");
                    for line in output.lines() {
                        println!("         {line}");
                    }
                    println!("         ─── End stdout ───");
                }
            }
        }
    } else {
        println!(
            "  {} passed, {} failed in {}",
            initial_passed,
            initial_failed,
            format_duration(total_duration)
        );

        if initial_failed > 0 {
            println!();
            println!("  Failures:");
            for result in initial_results.iter().filter(|r| !r.passed) {
                println!("    FAIL {}::{}", result.category, result.name);
                if let Some(ref error) = result.error_message {
                    println!("         Error: {error}");
                }
                // Show captured output for failed tests
                if let Some(ref output) = result.captured_output {
                    println!("         ─── Captured stdout ───");
                    for line in output.lines() {
                        println!("         {line}");
                    }
                    println!("         ─── End stdout ───");
                }
            }
        }
    }

    // Print timing statistics using all results (initial + retries)
    let all_results: Vec<_> = initial_results.iter().chain(retry_results.iter()).collect();

    if all_results.len() > 1 {
        let mut durations: Vec<_> = all_results.iter().map(|r| r.duration).collect();
        durations.sort();

        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let median_duration = durations[durations.len() / 2];
        let slowest = all_results.iter().max_by_key(|r| r.duration).unwrap();

        println!();
        println!(
            "  Timing: avg {} | median {} | slowest {} ({}::{})",
            format_duration(avg_duration),
            format_duration(median_duration),
            format_duration(slowest.duration),
            slowest.category,
            slowest.name,
        );
    }

    println!("─────────────────────────────────────────────────────────");
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
