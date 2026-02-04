//! Tests for measuring database growth and validating pruning functionality.
//!
//! This module provides tests to:
//! 1. Measure how the database grows as account states accumulate
//! 2. Validate that pruning correctly removes old states while preserving necessary data

use std::collections::BTreeMap;

use miden_client::account::component::{AccountComponent, basic_wallet_library};
use miden_client::account::{
    Account, AccountBuilder, AccountDelta, AccountHeader, AccountId, AccountType, Address,
    StorageSlot, StorageSlotName,
};
use miden_client::asset::{AccountStorageDelta, AccountVaultDelta};
use miden_client::auth::{AuthFalcon512Rpo, PublicKeyCommitment};
use miden_client::store::Store;
use miden_client::{EMPTY_WORD, Felt, ONE};
use rusqlite::Connection;

use crate::SqliteStore;
use crate::sql_error::SqlResultExt;
use crate::tests::create_test_store;

/// Metrics about the database state.
#[derive(Debug, Clone)]
pub struct DbMetrics {
    /// Total database size in bytes (page_count * page_size)
    pub total_size_bytes: i64,
    /// Total row count in accounts table
    pub accounts_count: i64,
    /// Total row count in account_storage table
    pub storage_count: i64,
    /// Total row count in account_assets table
    pub assets_count: i64,
    /// Total row count in storage_map_entries table
    pub map_entries_count: i64,
}

impl DbMetrics {
    /// Query current database metrics.
    pub fn query(conn: &Connection) -> Result<Self, rusqlite::Error> {
        let total_size_bytes: i64 = conn.query_row(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
            [],
            |row| row.get(0),
        )?;

        let accounts_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;

        let storage_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM account_storage", [], |row| row.get(0))?;

        let assets_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM account_assets", [], |row| row.get(0))?;

        let map_entries_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM storage_map_entries", [], |row| row.get(0))?;

        Ok(Self {
            total_size_bytes,
            accounts_count,
            storage_count,
            assets_count,
            map_entries_count,
        })
    }
}

/// Metrics specific to a single account.
#[derive(Debug, Clone)]
pub struct AccountMetrics {
    /// Total number of states stored for this account
    pub state_count: i64,
    /// Highest nonce
    pub latest_nonce: i64,
    /// Number of committed states (account_seed IS NULL)
    pub committed_count: i64,
    /// Number of pending states (account_seed IS NOT NULL)
    pub pending_count: i64,
}

impl AccountMetrics {
    /// Query metrics for a specific account.
    pub fn query(conn: &Connection, account_id: AccountId) -> Result<Self, rusqlite::Error> {
        let id_hex = account_id.to_hex();

        let state_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM accounts WHERE id = ?", [&id_hex], |row| {
                row.get(0)
            })?;

        let latest_nonce: i64 = conn.query_row(
            "SELECT COALESCE(MAX(nonce), 0) FROM accounts WHERE id = ?",
            [&id_hex],
            |row| row.get(0),
        )?;

        let committed_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM accounts WHERE id = ? AND account_seed IS NULL",
            [&id_hex],
            |row| row.get(0),
        )?;

        let pending_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM accounts WHERE id = ? AND account_seed IS NOT NULL",
            [&id_hex],
            |row| row.get(0),
        )?;

        Ok(Self { state_count, latest_nonce, committed_count, pending_count })
    }
}

/// Result of a single measurement point during the test.
#[derive(Debug, Clone)]
pub struct MeasurementPoint {
    pub transaction_number: usize,
    pub db_metrics: DbMetrics,
    pub account_metrics: AccountMetrics,
}

/// Measures database metrics at the current point.
async fn measure(
    store: &SqliteStore,
    account_id: AccountId,
    transaction_number: usize,
) -> anyhow::Result<MeasurementPoint> {
    store
        .interact_with_connection(move |conn| {
            let db_metrics = DbMetrics::query(conn)
                .map_err(|e| miden_client::store::StoreError::DatabaseError(e.to_string()))?;
            let account_metrics = AccountMetrics::query(conn, account_id)
                .map_err(|e| miden_client::store::StoreError::DatabaseError(e.to_string()))?;
            Ok(MeasurementPoint { transaction_number, db_metrics, account_metrics })
        })
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Print a report of all measurements.
fn print_measurement_report(measurements: &[MeasurementPoint]) {
    println!("\n=== Database Growth Measurement Report ===\n");
    println!(
        "{:>5} | {:>12} | {:>10} | {:>10} | {:>10} | {:>10} | {:>6} | {:>6}",
        "Tx#", "DB Size (B)", "Accounts", "Storage", "Assets", "MapEntries", "States", "Nonce"
    );
    println!("{}", "-".repeat(90));

    for m in measurements {
        println!(
            "{:>5} | {:>12} | {:>10} | {:>10} | {:>10} | {:>10} | {:>6} | {:>6}",
            m.transaction_number,
            m.db_metrics.total_size_bytes,
            m.db_metrics.accounts_count,
            m.db_metrics.storage_count,
            m.db_metrics.assets_count,
            m.db_metrics.map_entries_count,
            m.account_metrics.state_count,
            m.account_metrics.latest_nonce
        );
    }

    // Calculate growth rates
    if measurements.len() >= 2 {
        let first = &measurements[0];
        let last = measurements.last().unwrap();
        let tx_count = last.transaction_number - first.transaction_number;

        if tx_count > 0 {
            println!("\n=== Growth Analysis ===\n");
            println!("Total transactions: {tx_count}");
            println!(
                "DB size growth: {} -> {} bytes ({} bytes/tx)",
                first.db_metrics.total_size_bytes,
                last.db_metrics.total_size_bytes,
                (last.db_metrics.total_size_bytes - first.db_metrics.total_size_bytes)
                    / tx_count as i64
            );
            println!(
                "Account states: {} -> {} ({:.2} states/tx)",
                first.account_metrics.state_count,
                last.account_metrics.state_count,
                (last.account_metrics.state_count - first.account_metrics.state_count) as f64
                    / tx_count as f64
            );
            println!(
                "Storage rows: {} -> {} ({:.2} rows/tx)",
                first.db_metrics.storage_count,
                last.db_metrics.storage_count,
                (last.db_metrics.storage_count - first.db_metrics.storage_count) as f64
                    / tx_count as f64
            );
            println!(
                "Asset rows: {} -> {} ({:.2} rows/tx)",
                first.db_metrics.assets_count,
                last.db_metrics.assets_count,
                (last.db_metrics.assets_count - first.db_metrics.assets_count) as f64
                    / tx_count as f64
            );
        }
    }
}

/// Test that measures database growth over multiple transactions using account deltas.
///
/// This test establishes a baseline for how the database grows without pruning.
/// Each transaction applies a delta to the account, which creates a new account state
/// in the database.
///
/// Run this BEFORE implementing pruning to see the problem, then AFTER to verify the fix.
#[tokio::test]
async fn measure_database_growth_without_pruning() -> anyhow::Result<()> {
    let store = create_test_store().await;
    let mut measurements = Vec::new();

    let value_slot_name =
        StorageSlotName::new("miden::testing::pruning::value").expect("valid slot name");

    // Create account with a value slot
    let dummy_component = AccountComponent::new(
        basic_wallet_library(),
        vec![StorageSlot::with_empty_value(value_slot_name.clone())],
    )?
    .with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build()?;

    let account_id = account.id();
    let default_address = Address::new(account_id);
    store.insert_account(&account, default_address).await?;

    // Measure initial state
    measurements.push(measure(&store, account_id, 0).await?);

    // Simulate N transactions using deltas (this is how real transactions work)
    const NUM_TRANSACTIONS: usize = 10;
    let mut current_account = account;

    for i in 1..=NUM_TRANSACTIONS {
        // Create a storage delta that changes the value slot
        let mut storage_delta = AccountStorageDelta::new();
        let new_value = [
            Felt::new(i as u64),
            Felt::new(i as u64 + 1),
            Felt::new(i as u64 + 2),
            Felt::new(i as u64 + 3),
        ];
        storage_delta.set_item(value_slot_name.clone(), new_value.into())?;

        // Empty vault delta - we only test storage changes to keep things simple
        let vault_delta = AccountVaultDelta::default();

        // Create the account delta
        let delta = AccountDelta::new(current_account.id(), storage_delta, vault_delta, ONE)?;

        // Get the initial header before applying delta
        let init_header: AccountHeader = (&current_account).into();

        // Apply delta to account in memory
        current_account.apply_delta(&delta)?;

        // Get the final header after applying delta
        let final_header: AccountHeader = (&current_account).into();

        // Apply to store using apply_account_delta
        let smt_forest = store.smt_forest.clone();
        let delta_clone = delta.clone();

        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().into_store_error()?;
                let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");

                SqliteStore::apply_account_delta(
                    &tx,
                    &mut smt_forest,
                    &init_header,
                    &final_header,
                    BTreeMap::default(), // No fungible asset updates needed for additions
                    BTreeMap::default(), // No storage map updates needed
                    &delta_clone,
                )?;

                tx.commit().into_store_error()?;
                Ok(())
            })
            .await?;

        // Measure after transaction
        measurements.push(measure(&store, account_id, i).await?);
    }

    // Print report
    print_measurement_report(&measurements);

    // Assertions to validate growth behavior
    let last = measurements.last().unwrap();

    // Each transaction should add exactly 1 account state
    assert_eq!(
        last.account_metrics.state_count,
        (NUM_TRANSACTIONS + 1) as i64, // +1 for initial state
        "Expected {} account states after {} transactions",
        NUM_TRANSACTIONS + 1,
        NUM_TRANSACTIONS
    );

    // The nonce should match the transaction count
    assert_eq!(
        last.account_metrics.latest_nonce,
        NUM_TRANSACTIONS as i64,
        "Expected nonce {} after {} transactions",
        NUM_TRANSACTIONS,
        NUM_TRANSACTIONS
    );

    // All states except the first should be committed (no account_seed)
    assert_eq!(
        last.account_metrics.committed_count,
        NUM_TRANSACTIONS as i64, // Deltas create committed states (no seed)
        "Expected {} committed states",
        NUM_TRANSACTIONS
    );

    // First state has account_seed (pending)
    assert_eq!(
        last.account_metrics.pending_count,
        1, // Only the initial state has a seed
        "Expected 1 pending state (initial)"
    );

    println!("\n[SUCCESS] Database growth measurement complete.");
    println!(
        "Without pruning, {} transactions created {} account states.",
        NUM_TRANSACTIONS, last.account_metrics.state_count
    );
    println!("Storage rows accumulated: {}", last.db_metrics.storage_count);
    println!("Asset rows accumulated: {}", last.db_metrics.assets_count);

    Ok(())
}

/// Test that verifies pruning correctly removes old states.
///
/// This test verifies that:
/// 1. Old committed states are removed
/// 2. Latest committed state is preserved
/// 3. Pending states are preserved (if any)
/// 4. Orphaned data in related tables is cleaned up
#[tokio::test]
async fn verify_pruning_removes_old_states() -> anyhow::Result<()> {
    let store = create_test_store().await;

    let value_slot_name =
        StorageSlotName::new("miden::testing::pruning::value").expect("valid slot name");

    // Create account
    let dummy_component = AccountComponent::new(
        basic_wallet_library(),
        vec![StorageSlot::with_empty_value(value_slot_name.clone())],
    )?
    .with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build()?;

    let account_id = account.id();
    let default_address = Address::new(account_id);
    store.insert_account(&account, default_address).await?;

    // Apply several transactions
    const NUM_TRANSACTIONS: usize = 10;
    let mut current_account = account;

    for i in 1..=NUM_TRANSACTIONS {
        let mut storage_delta = AccountStorageDelta::new();
        let new_value = [
            Felt::new(i as u64),
            Felt::new(i as u64 + 1),
            Felt::new(i as u64 + 2),
            Felt::new(i as u64 + 3),
        ];
        storage_delta.set_item(value_slot_name.clone(), new_value.into())?;

        let vault_delta = AccountVaultDelta::default();
        let delta = AccountDelta::new(current_account.id(), storage_delta, vault_delta, ONE)?;

        let init_header: AccountHeader = (&current_account).into();
        current_account.apply_delta(&delta)?;
        let final_header: AccountHeader = (&current_account).into();

        let smt_forest = store.smt_forest.clone();
        let delta_clone = delta.clone();

        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().into_store_error()?;
                let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");

                SqliteStore::apply_account_delta(
                    &tx,
                    &mut smt_forest,
                    &init_header,
                    &final_header,
                    BTreeMap::default(),
                    BTreeMap::default(),
                    &delta_clone,
                )?;

                tx.commit().into_store_error()?;
                Ok(())
            })
            .await?;
    }

    // Measure before pruning
    let before_prune = measure(&store, account_id, NUM_TRANSACTIONS).await?;
    println!("\n=== Before Pruning ===");
    println!("Account states: {}", before_prune.account_metrics.state_count);
    println!("Storage rows: {}", before_prune.db_metrics.storage_count);
    println!("Asset rows: {}", before_prune.db_metrics.assets_count);

    // Call pruning
    let prune_result = store.prune_account_history(account_id).await?;
    println!("\n=== Pruning Result ===");
    println!("States pruned: {}", prune_result.state_count());
    println!("Orphaned storage rows deleted: {}", prune_result.orphaned_storage_rows);
    println!("Orphaned asset rows deleted: {}", prune_result.orphaned_asset_rows);

    // Measure after pruning
    let after_prune = measure(&store, account_id, NUM_TRANSACTIONS).await?;

    println!("\n=== After Pruning ===");
    println!("Account states: {}", after_prune.account_metrics.state_count);
    println!("Storage rows: {}", after_prune.db_metrics.storage_count);
    println!("Asset rows: {}", after_prune.db_metrics.assets_count);

    // Verify pruning worked
    // Should have only the latest committed state + initial pending state
    assert!(
        after_prune.account_metrics.state_count <= 2,
        "Expected at most 2 states after pruning, got {}",
        after_prune.account_metrics.state_count
    );

    // Verify storage and asset rows were cleaned up
    assert!(
        after_prune.db_metrics.storage_count < before_prune.db_metrics.storage_count,
        "Storage rows should be reduced after pruning"
    );

    // Verify latest state is still accessible
    let latest_account = store.get_account(account_id).await?;
    assert!(latest_account.is_some(), "Latest account state should still be accessible");

    // Verify the account has the correct nonce (latest transaction)
    let account_record = latest_account.unwrap();
    let account: Account = account_record.try_into()?;
    assert_eq!(
        account.nonce().as_int(),
        NUM_TRANSACTIONS as u64,
        "Account should have nonce from latest transaction"
    );

    Ok(())
}

/// Test multiple rounds of transactions followed by pruning.
///
/// This simulates a realistic usage pattern where:
/// 1. User performs several transactions
/// 2. User prunes old states
/// 3. User performs more transactions
/// 4. User prunes again
///
/// Verifies that the account remains fully functional throughout.
#[tokio::test]
async fn multiple_rounds_of_transactions_and_pruning() -> anyhow::Result<()> {
    let store = create_test_store().await;

    let value_slot_name =
        StorageSlotName::new("miden::testing::pruning::value").expect("valid slot name");

    // Create account
    let dummy_component = AccountComponent::new(
        basic_wallet_library(),
        vec![StorageSlot::with_empty_value(value_slot_name.clone())],
    )?
    .with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build()?;

    let account_id = account.id();
    let default_address = Address::new(account_id);
    store.insert_account(&account, default_address).await?;

    let mut current_account = account;
    let mut total_transactions = 0;

    println!("\n=== Multiple Rounds Test ===\n");

    // =========================================================================
    // ROUND 1: 5 transactions, then prune
    // =========================================================================
    println!("--- Round 1: 5 transactions ---");

    for i in 1..=5 {
        let mut storage_delta = AccountStorageDelta::new();
        let new_value = [
            Felt::new(i as u64),
            Felt::new(i as u64 + 100),
            Felt::new(i as u64 + 200),
            Felt::new(i as u64 + 300),
        ];
        storage_delta.set_item(value_slot_name.clone(), new_value.into())?;

        let vault_delta = AccountVaultDelta::default();
        let delta = AccountDelta::new(current_account.id(), storage_delta, vault_delta, ONE)?;

        let init_header: AccountHeader = (&current_account).into();
        current_account.apply_delta(&delta)?;
        let final_header: AccountHeader = (&current_account).into();

        let smt_forest = store.smt_forest.clone();
        let delta_clone = delta.clone();

        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().into_store_error()?;
                let mut smt_forest =
                    smt_forest.write().expect("smt_forest write lock not poisoned");

                SqliteStore::apply_account_delta(
                    &tx,
                    &mut smt_forest,
                    &init_header,
                    &final_header,
                    BTreeMap::default(),
                    BTreeMap::default(),
                    &delta_clone,
                )?;

                tx.commit().into_store_error()?;
                Ok(())
            })
            .await?;

        total_transactions += 1;
    }

    let before_prune_1 = measure(&store, account_id, total_transactions).await?;
    println!(
        "Before prune 1: {} states, {} storage rows",
        before_prune_1.account_metrics.state_count, before_prune_1.db_metrics.storage_count
    );

    // Prune round 1
    let prune_result_1 = store.prune_account_history(account_id).await?;
    println!(
        "Prune 1: {} states deleted, {} storage rows cleaned",
        prune_result_1.state_count(),
        prune_result_1.orphaned_storage_rows
    );

    let after_prune_1 = measure(&store, account_id, total_transactions).await?;
    println!(
        "After prune 1: {} states, {} storage rows",
        after_prune_1.account_metrics.state_count, after_prune_1.db_metrics.storage_count
    );

    // Verify account still works
    let account_record = store.get_account(account_id).await?.expect("account should exist");
    assert_eq!(
        account_record.nonce().as_int(),
        5,
        "Account should have nonce 5 after round 1"
    );

    // =========================================================================
    // ROUND 2: 3 more transactions, then prune
    // =========================================================================
    println!("\n--- Round 2: 3 more transactions ---");

    for i in 6..=8 {
        let mut storage_delta = AccountStorageDelta::new();
        let new_value = [
            Felt::new(i as u64),
            Felt::new(i as u64 + 100),
            Felt::new(i as u64 + 200),
            Felt::new(i as u64 + 300),
        ];
        storage_delta.set_item(value_slot_name.clone(), new_value.into())?;

        let vault_delta = AccountVaultDelta::default();
        let delta = AccountDelta::new(current_account.id(), storage_delta, vault_delta, ONE)?;

        let init_header: AccountHeader = (&current_account).into();
        current_account.apply_delta(&delta)?;
        let final_header: AccountHeader = (&current_account).into();

        let smt_forest = store.smt_forest.clone();
        let delta_clone = delta.clone();

        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().into_store_error()?;
                let mut smt_forest =
                    smt_forest.write().expect("smt_forest write lock not poisoned");

                SqliteStore::apply_account_delta(
                    &tx,
                    &mut smt_forest,
                    &init_header,
                    &final_header,
                    BTreeMap::default(),
                    BTreeMap::default(),
                    &delta_clone,
                )?;

                tx.commit().into_store_error()?;
                Ok(())
            })
            .await?;

        total_transactions += 1;
    }

    let before_prune_2 = measure(&store, account_id, total_transactions).await?;
    println!(
        "Before prune 2: {} states, {} storage rows",
        before_prune_2.account_metrics.state_count, before_prune_2.db_metrics.storage_count
    );

    // Prune round 2
    let prune_result_2 = store.prune_account_history(account_id).await?;
    println!(
        "Prune 2: {} states deleted, {} storage rows cleaned",
        prune_result_2.state_count(),
        prune_result_2.orphaned_storage_rows
    );

    let after_prune_2 = measure(&store, account_id, total_transactions).await?;
    println!(
        "After prune 2: {} states, {} storage rows",
        after_prune_2.account_metrics.state_count, after_prune_2.db_metrics.storage_count
    );

    // Verify account still works
    let account_record = store.get_account(account_id).await?.expect("account should exist");
    assert_eq!(
        account_record.nonce().as_int(),
        8,
        "Account should have nonce 8 after round 2"
    );

    // =========================================================================
    // ROUND 3: 2 more transactions, then prune
    // =========================================================================
    println!("\n--- Round 3: 2 more transactions ---");

    for i in 9..=10 {
        let mut storage_delta = AccountStorageDelta::new();
        let new_value = [
            Felt::new(i as u64),
            Felt::new(i as u64 + 100),
            Felt::new(i as u64 + 200),
            Felt::new(i as u64 + 300),
        ];
        storage_delta.set_item(value_slot_name.clone(), new_value.into())?;

        let vault_delta = AccountVaultDelta::default();
        let delta = AccountDelta::new(current_account.id(), storage_delta, vault_delta, ONE)?;

        let init_header: AccountHeader = (&current_account).into();
        current_account.apply_delta(&delta)?;
        let final_header: AccountHeader = (&current_account).into();

        let smt_forest = store.smt_forest.clone();
        let delta_clone = delta.clone();

        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().into_store_error()?;
                let mut smt_forest =
                    smt_forest.write().expect("smt_forest write lock not poisoned");

                SqliteStore::apply_account_delta(
                    &tx,
                    &mut smt_forest,
                    &init_header,
                    &final_header,
                    BTreeMap::default(),
                    BTreeMap::default(),
                    &delta_clone,
                )?;

                tx.commit().into_store_error()?;
                Ok(())
            })
            .await?;

        total_transactions += 1;
    }

    let before_prune_3 = measure(&store, account_id, total_transactions).await?;
    println!(
        "Before prune 3: {} states, {} storage rows",
        before_prune_3.account_metrics.state_count, before_prune_3.db_metrics.storage_count
    );

    // Prune round 3
    let prune_result_3 = store.prune_account_history(account_id).await?;
    println!(
        "Prune 3: {} states deleted, {} storage rows cleaned",
        prune_result_3.state_count(),
        prune_result_3.orphaned_storage_rows
    );

    let after_prune_3 = measure(&store, account_id, total_transactions).await?;
    println!(
        "After prune 3: {} states, {} storage rows",
        after_prune_3.account_metrics.state_count, after_prune_3.db_metrics.storage_count
    );

    // =========================================================================
    // Final verification
    // =========================================================================
    println!("\n--- Final Verification ---");

    // Verify final state
    let final_account_record = store.get_account(account_id).await?.expect("account should exist");
    let final_account: Account = final_account_record.try_into()?;

    assert_eq!(
        final_account.nonce().as_int(),
        10,
        "Final account should have nonce 10"
    );

    // Verify we always end up with 2 states (1 pending initial + 1 latest committed)
    assert_eq!(
        after_prune_3.account_metrics.state_count,
        2,
        "Should always have exactly 2 states after pruning"
    );

    // Verify storage is minimal
    assert_eq!(
        after_prune_3.db_metrics.storage_count,
        4,
        "Should have 4 storage rows (2 per state)"
    );

    // Verify the storage value is from the last transaction
    let storage_value = store
        .get_account_storage_item(account_id, value_slot_name)
        .await?;
    let expected_value = [
        Felt::new(10),
        Felt::new(110),
        Felt::new(210),
        Felt::new(310),
    ];
    assert_eq!(
        storage_value,
        expected_value.into(),
        "Storage should have value from last transaction"
    );

    println!("\n[SUCCESS] Multiple rounds test passed!");
    println!("Total transactions: {}", total_transactions);
    println!("Final state count: {}", after_prune_3.account_metrics.state_count);
    println!("Final storage rows: {}", after_prune_3.db_metrics.storage_count);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn db_metrics_query_works() -> anyhow::Result<()> {
        let store = create_test_store().await;

        let metrics = store
            .interact_with_connection(|conn| {
                DbMetrics::query(conn)
                    .map_err(|e| miden_client::store::StoreError::DatabaseError(e.to_string()))
            })
            .await?;

        // Fresh database should have minimal size
        assert!(metrics.total_size_bytes > 0, "Database should have some size");
        assert_eq!(metrics.accounts_count, 0, "Fresh database should have no accounts");

        Ok(())
    }

    /// Test that states corresponding to pending transactions are NOT prunable.
    /// SAFETY: States from pending transactions may be committed on-chain later
    /// and must NEVER be pruned.
    #[tokio::test]
    async fn prunable_data_excludes_pending_transaction_states() -> anyhow::Result<()> {
        use miden_client::transaction::{
            TransactionDetails, TransactionId, TransactionRecord, TransactionStatus,
        };
        use miden_protocol::block::BlockNumber;
        use miden_protocol::transaction::OutputNotes;

        use crate::transaction::upsert_transaction_record;

        let store = create_test_store().await;

        let value_slot_name =
            StorageSlotName::new("miden::testing::pruning::pending").expect("valid slot name");

        let dummy_component = AccountComponent::new(
            basic_wallet_library(),
            vec![StorageSlot::with_empty_value(value_slot_name.clone())],
        )?
        .with_supports_all_types();

        let account = AccountBuilder::new([0; 32])
            .account_type(AccountType::RegularAccountImmutableCode)
            .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
            .with_component(dummy_component)
            .build()?;

        let account_id = account.id();
        let default_address = Address::new(account_id);
        store.insert_account(&account, default_address).await?;

        // Apply 4 transactions to create states with nonces 1, 2, 3, 4
        let mut current_account = account;
        let mut account_states: Vec<(u64, miden_client::Word)> = Vec::new(); // (nonce, commitment)

        for i in 1..=4 {
            let mut storage_delta = AccountStorageDelta::new();
            let new_value = [
                Felt::new(i as u64 * 10),
                Felt::new(i as u64 * 10 + 1),
                Felt::new(i as u64 * 10 + 2),
                Felt::new(i as u64 * 10 + 3),
            ];
            storage_delta.set_item(value_slot_name.clone(), new_value.into())?;

            let vault_delta = AccountVaultDelta::default();
            let delta = AccountDelta::new(current_account.id(), storage_delta, vault_delta, ONE)?;

            let init_header: AccountHeader = (&current_account).into();
            current_account.apply_delta(&delta)?;
            let final_header: AccountHeader = (&current_account).into();

            // Store the commitment for this state
            account_states.push((i, current_account.commitment()));

            let smt_forest = store.smt_forest.clone();
            let delta_clone = delta.clone();

            store
                .interact_with_connection(move |conn| {
                    let tx = conn.transaction().into_store_error()?;
                    let mut smt_forest =
                        smt_forest.write().expect("smt_forest write lock not poisoned");

                    SqliteStore::apply_account_delta(
                        &tx,
                        &mut smt_forest,
                        &init_header,
                        &final_header,
                        BTreeMap::default(),
                        BTreeMap::default(),
                        &delta_clone,
                    )?;

                    tx.commit().into_store_error()?;
                    Ok(())
                })
                .await?;
        }

        // Now create "pending" transaction records for states 2 and 3
        // These represent transactions that have been executed locally but not yet committed on-chain
        let state_2_commitment = account_states[1].1; // nonce=2
        let state_3_commitment = account_states[2].1; // nonce=3

        // Insert pending transaction for state 2
        let tx_details_2 = TransactionDetails {
            account_id,
            init_account_state: account_states[0].1, // starts from nonce=1
            final_account_state: state_2_commitment,
            input_note_nullifiers: vec![],
            output_notes: OutputNotes::new(vec![]).expect("valid empty output notes"),
            block_num: BlockNumber::from(1u32),
            submission_height: BlockNumber::from(1u32),
            expiration_block_num: BlockNumber::from(100u32),
            creation_timestamp: 12345,
        };

        let pending_tx_2 = TransactionRecord::new(
            TransactionId::from_raw([1; 4].map(Felt::new).into()),
            tx_details_2,
            None,
            TransactionStatus::Pending,
        );

        // Insert pending transaction for state 3
        let tx_details_3 = TransactionDetails {
            account_id,
            init_account_state: state_2_commitment,
            final_account_state: state_3_commitment,
            input_note_nullifiers: vec![],
            output_notes: OutputNotes::new(vec![]).expect("valid empty output notes"),
            block_num: BlockNumber::from(1u32),
            submission_height: BlockNumber::from(1u32),
            expiration_block_num: BlockNumber::from(100u32),
            creation_timestamp: 12346,
        };

        let pending_tx_3 = TransactionRecord::new(
            TransactionId::from_raw([2; 4].map(Felt::new).into()),
            tx_details_3,
            None,
            TransactionStatus::Pending,
        );

        // Insert the pending transactions into the database
        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().into_store_error()?;
                upsert_transaction_record(&tx, &pending_tx_2)?;
                upsert_transaction_record(&tx, &pending_tx_3)?;
                tx.commit().into_store_error()?;
                Ok(())
            })
            .await?;

        // Now prune - nothing should be pruned because all states are protected:
        // - S0: initial state (has account_seed)
        // - S1: init_account_state of pending tx2 (rollback point if tx2 fails)
        // - S2: final of tx2 / init of tx3
        // - S3: final of tx3
        // - S4: latest state
        println!("\n=== Pending Transaction Protection Test ===");
        println!("Account states: nonces 0 (initial), 1, 2, 3, 4 (latest)");
        println!("Pending tx2: S1 -> S2 (protects both S1 and S2)");
        println!("Pending tx3: S2 -> S3 (protects both S2 and S3)");
        println!("Latest: S4 (protected)");

        // Verify pruning does nothing
        let pruned = store.prune_account_history(account_id).await?;
        assert!(
            pruned.is_empty(),
            "Expected nothing pruned (all states protected), got {} states",
            pruned.state_count()
        );

        // Verify all states still exist after pruning (nothing should have changed)
        let remaining_states = store
            .interact_with_connection(move |conn| {
                let mut stmt = conn
                    .prepare("SELECT nonce FROM accounts WHERE id = ? ORDER BY nonce")
                    .into_store_error()?;
                let nonces: Vec<i64> = stmt
                    .query_map([account_id.to_hex()], |row| row.get(0))
                    .into_store_error()?
                    .filter_map(|r| r.ok())
                    .collect();
                Ok(nonces)
            })
            .await?;

        println!("Remaining states after pruning: {:?}", remaining_states);

        // ALL states should still exist: 0 (initial), 1, 2, 3, 4 (latest)
        assert!(remaining_states.contains(&0), "Initial state (nonce=0) must be preserved");
        assert!(remaining_states.contains(&1), "State (nonce=1) must be preserved (init of pending tx)");
        assert!(remaining_states.contains(&2), "State (nonce=2) must be preserved (final of pending tx)");
        assert!(remaining_states.contains(&3), "State (nonce=3) must be preserved (final of pending tx)");
        assert!(remaining_states.contains(&4), "Latest state (nonce=4) must be preserved");
        assert_eq!(remaining_states.len(), 5, "All 5 states should be preserved");

        Ok(())
    }

    /// Test the exact scenario from issue #1158:
    /// "We should only need to track the latest committed account state and all subsequent pending states."
    ///
    /// Scenario:
    /// - S0: initial state
    /// - S1: from COMMITTED tx (old, can be pruned)
    /// - S2: from COMMITTED tx (latest committed = init of first pending, KEEP)
    /// - S3: from PENDING tx (KEEP)
    /// - S4: from PENDING tx (latest, KEEP)
    ///
    /// Only S1 should be prunable.
    #[tokio::test]
    async fn prune_old_committed_keep_latest_committed_and_pending() -> anyhow::Result<()> {
        use miden_client::transaction::{
            TransactionDetails, TransactionId, TransactionRecord, TransactionStatus,
        };
        use miden_protocol::block::BlockNumber;
        use miden_protocol::transaction::OutputNotes;

        use crate::transaction::upsert_transaction_record;

        let store = create_test_store().await;

        let value_slot_name =
            StorageSlotName::new("miden::testing::pruning::issue1158").expect("valid slot name");

        let dummy_component = AccountComponent::new(
            basic_wallet_library(),
            vec![StorageSlot::with_empty_value(value_slot_name.clone())],
        )?
        .with_supports_all_types();

        let account = AccountBuilder::new([0; 32])
            .account_type(AccountType::RegularAccountImmutableCode)
            .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
            .with_component(dummy_component)
            .build()?;

        let account_id = account.id();
        let default_address = Address::new(account_id);
        store.insert_account(&account, default_address).await?;

        // Apply 4 transactions to create states with nonces 1, 2, 3, 4
        let mut current_account = account.clone();
        let mut account_states: Vec<(u64, miden_client::Word)> = Vec::new();
        // Include initial state
        account_states.push((0, account.commitment()));

        for i in 1..=4 {
            let mut storage_delta = AccountStorageDelta::new();
            let new_value = [
                Felt::new(i as u64 * 100),
                Felt::new(i as u64 * 100 + 1),
                Felt::new(i as u64 * 100 + 2),
                Felt::new(i as u64 * 100 + 3),
            ];
            storage_delta.set_item(value_slot_name.clone(), new_value.into())?;

            let vault_delta = AccountVaultDelta::default();
            let delta = AccountDelta::new(current_account.id(), storage_delta, vault_delta, ONE)?;

            let init_header: AccountHeader = (&current_account).into();
            current_account.apply_delta(&delta)?;
            let final_header: AccountHeader = (&current_account).into();

            account_states.push((i, current_account.commitment()));

            let smt_forest = store.smt_forest.clone();
            let delta_clone = delta.clone();

            store
                .interact_with_connection(move |conn| {
                    let tx = conn.transaction().into_store_error()?;
                    let mut smt_forest =
                        smt_forest.write().expect("smt_forest write lock not poisoned");

                    SqliteStore::apply_account_delta(
                        &tx,
                        &mut smt_forest,
                        &init_header,
                        &final_header,
                        BTreeMap::default(),
                        BTreeMap::default(),
                        &delta_clone,
                    )?;

                    tx.commit().into_store_error()?;
                    Ok(())
                })
                .await?;
        }

        // Now create transaction records:
        // tx1: S0 -> S1, COMMITTED (old committed, can be pruned)
        // tx2: S1 -> S2, COMMITTED (latest committed, KEEP)
        // tx3: S2 -> S3, PENDING (KEEP)
        // tx4: S3 -> S4, PENDING (KEEP)

        let tx_records = vec![
            // tx1: COMMITTED
            TransactionRecord::new(
                TransactionId::from_raw([1; 4].map(Felt::new).into()),
                TransactionDetails {
                    account_id,
                    init_account_state: account_states[0].1, // S0
                    final_account_state: account_states[1].1, // S1
                    input_note_nullifiers: vec![],
                    output_notes: OutputNotes::new(vec![]).expect("valid"),
                    block_num: BlockNumber::from(1u32),
                    submission_height: BlockNumber::from(1u32),
                    expiration_block_num: BlockNumber::from(100u32),
                    creation_timestamp: 1000,
                },
                None,
                TransactionStatus::Committed {
                    block_number: BlockNumber::from(10u32),
                    commit_timestamp: 2000,
                },
            ),
            // tx2: COMMITTED (latest committed)
            TransactionRecord::new(
                TransactionId::from_raw([2; 4].map(Felt::new).into()),
                TransactionDetails {
                    account_id,
                    init_account_state: account_states[1].1, // S1
                    final_account_state: account_states[2].1, // S2
                    input_note_nullifiers: vec![],
                    output_notes: OutputNotes::new(vec![]).expect("valid"),
                    block_num: BlockNumber::from(2u32),
                    submission_height: BlockNumber::from(2u32),
                    expiration_block_num: BlockNumber::from(100u32),
                    creation_timestamp: 3000,
                },
                None,
                TransactionStatus::Committed {
                    block_number: BlockNumber::from(20u32),
                    commit_timestamp: 4000,
                },
            ),
            // tx3: PENDING
            TransactionRecord::new(
                TransactionId::from_raw([3; 4].map(Felt::new).into()),
                TransactionDetails {
                    account_id,
                    init_account_state: account_states[2].1, // S2 (latest committed)
                    final_account_state: account_states[3].1, // S3
                    input_note_nullifiers: vec![],
                    output_notes: OutputNotes::new(vec![]).expect("valid"),
                    block_num: BlockNumber::from(3u32),
                    submission_height: BlockNumber::from(3u32),
                    expiration_block_num: BlockNumber::from(100u32),
                    creation_timestamp: 5000,
                },
                None,
                TransactionStatus::Pending,
            ),
            // tx4: PENDING
            TransactionRecord::new(
                TransactionId::from_raw([4; 4].map(Felt::new).into()),
                TransactionDetails {
                    account_id,
                    init_account_state: account_states[3].1, // S3
                    final_account_state: account_states[4].1, // S4
                    input_note_nullifiers: vec![],
                    output_notes: OutputNotes::new(vec![]).expect("valid"),
                    block_num: BlockNumber::from(3u32),
                    submission_height: BlockNumber::from(3u32),
                    expiration_block_num: BlockNumber::from(100u32),
                    creation_timestamp: 6000,
                },
                None,
                TransactionStatus::Pending,
            ),
        ];

        // Insert all transaction records
        let tx_records_clone = tx_records.clone();
        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().into_store_error()?;
                for record in &tx_records_clone {
                    upsert_transaction_record(&tx, record)?;
                }
                tx.commit().into_store_error()?;
                Ok(())
            })
            .await?;

        println!("\n=== Issue #1158 Test: Latest Committed + Pending ===");
        println!("Account states:");
        println!("  S0 (nonce=0): initial state");
        println!("  S1 (nonce=1): COMMITTED tx (old, should be PRUNABLE)");
        println!("  S2 (nonce=2): COMMITTED tx (latest committed, KEEP)");
        println!("  S3 (nonce=3): PENDING tx (KEEP)");
        println!("  S4 (nonce=4): PENDING tx, also latest (KEEP)");

        // Prune - only S1 should be pruned
        let pruned = store.prune_account_history(account_id).await?;
        println!(
            "Pruned states: {:?}",
            pruned.states.iter().map(|s| s.nonce).collect::<Vec<_>>()
        );

        assert_eq!(pruned.state_count(), 1, "Should have pruned exactly 1 state");
        assert_eq!(pruned.states[0].nonce, 1, "Should have pruned S1");

        // Verify remaining states
        let remaining_states = store
            .interact_with_connection(move |conn| {
                let mut stmt = conn
                    .prepare("SELECT nonce FROM accounts WHERE id = ? ORDER BY nonce")
                    .into_store_error()?;
                let nonces: Vec<i64> = stmt
                    .query_map([account_id.to_hex()], |row| row.get(0))
                    .into_store_error()?
                    .filter_map(|r| r.ok())
                    .collect();
                Ok(nonces)
            })
            .await?;

        println!("Remaining states after pruning: {:?}", remaining_states);

        // Should have: 0 (initial), 2 (latest committed), 3, 4 (pending)
        assert_eq!(remaining_states, vec![0, 2, 3, 4], "Should keep S0, S2, S3, S4");

        // S1 should be gone
        assert!(
            !remaining_states.contains(&1),
            "S1 (old committed) should have been pruned"
        );

        // S2 should be kept as the latest committed (init of first pending)
        assert!(
            remaining_states.contains(&2),
            "S2 (latest committed) must be preserved"
        );

        Ok(())
    }
}
