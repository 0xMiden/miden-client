use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::Parser;
use futures::FutureExt;
use miden_client::rpc::Endpoint;
use miden_client::testing::config::{ClientConfig, create_test_auth_path, create_test_store_path};
use url::Url;

use crate::tests::client::*;
use crate::tests::custom_transaction::*;
use crate::tests::fpi::*;
use crate::tests::network_transaction::*;
use crate::tests::onchain::*;
use crate::tests::swap_transaction::*;

mod tests;

#[derive(Parser)]
#[command(
    name = "miden-client-integration-tests",
    about = "Integration tests for the Miden client library",
    version
)]
struct Args {
    /// The URL of the RPC endpoint to use.
    #[arg(short, long, default_value = "http://localhost:57291")]
    rpc_endpoint: Url,

    /// The path to the store directory, it will use a temporary directory if not provided.
    #[arg(short, long)]
    store_path: Option<PathBuf>,

    /// The path to the keystore directory, it will use a temporary directory if not provided.
    #[arg(short, long)]
    keystore_path: Option<PathBuf>,

    /// Timeout for the RPC requests in milliseconds.
    #[arg(short, long, default_value = "10000")]
    timeout: u64,
}

impl From<Args> for ClientConfig {
    fn from(args: Args) -> Self {
        let endpoint = Endpoint::new(
            args.rpc_endpoint.scheme().to_string(),
            args.rpc_endpoint.host_str().unwrap().to_string(),
            Some(args.rpc_endpoint.port().unwrap()),
        );
        let timeout_ms = args.timeout;
        let store_path = args.store_path.unwrap_or_else(create_test_store_path);
        let auth_path = args.keystore_path.unwrap_or_else(create_test_auth_path);

        ClientConfig::new(endpoint, timeout_ms, store_path, auth_path)
    }
}

async fn run_test<F, Fut>(
    name: &str,
    test_fn: F,
    failed_tests: &Arc<Mutex<Vec<String>>>,
    client_config: &ClientConfig,
) where
    F: FnOnce(ClientConfig) -> Fut,
    Fut: Future<Output = ()>,
{
    let result = std::panic::AssertUnwindSafe(test_fn(client_config.clone()))
        .catch_unwind()
        .await;

    match result {
        Ok(_) => {
            println!(" - {name}: PASSED");
        },
        Err(panic_info) => {
            println!(" - {name}: FAILED");
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

async fn run_tests(client_config: &ClientConfig) {
    println!("  Starting Miden Client Integration Tests...");
    println!("==============================================");

    let failed_tests = Arc::new(Mutex::new(Vec::new()));

    // CLIENT
    run_test(
        "client_builder_initializes_client_with_endpoint",
        client_builder_initializes_client_with_endpoint,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "multiple_tx_on_same_block",
        multiple_tx_on_same_block,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("import_expected_notes", import_expected_notes, &failed_tests, client_config).await;
    run_test(
        "import_expected_note_uncommitted",
        import_expected_note_uncommitted,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "import_expected_notes_from_the_past_as_committed",
        import_expected_notes_from_the_past_as_committed,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("get_account_update", get_account_update, &failed_tests, client_config).await;
    run_test("sync_detail_values", sync_detail_values, &failed_tests, client_config).await;
    run_test(
        "multiple_transactions_can_be_committed_in_different_blocks_without_sync",
        multiple_transactions_can_be_committed_in_different_blocks_without_sync,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "consume_multiple_expected_notes",
        consume_multiple_expected_notes,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "import_consumed_note_with_proof",
        import_consumed_note_with_proof,
        &failed_tests,
        client_config,
    )
    .await;
    run_test(
        "import_consumed_note_with_id",
        import_consumed_note_with_id,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("import_note_with_proof", import_note_with_proof, &failed_tests, client_config).await;
    run_test("discarded_transaction", discarded_transaction, &failed_tests, client_config).await;
    run_test(
        "custom_transaction_prover",
        custom_transaction_prover,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("locked_account", locked_account, &failed_tests, client_config).await;
    run_test(
        "expired_transaction_fails",
        expired_transaction_fails,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("unused_rpc_api", unused_rpc_api, &failed_tests, client_config).await;
    run_test("ignore_invalid_notes", ignore_invalid_notes, &failed_tests, client_config).await;
    run_test("output_only_note", output_only_note, &failed_tests, client_config).await;
    // CUSTOM TRANSACTION
    run_test("merkle_store", merkle_store, &failed_tests, client_config).await;
    run_test(
        "onchain_notes_sync_with_tag",
        onchain_notes_sync_with_tag,
        &failed_tests,
        client_config,
    )
    .await;
    run_test("transaction_request", transaction_request, &failed_tests, client_config).await;
    // FPI
    run_test("standard_fpi_public", standard_fpi_public, &failed_tests, client_config).await;
    run_test("standard_fpi_private", standard_fpi_private, &failed_tests, client_config).await;
    run_test("fpi_execute_program", fpi_execute_program, &failed_tests, client_config).await;
    run_test("nested_fpi_calls", nested_fpi_calls, &failed_tests, client_config).await;
    // NETWORK TRANSACTION
    run_test("counter_contract_ntx", counter_contract_ntx, &failed_tests, client_config).await;
    run_test(
        "recall_note_before_ntx_consumes_it",
        recall_note_before_ntx_consumes_it,
        &failed_tests,
        client_config,
    )
    .await;
    // ONCHAIN
    run_test("import_account_by_id", import_account_by_id, &failed_tests, client_config).await;
    run_test("onchain_accounts", onchain_accounts, &failed_tests, client_config).await;
    run_test("onchain_notes_flow", onchain_notes_flow, &failed_tests, client_config).await;
    run_test("incorrect_genesis", incorrect_genesis, &failed_tests, client_config).await;
    // SWAP TRANSACTION
    run_test("swap_fully_onchain", swap_fully_onchain, &failed_tests, client_config).await;
    run_test("swap_private", swap_private, &failed_tests, client_config).await;

    // Print summary
    println!("\n=== TEST SUMMARY ===");
    if failed_tests.lock().unwrap().is_empty() {
        println!("All tests passed!");
    } else {
        println!("{} tests failed:", failed_tests.lock().unwrap().len());
        for failed_test in failed_tests.lock().unwrap().iter() {
            println!("  - {failed_test}");
        }
        std::process::exit(1);
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    run_tests(&args.into()).await;
}
