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

use futures::FutureExt;

mod tests;


async fn run_test<F, Fut>(name: &str, test_fn: F) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    println!(" - {}: RUNNING", name);
    let result = std::panic::AssertUnwindSafe(test_fn())
        .catch_unwind()
        .await;

    match result {
        Ok(_) => {
            Ok(())
        }
        Err(panic_info) => {
            println!(" - {}: FAILED", name);
            let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".into()
            };
            Err(format!("{}: {}", name, msg))
        }
    }
}


#[tokio::main]
async fn main() {
    println!("  Starting Miden Client Integration Tests...");
    println!("==============================================");
    
    let mut failed_tests = Vec::new();

    if let Err(e) = run_test("client_builder_initializes_client_with_endpoint", || {
        client_builder_initializes_client_with_endpoint()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("client_builder_fails_without_keystore", || {
        client_builder_fails_without_keystore()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("multiple_tx_on_same_block", || {
        multiple_tx_on_same_block()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("import_expected_notes", || {
        import_expected_notes()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("import_expected_note_uncommitted", || {
        import_expected_note_uncommitted()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("import_expected_notes_from_the_past_as_committed", || {
        import_expected_notes_from_the_past_as_committed()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("get_account_update", || {
        get_account_update()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("sync_detail_values", || {
        sync_detail_values()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("multiple_transactions_can_be_committed_in_different_blocks_without_sync", || {
        multiple_transactions_can_be_committed_in_different_blocks_without_sync()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("consume_multiple_expected_notes", || {
        consume_multiple_expected_notes()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("import_consumed_note_with_proof", || {
        import_consumed_note_with_proof()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("import_consumed_note_with_id", || {
        import_consumed_note_with_id()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("import_note_with_proof", || {
        import_note_with_proof()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("discarded_transaction", || {
        discarded_transaction()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("custom_transaction_prover", || {
        custom_transaction_prover()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("locked_account", || {
        locked_account()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("expired_transaction_fails", || {
        expired_transaction_fails()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("unused_rpc_api", || {
        unused_rpc_api()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("ignore_invalid_notes", || {
        ignore_invalid_notes()
    }).await {
        failed_tests.push(e);
    }

    // CUSTOM TRANSACTION
    if let Err(e) = run_test("merkle_store", || {
        merkle_store()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("onchain_notes_sync_with_tag", || {
        onchain_notes_sync_with_tag()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("transaction_request", || {
        transaction_request()
    }).await {
        failed_tests.push(e);
    }

    // FPI
    if let Err(e) = run_test("standard_fpi_public", || {
        standard_fpi_public()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("standard_fpi_private", || {
        standard_fpi_private()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("fpi_execute_program", || {
        fpi_execute_program()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("nested_fpi_calls", || {
        nested_fpi_calls()
    }).await {
        failed_tests.push(e);
    }

    // NETWORK TRANSACTION
    if let Err(e) = run_test("counter_contract_ntx", || {
        counter_contract_ntx()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("recall_note_before_ntx_consumes_it", || {
        recall_note_before_ntx_consumes_it()
    }).await {
        failed_tests.push(e);
    }

    // ONCHAIN
    if let Err(e) = run_test("import_account_by_id", || {
        import_account_by_id()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("onchain_accounts", || {
        onchain_accounts()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("onchain_notes_flow", || {
        onchain_notes_flow()
    }).await {
        failed_tests.push(e);
    }

    // SWAP TRANSACTION
    if let Err(e) = run_test("swap_fully_onchain", || {
        swap_fully_onchain()
    }).await {
        failed_tests.push(e);
    }
    
    if let Err(e) = run_test("swap_private", || {
        swap_private()
    }).await {
        failed_tests.push(e);
    }

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
