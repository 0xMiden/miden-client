use std::sync::{Arc, Mutex};

use futures::{FutureExt, join};

use crate::tests::{
    client::{
        client_builder_initializes_client_with_endpoint, consume_multiple_expected_notes,
        custom_transaction_prover, discarded_transaction, expired_transaction_fails,
        get_account_update, ignore_invalid_notes, import_consumed_note_with_id,
        import_consumed_note_with_proof, import_expected_note_uncommitted, import_expected_notes,
        import_expected_notes_from_the_past_as_committed, import_note_with_proof, locked_account,
        multiple_transactions_can_be_committed_in_different_blocks_without_sync,
        multiple_tx_on_same_block, output_only_note, sync_detail_values, unused_rpc_api,
    },
    custom_transaction::{merkle_store, onchain_notes_sync_with_tag, transaction_request},
    fpi::{fpi_execute_program, nested_fpi_calls, standard_fpi_private, standard_fpi_public},
    network_transaction::{counter_contract_ntx, recall_note_before_ntx_consumes_it},
    onchain::{import_account_by_id, incorrect_genesis, onchain_accounts, onchain_notes_flow},
    swap_transaction::{swap_fully_onchain, swap_private},
};

mod tests;

async fn run_test<F, Fut>(name: &str, test_fn: F, failed_tests: &Arc<Mutex<Vec<String>>>)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    let result = std::panic::AssertUnwindSafe(test_fn()).catch_unwind().await;

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

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    println!("  Starting Miden Client Integration Tests...");
    println!("==============================================");

    let failed_tests = Arc::new(Mutex::new(Vec::new()));

    join!(
        // CLIENT
        // run_test(
        //     "client_builder_initializes_client_with_endpoint",
        //     client_builder_initializes_client_with_endpoint,
        //     &failed_tests,
        // ),
        // run_test("multiple_tx_on_same_block", multiple_tx_on_same_block, &failed_tests),
        // run_test("import_expected_notes", import_expected_notes, &failed_tests),
        // run_test(
        //     "import_expected_note_uncommitted",
        //     import_expected_note_uncommitted,
        //     &failed_tests,
        // ),
        // run_test(
        //     "import_expected_notes_from_the_past_as_committed",
        //     import_expected_notes_from_the_past_as_committed,
        //     &failed_tests,
        // ),
        // run_test("get_account_update", get_account_update, &failed_tests),
        // run_test("sync_detail_values", sync_detail_values, &failed_tests),
        // run_test(
        //     "multiple_transactions_can_be_committed_in_different_blocks_without_sync",
        //     multiple_transactions_can_be_committed_in_different_blocks_without_sync,
        //     &failed_tests,
        // ),
        // run_test(
        //     "consume_multiple_expected_notes",
        //     consume_multiple_expected_notes,
        //     &failed_tests,
        // ),
        // run_test(
        //     "import_consumed_note_with_proof",
        //     import_consumed_note_with_proof,
        //     &failed_tests,
        // ),
        // run_test("import_consumed_note_with_id", import_consumed_note_with_id, &failed_tests,),
        // run_test("import_note_with_proof", import_note_with_proof, &failed_tests),
        // run_test("discarded_transaction", discarded_transaction, &failed_tests),
        // run_test("custom_transaction_prover", custom_transaction_prover, &failed_tests),
        // run_test("locked_account", locked_account, &failed_tests),
        // run_test("expired_transaction_fails", expired_transaction_fails, &failed_tests),
        // run_test("unused_rpc_api", unused_rpc_api, &failed_tests),
        // run_test("ignore_invalid_notes", ignore_invalid_notes, &failed_tests),
        // run_test("output_only_note", output_only_note, &failed_tests),
        // // CUSTOM TRANSACTION
        // run_test("merkle_store", merkle_store, &failed_tests),
        // run_test("onchain_notes_sync_with_tag", onchain_notes_sync_with_tag, &failed_tests,),
        // run_test("transaction_request", transaction_request, &failed_tests),
        // FPI
        run_test("standard_fpi_public", standard_fpi_public, &failed_tests),
        run_test("standard_fpi_private", standard_fpi_private, &failed_tests),
        run_test("fpi_execute_program", fpi_execute_program, &failed_tests),
        run_test("nested_fpi_calls", nested_fpi_calls, &failed_tests),
        // // NETWORK TRANSACTION
        // run_test("counter_contract_ntx", counter_contract_ntx, &failed_tests),
        // run_test(
        //     "recall_note_before_ntx_consumes_it",
        //     recall_note_before_ntx_consumes_it,
        //     &failed_tests,
        // ),
        // // ONCHAIN
        // run_test("import_account_by_id", import_account_by_id, &failed_tests),
        // run_test("onchain_accounts", onchain_accounts, &failed_tests),
        // run_test("onchain_notes_flow", onchain_notes_flow, &failed_tests),
        // run_test("incorrect_genesis", incorrect_genesis, &failed_tests),
        // // SWAP TRANSACTION
        // run_test("swap_fully_onchain", swap_fully_onchain, &failed_tests),
        // run_test("swap_private", swap_private, &failed_tests),
    );
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
