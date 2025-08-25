use std::sync::{Arc, Mutex};

use anyhow::Result;
use clap::Parser;
use futures::FutureExt;
use miden_client::rpc::Endpoint;
use miden_client::testing::config::ClientConfig;
use url::Url;

use crate::tests::{
    client,
    custom_transaction,
    fpi,
    network_transaction,
    onchain,
    swap_transaction,
};

mod tests;

// MAIN
// ================================================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client_config = args.into_client_config()?;

    run_tests(&client_config).await
}

// ARGS
// ================================================================================================

#[derive(Parser)]
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
}

impl Args {
    fn into_client_config(self) -> Result<ClientConfig> {
        let host = self
            .rpc_endpoint
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid host in RPC endpoint"))?;
        let port = self
            .rpc_endpoint
            .port()
            .ok_or_else(|| anyhow::anyhow!("Invalid port in RPC endpoint"))?;

        let endpoint =
            Endpoint::new(self.rpc_endpoint.scheme().to_string(), host.to_string(), Some(port));
        let timeout_ms = self.timeout;

        Ok(ClientConfig::new(endpoint, timeout_ms))
    }
}

/// Runs a test function and prints the result.
///
/// # Arguments
///
/// * `name` - The name of the test.
/// * `test_fn` - The test function to run.
/// * `failed_tests` - A reference to a vector of failed tests.
/// * `client_config` - The client configuration.
///
/// Works by wrapping the test function in a `std::panic::AssertUnwindSafe` and catching any panics.
/// If the test function panics, the panic is caught and the test is considered failed.
/// If the test function succeeds, the test is considered passed.
///
/// The test function is expected to return a `Future` that resolves when the test is complete.
async fn run_test<F, Fut>(
    name: &str,
    test_fn: F,
    failed_tests: &Arc<Mutex<Vec<String>>>,
    client_config: &ClientConfig,
) where
    F: FnOnce(ClientConfig) -> Fut,
    Fut: Future<Output = Result<()>>,
{
    let result = std::panic::AssertUnwindSafe(test_fn(client_config.clone()))
        .catch_unwind()
        .await;

    match result {
        Ok(Ok(_)) => {
            println!(" - {name}: PASSED");
        },
        Ok(Err(e)) => {
            println!(" - {name}: FAILED");
            let error_report = format_error_report(e);
            failed_tests.lock().unwrap().push(format!("{name}:\n{error_report}"));
        },
        Err(panic_info) => {
            println!(" - {name}: FAILED (panic)");
            let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".into()
            };
            failed_tests.lock().unwrap().push(format!("{name}: {msg}"));
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

/// Runs all the tests sequentially.
///
/// # Arguments
///
/// * `client_config` - The client configuration.
async fn run_tests(client_config: &ClientConfig) -> Result<()> {
    println!("Starting Miden client integration tests");
    println!("==========================================================");
    println!("Using:");
    println!(" - RPC endpoint: {}", client_config.rpc_endpoint);
    println!(" - Timeout: {}ms", client_config.rpc_timeout_ms);
    println!("==========================================================");

    let failed_tests = Arc::new(Mutex::new(Vec::new()));

    // CLIENT
    run_test(
        "client_builder_initializes_client_with_endpoint",
        client::client_builder_initializes_client_with_endpoint,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "multiple_tx_on_same_block",
        client::multiple_tx_on_same_block,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "import_expected_notes",
        client::import_expected_notes,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "import_expected_note_uncommitted",
        client::import_expected_note_uncommitted,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "import_expected_notes_from_the_past_as_committed",
        client::import_expected_notes_from_the_past_as_committed,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("get_account_update", client::get_account_update, &failed_tests, client_config).await;
    run_test("sync_detail_values", client::sync_detail_values, &failed_tests, client_config).await;
    run_test(
        "multiple_transactions_can_be_committed_in_different_blocks_without_sync",
        client::multiple_transactions_can_be_committed_in_different_blocks_without_sync,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "consume_multiple_expected_notes",
        client::consume_multiple_expected_notes,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "import_consumed_note_with_proof",
        client::import_consumed_note_with_proof,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "import_consumed_note_with_id",
        client::import_consumed_note_with_id,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "import_note_with_proof",
        client::import_note_with_proof,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "discarded_transaction",
        client::discarded_transaction,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "custom_transaction_prover",
        client::custom_transaction_prover,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("locked_account", client::locked_account, &failed_tests, client_config).await;
    run_test(
        "expired_transaction_fails",
        client::expired_transaction_fails,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("unused_rpc_api", client::unused_rpc_api, &failed_tests, client_config).await;
    run_test(
        "ignore_invalid_notes",
        client::ignore_invalid_notes,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("output_only_note", client::output_only_note, &failed_tests, client_config).await;
    // CUSTOM TRANSACTION
    run_test("merkle_store", custom_transaction::merkle_store, &failed_tests, client_config).await;
    run_test(
        "onchain_notes_sync_with_tag",
        custom_transaction::onchain_notes_sync_with_tag,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "transaction_request",
        custom_transaction::transaction_request,
        &failed_tests,
        client_config,
    )
    .await;
    // FPI
    run_test("standard_fpi_public", fpi::standard_fpi_public, &failed_tests, client_config).await;
    run_test("standard_fpi_private", fpi::standard_fpi_private, &failed_tests, client_config).await;
    run_test("fpi_execute_program", fpi::fpi_execute_program, &failed_tests, client_config).await;
    run_test("nested_fpi_calls", fpi::nested_fpi_calls, &failed_tests, client_config).await;
    // NETWORK TRANSACTION
    run_test(
        "counter_contract_ntx",
        network_transaction::counter_contract_ntx,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "recall_note_before_ntx_consumes_it",
        network_transaction::recall_note_before_ntx_consumes_it,
        &failed_tests,
        client_config,
    )
    .await;
    // ONCHAIN
    run_test(
        "import_account_by_id",
        onchain::import_account_by_id,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("onchain_accounts", onchain::onchain_accounts, &failed_tests, client_config).await;
    run_test("onchain_notes_flow", onchain::onchain_notes_flow, &failed_tests, client_config).await;
    run_test("incorrect_genesis", onchain::incorrect_genesis, &failed_tests, client_config).await;
    // SWAP TRANSACTION
    run_test(
        "swap_fully_onchain",
        swap_transaction::swap_fully_onchain,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("swap_private", swap_transaction::swap_private, &failed_tests, client_config).await;

    // Print summary
    println!("\n====================== TEST SUMMARY ======================");
    if failed_tests.lock().unwrap().is_empty() {
        println!("All tests passed!");
        Ok(())
    } else {
        let failed = failed_tests.lock().unwrap();
        println!("{} tests failed:", failed.len());
        for (i, failed_test) in failed.iter().enumerate() {
            println!("\n[{}] {}", i + 1, failed_test);
            println!("{}", "─".repeat(80));
        }
        std::process::exit(1);
    }
}
