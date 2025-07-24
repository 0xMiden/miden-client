use futures::FutureExt;

use crate::tests::{
    client::{
        client_builder_fails_without_keystore, client_builder_initializes_client_with_endpoint,
        consume_multiple_expected_notes, custom_transaction_prover, discarded_transaction,
        expired_transaction_fails, get_account_update, ignore_invalid_notes,
        import_consumed_note_with_id, import_consumed_note_with_proof,
        import_expected_note_uncommitted, import_expected_notes,
        import_expected_notes_from_the_past_as_committed, import_note_with_proof, locked_account,
        multiple_transactions_can_be_committed_in_different_blocks_without_sync,
        multiple_tx_on_same_block, sync_detail_values, unused_rpc_api,
    },
    custom_transaction::{merkle_store, onchain_notes_sync_with_tag, transaction_request},
    fpi::{fpi_execute_program, nested_fpi_calls, standard_fpi_private, standard_fpi_public},
    network_transaction::{counter_contract_ntx, recall_note_before_ntx_consumes_it},
    onchain::{import_account_by_id, onchain_accounts, onchain_notes_flow},
    swap_transaction::{swap_fully_onchain, swap_private},
};

mod tests;

async fn run_test<F, Fut>(name: &str, test_fn: F, failed_tests: &mut Vec<String>)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    println!(" - {}: RUNNING", name);
    let result = std::panic::AssertUnwindSafe(test_fn()).catch_unwind().await;

    match result {
        Ok(_) => (),
        Err(panic_info) => {
            println!(" - {}: FAILED", name);
            let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".into()
            };
            failed_tests.push(format!("{}: {}", name, msg));
        },
    }
}

#[tokio::main]
async fn main() {
    println!("  Starting Miden Client Integration Tests...");
    println!("==============================================");

    let mut failed_tests = Vec::new();

    run_test(
        "client_builder_initializes_client_with_endpoint",
        || client_builder_initializes_client_with_endpoint(),
        &mut failed_tests,
    )
    .await;
    run_test(
        "client_builder_fails_without_keystore",
        || client_builder_fails_without_keystore(),
        &mut failed_tests,
    )
    .await;
    run_test("multiple_tx_on_same_block", || multiple_tx_on_same_block(), &mut failed_tests).await;
    run_test("import_expected_notes", || import_expected_notes(), &mut failed_tests).await;
    run_test(
        "import_expected_note_uncommitted",
        || import_expected_note_uncommitted(),
        &mut failed_tests,
    )
    .await;
    run_test(
        "import_expected_notes_from_the_past_as_committed",
        || import_expected_notes_from_the_past_as_committed(),
        &mut failed_tests,
    )
    .await;
    run_test("get_account_update", || get_account_update(), &mut failed_tests).await;
    run_test("sync_detail_values", || sync_detail_values(), &mut failed_tests).await;
    run_test(
        "multiple_transactions_can_be_committed_in_different_blocks_without_sync",
        || multiple_transactions_can_be_committed_in_different_blocks_without_sync(),
        &mut failed_tests,
    )
    .await;
    run_test(
        "consume_multiple_expected_notes",
        || consume_multiple_expected_notes(),
        &mut failed_tests,
    )
    .await;
    run_test(
        "import_consumed_note_with_proof",
        || import_consumed_note_with_proof(),
        &mut failed_tests,
    )
    .await;
    run_test(
        "import_consumed_note_with_id",
        || import_consumed_note_with_id(),
        &mut failed_tests,
    )
    .await;
    run_test("import_note_with_proof", || import_note_with_proof(), &mut failed_tests).await;
    run_test("discarded_transaction", || discarded_transaction(), &mut failed_tests).await;
    run_test("custom_transaction_prover", || custom_transaction_prover(), &mut failed_tests).await;
    run_test("locked_account", || locked_account(), &mut failed_tests).await;
    run_test("expired_transaction_fails", || expired_transaction_fails(), &mut failed_tests).await;
    run_test("unused_rpc_api", || unused_rpc_api(), &mut failed_tests).await;
    run_test("ignore_invalid_notes", || ignore_invalid_notes(), &mut failed_tests).await;

    // CUSTOM TRANSACTION
    run_test("merkle_store", || merkle_store(), &mut failed_tests).await;
    run_test(
        "onchain_notes_sync_with_tag",
        || onchain_notes_sync_with_tag(),
        &mut failed_tests,
    )
    .await;
    run_test("transaction_request", || transaction_request(), &mut failed_tests).await;

    // FPI
    run_test("standard_fpi_public", || standard_fpi_public(), &mut failed_tests).await;
    run_test("standard_fpi_private", || standard_fpi_private(), &mut failed_tests).await;
    run_test("fpi_execute_program", || fpi_execute_program(), &mut failed_tests).await;
    run_test("nested_fpi_calls", || nested_fpi_calls(), &mut failed_tests).await;

    // NETWORK TRANSACTION
    run_test("counter_contract_ntx", || counter_contract_ntx(), &mut failed_tests).await;
    run_test(
        "recall_note_before_ntx_consumes_it",
        || recall_note_before_ntx_consumes_it(),
        &mut failed_tests,
    )
    .await;

    // ONCHAIN
    run_test("import_account_by_id", || import_account_by_id(), &mut failed_tests).await;
    run_test("onchain_accounts", || onchain_accounts(), &mut failed_tests).await;
    run_test("onchain_notes_flow", || onchain_notes_flow(), &mut failed_tests).await;

    // SWAP TRANSACTION
    run_test("swap_fully_onchain", || swap_fully_onchain(), &mut failed_tests).await;
    run_test("swap_private", || swap_private(), &mut failed_tests).await;

    // Print summary
    println!("\n=== TEST SUMMARY ===");
    if failed_tests.is_empty() {
        println!("All tests passed!");
    } else {
        println!("{} tests failed:", failed_tests.len());
        for failed_test in &failed_tests {
            println!("  - {}", failed_test);
        }
        std::process::exit(1);
    }
}
