use std::collections::BTreeMap;
use std::vec::Vec;

use anyhow::Context;
use miden_client::account::component::{AccountComponent, basic_wallet_library};
use miden_client::account::{
    Account,
    AccountBuilder,
    AccountCode,
    AccountDelta,
    AccountHeader,
    AccountId,
    AccountType,
    Address,
    StorageMap,
    StorageSlot,
    StorageSlotContent,
    StorageSlotName,
};
use miden_client::assembly::CodeBuilder;
use miden_client::asset::{
    AccountStorageDelta,
    AccountVaultDelta,
    Asset,
    FungibleAsset,
    NonFungibleAsset,
    NonFungibleAssetDetails,
};
use miden_client::auth::{AuthFalcon512Rpo, PublicKeyCommitment};
use miden_client::store::Store;
use miden_client::testing::common::ACCOUNT_ID_REGULAR;
use miden_client::{EMPTY_WORD, Felt, ONE, ZERO};
use miden_protocol::testing::account_id::{
    ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
    ACCOUNT_ID_PUBLIC_NON_FUNGIBLE_FAUCET,
};
use miden_protocol::testing::constants::NON_FUNGIBLE_ASSET_DATA;

use crate::SqliteStore;
use crate::sql_error::SqlResultExt;
use crate::tests::create_test_store;

#[tokio::test]
async fn account_code_insertion_no_duplicates() -> anyhow::Result<()> {
    let store = create_test_store().await;
    let component_code = CodeBuilder::default()
        .compile_component_code("miden::testing::dummy_component", "pub proc dummy nop end")?;
    let account_component =
        AccountComponent::new(component_code, vec![])?.with_supports_all_types();
    let account_code = AccountCode::from_components(
        &[
            AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)).into(),
            account_component,
        ],
        AccountType::RegularAccountUpdatableCode,
    )?;

    store
        .interact_with_connection(move |conn| {
            let tx = conn.transaction().into_store_error()?;

            // Table is empty at the beginning
            let mut actual: usize = tx
                .query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0))
                .into_store_error()?;
            assert_eq!(actual, 0);

            // First insertion generates a new row
            SqliteStore::insert_account_code(&tx, &account_code)?;
            actual = tx
                .query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0))
                .into_store_error()?;
            assert_eq!(actual, 1);

            // Second insertion passes but does not generate a new row
            assert!(SqliteStore::insert_account_code(&tx, &account_code).is_ok());
            actual = tx
                .query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0))
                .into_store_error()?;
            assert_eq!(actual, 1);

            Ok(())
        })
        .await?;

    Ok(())
}

#[tokio::test]
async fn apply_account_delta_additions() -> anyhow::Result<()> {
    let store = create_test_store().await;

    let value_slot_name =
        StorageSlotName::new("miden::testing::sqlite_store::value").expect("valid slot name");
    let map_slot_name =
        StorageSlotName::new("miden::testing::sqlite_store::map").expect("valid slot name");

    let dummy_component = AccountComponent::new(
        basic_wallet_library(),
        vec![
            StorageSlot::with_empty_value(value_slot_name.clone()),
            StorageSlot::with_empty_map(map_slot_name.clone()),
        ],
    )?
    .with_supports_all_types();

    // Create and insert an account
    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build()?;

    let default_address = Address::new(account.id());
    store.insert_account(&account, default_address).await?;

    let mut storage_delta = AccountStorageDelta::new();
    storage_delta.set_item(value_slot_name.clone(), [ZERO, ZERO, ZERO, ONE].into())?;
    storage_delta.set_map_item(
        map_slot_name.clone(),
        [ONE, ZERO, ZERO, ZERO].into(),
        [ONE, ONE, ONE, ONE].into(),
    )?;

    let vault_delta = AccountVaultDelta::from_iters(
        vec![
            FungibleAsset::new(AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET)?, 100)?
                .into(),
            NonFungibleAsset::new(&NonFungibleAssetDetails::new(
                AccountId::try_from(ACCOUNT_ID_PUBLIC_NON_FUNGIBLE_FAUCET)?.prefix(),
                NON_FUNGIBLE_ASSET_DATA.into(),
            )?)?
            .into(),
        ],
        [],
    );

    let delta = AccountDelta::new(account.id(), storage_delta, vault_delta, ONE)?;

    let mut account_after_delta = account.clone();
    account_after_delta.apply_delta(&delta)?;

    let account_id = account.id();
    let final_state: AccountHeader = (&account_after_delta).into();
    let smt_forest = store.smt_forest.clone();
    store
        .interact_with_connection(move |conn| {
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");

            SqliteStore::apply_account_delta(
                &tx,
                &mut smt_forest,
                &account.into(),
                &final_state,
                BTreeMap::default(),
                BTreeMap::default(),
                &delta,
            )?;

            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    let updated_account: Account = store
        .get_account(account_id)
        .await?
        .context("failed to find inserted account")?
        .try_into()?;

    assert_eq!(updated_account, account_after_delta);

    Ok(())
}

#[tokio::test]
async fn apply_account_delta_removals() -> anyhow::Result<()> {
    let store = create_test_store().await;

    let value_slot_name =
        StorageSlotName::new("miden::testing::sqlite_store::value").expect("valid slot name");
    let map_slot_name =
        StorageSlotName::new("miden::testing::sqlite_store::map").expect("valid slot name");

    let mut dummy_map = StorageMap::new();
    dummy_map.insert([ONE, ZERO, ZERO, ZERO].into(), [ONE, ONE, ONE, ONE].into())?;

    let dummy_component = AccountComponent::new(
        basic_wallet_library(),
        vec![
            StorageSlot::with_value(value_slot_name.clone(), [ZERO, ZERO, ZERO, ONE].into()),
            StorageSlot::with_map(map_slot_name.clone(), dummy_map),
        ],
    )?
    .with_supports_all_types();

    // Create and insert an account
    let assets: Vec<Asset> = vec![
        FungibleAsset::new(AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET)?, 100)?.into(),
        NonFungibleAsset::new(&NonFungibleAssetDetails::new(
            AccountId::try_from(ACCOUNT_ID_PUBLIC_NON_FUNGIBLE_FAUCET)?.prefix(),
            NON_FUNGIBLE_ASSET_DATA.into(),
        )?)?
        .into(),
    ];
    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .with_assets(assets.clone())
        .build_existing()?;
    let default_address = Address::new(account.id());
    store.insert_account(&account, default_address).await?;

    let mut storage_delta = AccountStorageDelta::new();
    storage_delta.set_item(value_slot_name.clone(), EMPTY_WORD)?;
    storage_delta.set_map_item(
        map_slot_name.clone(),
        [ONE, ZERO, ZERO, ZERO].into(),
        EMPTY_WORD,
    )?;

    let vault_delta = AccountVaultDelta::from_iters([], assets.clone());

    let delta = AccountDelta::new(account.id(), storage_delta, vault_delta, ONE)?;

    let mut account_after_delta = account.clone();
    account_after_delta.apply_delta(&delta)?;

    let account_id = account.id();
    let final_state: AccountHeader = (&account_after_delta).into();

    let smt_forest = store.smt_forest.clone();
    store
        .interact_with_connection(move |conn| {
            let fungible_assets =
                SqliteStore::get_account_fungible_assets_for_delta(conn, account.id(), &delta)?;
            let storage_maps =
                SqliteStore::get_account_storage_maps_for_delta(conn, account.id(), &delta)?;
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");

            SqliteStore::apply_account_delta(
                &tx,
                &mut smt_forest,
                &account.into(),
                &final_state,
                fungible_assets,
                storage_maps,
                &delta,
            )?;

            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    let updated_account: Account = store
        .get_account(account_id)
        .await?
        .context("failed to find inserted account")?
        .try_into()?;

    assert_eq!(updated_account, account_after_delta);
    assert!(updated_account.vault().is_empty());
    assert_eq!(updated_account.storage().get_item(&value_slot_name)?, EMPTY_WORD);
    let map_slot = updated_account
        .storage()
        .slots()
        .iter()
        .find(|slot| slot.name() == &map_slot_name)
        .expect("storage should contain map slot");
    let StorageSlotContent::Map(updated_map) = map_slot.content() else {
        panic!("Expected map slot content");
    };
    assert_eq!(updated_map.entries().count(), 0);

    Ok(())
}

#[tokio::test]
async fn get_account_storage_item_success() -> anyhow::Result<()> {
    let store = create_test_store().await;

    let value_slot_name =
        StorageSlotName::new("miden::testing::sqlite_store::value").expect("valid slot name");
    let test_value: [miden_client::Felt; 4] = [ONE, ONE, ONE, ONE];

    let dummy_component = AccountComponent::new(
        basic_wallet_library(),
        vec![StorageSlot::with_value(value_slot_name.clone(), test_value.into())],
    )?
    .with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build_existing()?;

    let default_address = Address::new(account.id());
    store.insert_account(&account, default_address).await?;

    // Test get_account_storage_item
    let result = store.get_account_storage_item(account.id(), value_slot_name).await?;

    assert_eq!(result, test_value.into());

    Ok(())
}

#[tokio::test]
async fn get_account_storage_item_not_found() -> anyhow::Result<()> {
    let store = create_test_store().await;

    let value_slot_name =
        StorageSlotName::new("miden::testing::sqlite_store::value").expect("valid slot name");

    let dummy_component = AccountComponent::new(
        basic_wallet_library(),
        vec![StorageSlot::with_empty_value(value_slot_name)],
    )?
    .with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build_existing()?;

    let default_address = Address::new(account.id());
    store.insert_account(&account, default_address).await?;

    // Test get_account_storage_item with missing slot name
    let missing_name =
        StorageSlotName::new("miden::testing::sqlite_store::missing").expect("valid slot name");
    let result = store.get_account_storage_item(account.id(), missing_name).await;

    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn get_account_map_item_success() -> anyhow::Result<()> {
    let store = create_test_store().await;

    let map_slot_name =
        StorageSlotName::new("miden::testing::sqlite_store::map").expect("valid slot name");

    let test_key: miden_client::Word = [ONE, ZERO, ZERO, ZERO].into();
    let test_value: miden_client::Word = [ONE, ONE, ONE, ONE].into();

    let mut storage_map = StorageMap::new();
    storage_map.insert(test_key, test_value)?;

    let dummy_component = AccountComponent::new(
        basic_wallet_library(),
        vec![StorageSlot::with_map(map_slot_name.clone(), storage_map)],
    )?
    .with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build_existing()?;

    let default_address = Address::new(account.id());
    store.insert_account(&account, default_address).await?;

    // Test get_account_map_item
    let (value, _witness) =
        store.get_account_map_item(account.id(), map_slot_name, test_key).await?;

    assert_eq!(value, test_value);

    Ok(())
}

#[tokio::test]
async fn get_account_map_item_value_slot_error() -> anyhow::Result<()> {
    let store = create_test_store().await;

    let value_slot_name =
        StorageSlotName::new("miden::testing::sqlite_store::value").expect("valid slot name");

    let dummy_component = AccountComponent::new(
        basic_wallet_library(),
        vec![StorageSlot::with_empty_value(value_slot_name.clone())],
    )?
    .with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build_existing()?;

    let default_address = Address::new(account.id());
    store.insert_account(&account, default_address).await?;

    // Test get_account_map_item on a value slot (should error)
    let test_key: miden_client::Word = [ONE, ZERO, ZERO, ZERO].into();
    let result = store.get_account_map_item(account.id(), value_slot_name, test_key).await;

    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn get_account_code() -> anyhow::Result<()> {
    let store = create_test_store().await;

    let dummy_component =
        AccountComponent::new(basic_wallet_library(), vec![])?.with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build_existing()?;

    let default_address = Address::new(account.id());
    store.insert_account(&account, default_address).await?;

    let code = store.get_account_code(account.id()).await?;

    assert!(code.is_some());
    let code = code.unwrap();
    assert_eq!(code.commitment(), account.code().commitment());

    Ok(())
}

#[tokio::test]
async fn get_account_code_not_found() -> anyhow::Result<()> {
    let store = create_test_store().await;

    // Create a valid but non-existent account ID
    let non_existent_id = AccountId::try_from(ACCOUNT_ID_REGULAR)?;

    // Test get_account_code with non-existent account
    let result = store.get_account_code(non_existent_id).await?;

    assert!(result.is_none());

    Ok(())
}

// ACCOUNT READER TESTS
// ================================================================================================

#[tokio::test]
async fn account_reader_nonce_and_status() -> anyhow::Result<()> {
    use std::sync::Arc;

    use miden_client::account::AccountReader;

    let store = Arc::new(create_test_store().await);

    let dummy_component =
        AccountComponent::new(basic_wallet_library(), vec![])?.with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build()?;

    let default_address = Address::new(account.id());
    store.insert_account(&account, default_address).await?;

    // Create an AccountReader
    let reader = AccountReader::new(store.clone(), account.id());

    // Test nonce access
    let nonce = reader.nonce().await?;
    assert_eq!(nonce, account.nonce());

    // Test status access
    let status = reader.status().await?;
    assert!(!status.is_locked());
    assert!(status.seed().is_some()); // New account should have a seed

    // Test commitment
    let commitment = reader.commitment().await?;
    assert_eq!(commitment, account.commitment());

    Ok(())
}

#[tokio::test]
async fn account_reader_not_found_error() -> anyhow::Result<()> {
    use std::sync::Arc;

    use miden_client::account::AccountReader;

    let store = Arc::new(create_test_store().await);

    // Create a valid but non-existent account ID
    let non_existent_id = AccountId::try_from(ACCOUNT_ID_REGULAR)?;

    // Create an AccountReader for non-existent account
    let reader = AccountReader::new(store.clone(), non_existent_id);

    // Test that header-based methods return AccountDataNotFound error
    let result = reader.nonce().await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), miden_client::ClientError::AccountDataNotFound(_)));

    // Test that status() returns AccountDataNotFound error
    let result = reader.status().await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), miden_client::ClientError::AccountDataNotFound(_)));

    Ok(())
}

#[tokio::test]
async fn account_reader_storage_access() -> anyhow::Result<()> {
    use std::sync::Arc;

    use miden_client::account::AccountReader;

    let store = Arc::new(create_test_store().await);

    let value_slot_name =
        StorageSlotName::new("miden::testing::sqlite_store::value").expect("valid slot name");
    let test_value: [miden_client::Felt; 4] = [ONE, ONE, ONE, ONE];

    let dummy_component = AccountComponent::new(
        basic_wallet_library(),
        vec![StorageSlot::with_value(value_slot_name.clone(), test_value.into())],
    )?
    .with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build_existing()?;

    let default_address = Address::new(account.id());
    store.insert_account(&account, default_address).await?;

    // Create an AccountReader
    let reader = AccountReader::new(store.clone(), account.id());

    // Test storage access via integrated method
    let result = reader.get_storage_item(value_slot_name).await?;

    assert_eq!(result, test_value.into());

    Ok(())
}

#[tokio::test]
async fn account_reader_addresses_access() -> anyhow::Result<()> {
    use std::sync::Arc;

    use miden_client::account::AccountReader;

    let store = Arc::new(create_test_store().await);

    let dummy_component =
        AccountComponent::new(basic_wallet_library(), vec![])?.with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(dummy_component)
        .build_existing()?;

    let default_address = Address::new(account.id());
    store.insert_account(&account, default_address.clone()).await?;

    // Create an AccountReader
    let reader = AccountReader::new(store.clone(), account.id());

    // Test addresses access
    let addresses = reader.addresses().await?;
    assert_eq!(addresses.len(), 1);
    assert_eq!(addresses[0], default_address);

    Ok(())
}

// STORAGE MODEL BENCHMARK (issue #1768)
// ================================================================================================

/// Row counts across the account-related tables.
struct StorageMetrics {
    latest_account_headers: usize,
    historical_account_headers: usize,
    latest_account_storage: usize,
    latest_storage_map_entries: usize,
    latest_account_assets: usize,
    historical_account_storage: usize,
    historical_storage_map_entries: usize,
    historical_account_assets: usize,
}

impl std::fmt::Display for StorageMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "latest_headers={:<3} hist_headers={:<3} latest_storage={:<3} latest_map={:<3} \
             latest_assets={:<3} hist_storage={:<3} hist_map={:<3} hist_assets={:<3}",
            self.latest_account_headers,
            self.historical_account_headers,
            self.latest_account_storage,
            self.latest_storage_map_entries,
            self.latest_account_assets,
            self.historical_account_storage,
            self.historical_storage_map_entries,
            self.historical_account_assets,
        )
    }
}

async fn get_storage_metrics(store: &SqliteStore) -> StorageMetrics {
    store
        .interact_with_connection(|conn| {
            let count = |table| {
                conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                    .into_store_error()
            };
            Ok(StorageMetrics {
                latest_account_headers: count("latest_account_headers")?,
                historical_account_headers: count("historical_account_headers")?,
                latest_account_storage: count("latest_account_storage")?,
                latest_storage_map_entries: count("latest_storage_map_entries")?,
                latest_account_assets: count("latest_account_assets")?,
                historical_account_storage: count("historical_account_storage")?,
                historical_storage_map_entries: count("historical_storage_map_entries")?,
                historical_account_assets: count("historical_account_assets")?,
            })
        })
        .await
        .unwrap()
}

/// Creates an account with a storage map of `map_size` entries, inserts it into the store,
/// and returns the account. Uses Store::insert_account (public API).
async fn setup_account_with_map(
    store: &SqliteStore,
    map_size: u64,
    map_slot_name: &StorageSlotName,
) -> anyhow::Result<Account> {
    let mut map = StorageMap::new();
    for i in 1..=map_size {
        map.insert(
            [Felt::new(i), ZERO, ZERO, ZERO].into(),
            [Felt::new(i * 100), ZERO, ZERO, ZERO].into(),
        )?;
    }

    let component = AccountComponent::new(
        basic_wallet_library(),
        vec![StorageSlot::with_map(map_slot_name.clone(), map)],
    )?
    .with_supports_all_types();

    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(component)
        .build()?;

    store.insert_account(&account, Address::new(account.id())).await?;
    Ok(account)
}

/// Applies a delta that changes a single map entry (key=1) and persists it.
async fn apply_single_entry_update(
    store: &SqliteStore,
    account: &mut Account,
    map_slot_name: &StorageSlotName,
    nonce: u64,
) -> anyhow::Result<()> {
    let mut storage_delta = AccountStorageDelta::new();
    storage_delta.set_map_item(
        map_slot_name.clone(),
        [Felt::new(1), ZERO, ZERO, ZERO].into(),
        [Felt::new(nonce * 1000), ZERO, ZERO, ZERO].into(),
    )?;

    let delta = AccountDelta::new(
        account.id(),
        storage_delta,
        AccountVaultDelta::from_iters([], []),
        Felt::new(nonce),
    )?;

    let prev_header: AccountHeader = (&*account).into();
    account.apply_delta(&delta)?;
    let final_header: AccountHeader = (&*account).into();

    let smt_forest = store.smt_forest.clone();
    let delta_clone = delta.clone();
    let account_id = account.id();
    store
        .interact_with_connection(move |conn| {
            let storage_maps =
                SqliteStore::get_account_storage_maps_for_delta(conn, account_id, &delta_clone)?;
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");

            SqliteStore::apply_account_delta(
                &tx,
                &mut smt_forest,
                &prev_header,
                &final_header,
                BTreeMap::default(),
                storage_maps,
                &delta,
            )?;

            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    Ok(())
}

/// Measures storage row growth and verifies correctness after multiple updates.
///
/// After refactor:
///   50 map entries + 5 updates:
///     latest_storage_map_entries = 50 (always the full current state)
///     historical_storage_map_entries = 50 (initial) + 5 (one changed entry per update) = 55
#[tokio::test]
async fn storage_model_benchmark() -> anyhow::Result<()> {
    const MAP_SIZE: u64 = 50;
    const NUM_UPDATES: u64 = 5;

    let store = create_test_store().await;
    let map_slot_name = StorageSlotName::new("test::benchmark::map").expect("valid slot name");

    // ── Phase 1: Data growth ─────────────────────────────────────────────
    let mut account = setup_account_with_map(&store, MAP_SIZE, &map_slot_name).await?;
    let account_id = account.id();

    eprintln!("\n=== STORAGE MODEL BENCHMARK (issue #1768) ===");
    eprintln!("Map size: {MAP_SIZE} entries, Updates: {NUM_UPDATES}\n");

    let m = get_storage_metrics(&store).await;
    eprintln!("After insert:    {m}");

    for i in 1..=NUM_UPDATES {
        apply_single_entry_update(&store, &mut account, &map_slot_name, i).await?;
        let m = get_storage_metrics(&store).await;
        eprintln!("After update {i}:  {m}");
    }

    let after_updates = get_storage_metrics(&store).await;
    let num_states = (1 + NUM_UPDATES) as usize;

    // Latest account headers: always one row per account
    assert_eq!(after_updates.latest_account_headers, 1);
    // Historical account headers: one row per state transition
    assert_eq!(after_updates.historical_account_headers, num_states);

    // Latest tables always hold the full current state
    assert_eq!(after_updates.latest_storage_map_entries, MAP_SIZE as usize);
    // Account has 2 storage slots: the auth key slot + the map slot
    assert_eq!(after_updates.latest_account_storage, 2);

    // Historical: initial insert writes all N entries at nonce 0,
    // then each update writes ONLY the changed entry to historical.
    // Total: N (initial) + M (one changed entry per update) = 50 + 5 = 55.
    assert_eq!(
        after_updates.historical_storage_map_entries,
        MAP_SIZE as usize + NUM_UPDATES as usize
    );

    // ── Phase 2: Correctness (Store trait) ───────────────────────────────
    let changed_key = [Felt::new(1), ZERO, ZERO, ZERO].into();
    let (value, _witness) = store
        .get_account_map_item(account_id, map_slot_name.clone(), changed_key)
        .await?;
    let expected_value = [Felt::new(NUM_UPDATES * 1000), ZERO, ZERO, ZERO].into();
    assert_eq!(value, expected_value, "Changed entry should reflect the last update");

    let unchanged_key = [Felt::new(2), ZERO, ZERO, ZERO].into();
    let (value, _witness) = store
        .get_account_map_item(account_id, map_slot_name.clone(), unchanged_key)
        .await?;
    let expected_original = [Felt::new(200), ZERO, ZERO, ZERO].into();
    assert_eq!(value, expected_original, "Unchanged entry should keep its original value");

    // ── Summary ──────────────────────────────────────────────────────────
    eprintln!("\n=== SUMMARY ===");
    eprintln!(
        "Latest storage_map_entries:     {} (always N={MAP_SIZE})",
        after_updates.latest_storage_map_entries,
    );
    eprintln!(
        "Historical storage_map_entries: {}",
        after_updates.historical_storage_map_entries,
    );

    Ok(())
}
