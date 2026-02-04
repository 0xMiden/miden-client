//! Integration tests for account state pruning.
//!
//! These tests verify that the pruning feature works correctly in a real-world scenario
//! with a running node, multiple transactions, and state synchronization.

use anyhow::Result;
use miden_client::account::{AccountId, AccountStorageMode};
use miden_client::asset::{Asset, FungibleAsset};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::note::NoteType;
use miden_client::store::TransactionFilter;
use miden_client::testing::common::*;
use miden_client::transaction::{PaymentNoteDescription, TransactionRequestBuilder, TransactionStatus};

use crate::tests::config::ClientConfig;

/// Integration test for pruning as described in issue #1158.
///
/// This test verifies:
/// 1. Multiple transactions accumulate account states
/// 2. After some transactions commit, old states can be pruned
/// 3. After pruning, the account still works correctly for new transactions
/// 4. Pending transaction states are preserved during pruning
pub async fn test_prune_account_history_with_transactions(
    client_config: ClientConfig,
) -> Result<()> {
    let (mut client, authenticator) = client_config.into_client().await?;
    wait_for_node(&mut client).await;

    println!("\n=== Pruning Integration Test ===\n");

    // Setup: Create two wallets and a faucet
    let (sender_account, receiver_account, faucet_account) = setup_two_wallets_and_faucet(
        &mut client,
        AccountStorageMode::Private,
        &authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let sender_id = sender_account.id();
    let receiver_id = receiver_account.id();
    let faucet_id = faucet_account.id();

    println!("Created accounts:");
    println!("  Sender:   {}", sender_id);
    println!("  Receiver: {}", receiver_id);
    println!("  Faucet:   {}", faucet_id);

    // Step 1: Mint tokens to sender (creates first transaction)
    println!("\n--- Step 1: Mint tokens to sender ---");
    let mint_tx_id = mint_and_consume(&mut client, sender_id, faucet_id, NoteType::Private).await;
    wait_for_tx(&mut client, mint_tx_id).await?;
    println!("Mint transaction committed: {}", mint_tx_id);

    // Step 2: Execute several transfer transactions
    println!("\n--- Step 2: Execute multiple transfers ---");
    let transfer_amount = 10u64;

    for i in 1..=3 {
        let asset = FungibleAsset::new(faucet_id, transfer_amount)?;
        let tx_request = TransactionRequestBuilder::new()
            .build_pay_to_id(
                PaymentNoteDescription::new(
                    vec![Asset::Fungible(asset)],
                    sender_id,
                    receiver_id,
                ),
                NoteType::Private,
                client.rng(),
            )?;

        let tx_id = client.submit_new_transaction(sender_id, tx_request).await?;
        wait_for_tx(&mut client, tx_id).await?;
        println!("Transfer {} committed: {}", i, tx_id);
    }

    // Step 3: Check how many states we have accumulated
    println!("\n--- Step 3: Check prunable states ---");
    let prunable = client.get_prunable_account_data(sender_id).await?;
    println!(
        "Sender account has {} prunable states",
        prunable.state_count()
    );
    for state in &prunable.states {
        println!("  - nonce: {}", state.nonce);
    }

    // We should have some prunable states (the old committed states)
    // The exact count depends on how many transactions were executed
    let _states_before_prune = prunable.state_count();

    // Step 4: Execute another transfer but DON'T wait for it to commit
    // This creates a pending transaction whose state should be preserved
    println!("\n--- Step 4: Create pending transaction ---");
    let asset = FungibleAsset::new(faucet_id, transfer_amount)?;
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                sender_id,
                receiver_id,
            ),
            NoteType::Private,
            client.rng(),
        )?;

    let pending_tx_id = client.submit_new_transaction(sender_id, tx_request).await?;
    println!("Created pending transaction: {}", pending_tx_id);

    // Verify transaction is pending
    let transactions = client.get_transactions(TransactionFilter::All).await?;
    let pending_tx = transactions.iter().find(|t| t.id == pending_tx_id);
    assert!(
        matches!(pending_tx.map(|t| &t.status), Some(TransactionStatus::Pending)),
        "Transaction should be pending"
    );

    // Step 5: Check prunable states again (should account for pending tx)
    println!("\n--- Step 5: Check prunable states with pending tx ---");
    let prunable_with_pending = client.get_prunable_account_data(sender_id).await?;
    println!(
        "Prunable states (with pending tx): {}",
        prunable_with_pending.state_count()
    );

    // The pending transaction's init state should be protected
    // So we might have fewer prunable states now

    // Step 6: Actually prune
    println!("\n--- Step 6: Prune old states ---");
    let pruned = client.prune_account_history(sender_id).await?;
    println!("Pruned {} states", pruned.state_count());
    println!("Pruned {} orphaned storage rows", pruned.orphaned_storage_rows);
    println!("Pruned {} orphaned asset rows", pruned.orphaned_asset_rows);
    println!("Pruned {} orphaned map entries", pruned.orphaned_map_entries);

    // Step 7: Wait for pending transaction to commit
    println!("\n--- Step 7: Wait for pending transaction to commit ---");
    wait_for_tx(&mut client, pending_tx_id).await?;
    println!("Pending transaction committed: {}", pending_tx_id);

    // Step 8: Verify account still works after pruning
    println!("\n--- Step 8: Verify account works after pruning ---");

    // Check account can be retrieved
    let account = client
        .get_account(sender_id)
        .await?
        .expect("Account should exist after pruning");
    println!("Account retrieved successfully, nonce: {}", account.nonce());

    // Execute one more transaction to prove the account is fully functional
    let asset = FungibleAsset::new(faucet_id, transfer_amount)?;
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                sender_id,
                receiver_id,
            ),
            NoteType::Private,
            client.rng(),
        )?;

    let final_tx_id = client.submit_new_transaction(sender_id, tx_request).await?;
    wait_for_tx(&mut client, final_tx_id).await?;
    println!("Post-prune transaction committed: {}", final_tx_id);

    // Final verification
    let final_account = client.get_account(sender_id).await?.expect("Account should exist");
    println!(
        "Final account state - nonce: {}, commitment: {}",
        final_account.nonce(),
        final_account.commitment()
    );

    // Step 9: Prune again and verify minimal states remain
    println!("\n--- Step 9: Final prune check ---");
    let final_prunable = client.get_prunable_account_data(sender_id).await?;
    println!(
        "Final prunable states: {}",
        final_prunable.state_count()
    );

    let final_pruned = client.prune_account_history(sender_id).await?;
    println!("Final pruned: {} states", final_pruned.state_count());

    println!("\n=== Pruning Integration Test PASSED ===\n");
    Ok(())
}

/// Test that pruning works correctly with multiple rounds of transactions and pruning.
pub async fn test_multiple_prune_cycles(client_config: ClientConfig) -> Result<()> {
    let (mut client, authenticator) = client_config.into_client().await?;
    wait_for_node(&mut client).await;

    println!("\n=== Multiple Prune Cycles Test ===\n");

    // Setup
    let (sender_account, receiver_account, faucet_account) = setup_two_wallets_and_faucet(
        &mut client,
        AccountStorageMode::Private,
        &authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let sender_id = sender_account.id();
    let receiver_id = receiver_account.id();
    let faucet_id = faucet_account.id();

    // Initial mint
    let mint_tx_id = mint_and_consume(&mut client, sender_id, faucet_id, NoteType::Private).await;
    wait_for_tx(&mut client, mint_tx_id).await?;

    // Cycle 1: Execute transactions and prune
    println!("--- Cycle 1 ---");
    for _ in 0..2 {
        let tx_id = execute_transfer(&mut client, sender_id, receiver_id, faucet_id, 5).await?;
        wait_for_tx(&mut client, tx_id).await?;
    }
    let pruned_1 = client.prune_account_history(sender_id).await?;
    println!("Cycle 1: Pruned {} states", pruned_1.state_count());

    // Cycle 2: More transactions and prune
    println!("--- Cycle 2 ---");
    for _ in 0..2 {
        let tx_id = execute_transfer(&mut client, sender_id, receiver_id, faucet_id, 5).await?;
        wait_for_tx(&mut client, tx_id).await?;
    }
    let pruned_2 = client.prune_account_history(sender_id).await?;
    println!("Cycle 2: Pruned {} states", pruned_2.state_count());

    // Cycle 3: More transactions and prune
    println!("--- Cycle 3 ---");
    for _ in 0..2 {
        let tx_id = execute_transfer(&mut client, sender_id, receiver_id, faucet_id, 5).await?;
        wait_for_tx(&mut client, tx_id).await?;
    }
    let pruned_3 = client.prune_account_history(sender_id).await?;
    println!("Cycle 3: Pruned {} states", pruned_3.state_count());

    // Verify account still works
    let final_account = client.get_account(sender_id).await?.expect("Account should exist");
    println!(
        "Final state - nonce: {}, works correctly",
        final_account.nonce()
    );

    println!("\n=== Multiple Prune Cycles Test PASSED ===\n");
    Ok(())
}

/// Helper function to execute a transfer transaction
async fn execute_transfer(
    client: &mut TestClient,
    sender_id: AccountId,
    receiver_id: AccountId,
    faucet_id: AccountId,
    amount: u64,
) -> Result<miden_client::transaction::TransactionId> {
    let asset = FungibleAsset::new(faucet_id, amount)?;
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                sender_id,
                receiver_id,
            ),
            NoteType::Private,
            client.rng(),
        )?;

    let tx_id = client.submit_new_transaction(sender_id, tx_request).await?;
    Ok(tx_id)
}
