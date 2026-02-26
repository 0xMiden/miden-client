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
use rusqlite::params;

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
                &BTreeMap::new(),
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
            let old_map_roots =
                SqliteStore::get_storage_map_roots_for_delta(conn, account.id(), &delta)?;
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");

            SqliteStore::apply_account_delta(
                &tx,
                &mut smt_forest,
                &account.into(),
                &final_state,
                fungible_assets,
                &old_map_roots,
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

// TEST HELPERS
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
/// and returns the account. Uses `Store::insert_account` (public API).
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
            let old_map_roots =
                SqliteStore::get_storage_map_roots_for_delta(conn, account_id, &delta_clone)?;
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");

            SqliteStore::apply_account_delta(
                &tx,
                &mut smt_forest,
                &prev_header,
                &final_header,
                BTreeMap::default(),
                &old_map_roots,
                &delta,
            )?;

            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    Ok(())
}

// UNDO & COMMITMENT LOOKUP TESTS (issue #1768)
// ================================================================================================

/// Verifies that `undo_account_state` correctly reverts the latest tables to the previous state.
/// Exercises `undo_account_state` + `rebuild_latest_for_account`.
///
/// The delta includes both storage and vault changes so that the vault root changes between
/// nonce 0 and nonce 1. This is required because `undo_account_state` pops SMT roots from the
/// forest, and the vault root must differ to avoid removing the initial state's root.
#[tokio::test]
async fn undo_account_state_restores_previous_latest() -> anyhow::Result<()> {
    let store = create_test_store().await;
    let map_slot_name = StorageSlotName::new("test::undo::map").expect("valid slot name");

    // Insert account with 5 map entries (nonce 0)
    let mut account = setup_account_with_map(&store, 5, &map_slot_name).await?;
    let initial_commitment = account.commitment();

    // Apply a delta (nonce 1) that changes a map entry AND adds a fungible asset.
    // The vault change ensures the vault root differs between nonce 0 and 1,
    // which is needed for pop_roots to work correctly.
    let mut storage_delta = AccountStorageDelta::new();
    storage_delta.set_map_item(
        map_slot_name.clone(),
        [Felt::new(1), ZERO, ZERO, ZERO].into(),
        [Felt::new(1000), ZERO, ZERO, ZERO].into(),
    )?;
    let vault_delta = AccountVaultDelta::from_iters(
        vec![
            FungibleAsset::new(AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET)?, 100)?
                .into(),
        ],
        [],
    );
    let delta = AccountDelta::new(account.id(), storage_delta, vault_delta, ONE)?;

    let prev_header: AccountHeader = (&account).into();
    account.apply_delta(&delta)?;
    let final_header: AccountHeader = (&account).into();
    let post_delta_commitment = account.commitment();

    let smt_forest = store.smt_forest.clone();
    let account_id = account.id();
    let delta_clone = delta.clone();
    store
        .interact_with_connection(move |conn| {
            let old_map_roots =
                SqliteStore::get_storage_map_roots_for_delta(conn, account_id, &delta_clone)?;
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
            SqliteStore::apply_account_delta(
                &tx,
                &mut smt_forest,
                &prev_header,
                &final_header,
                BTreeMap::default(),
                &old_map_roots,
                &delta,
            )?;
            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    // Pre-undo: 2 historical headers (nonce 0 + nonce 1), 1 latest
    let m = get_storage_metrics(&store).await;
    assert_eq!(m.historical_account_headers, 2);
    assert_eq!(m.latest_account_headers, 1);
    assert_eq!(m.latest_account_assets, 1);

    // Undo the nonce-1 state
    let smt_forest = store.smt_forest.clone();
    store
        .interact_with_connection(move |conn| {
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
            SqliteStore::undo_account_state(&tx, &mut smt_forest, &[post_delta_commitment])?;
            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    // After undo: only nonce-0 state remains in historical, latest rebuilt from it
    let m = get_storage_metrics(&store).await;
    assert_eq!(m.historical_account_headers, 1);
    assert_eq!(m.latest_account_headers, 1);
    assert_eq!(m.latest_storage_map_entries, 5);
    assert_eq!(m.historical_storage_map_entries, 5);
    assert_eq!(m.latest_account_assets, 0, "Vault should be empty after undo to nonce 0");

    // Latest header should reflect nonce 0 with the initial commitment
    let (header, _status) = store
        .interact_with_connection(move |conn| SqliteStore::get_account_header(conn, account_id))
        .await?
        .expect("account should still exist after undo");
    assert_eq!(header.nonce().as_int(), 0);
    assert_eq!(header.commitment(), initial_commitment);

    Ok(())
}

/// Verifies that undoing the only state (nonce 0) of an account removes it entirely from both
/// latest and historical tables. This exercises the `rebuild_latest_for_account` early-return
/// path when `MAX(nonce)` is None.
///
/// The account is created with assets so the vault root is non-trivial — the SMT forest
/// only ref-counts non-empty roots, so `pop_roots` after undo would underflow on an empty vault.
#[tokio::test]
async fn undo_account_state_deletes_account_entirely() -> anyhow::Result<()> {
    let store = create_test_store().await;
    let map_slot_name = StorageSlotName::new("test::undo_del::map").expect("valid slot name");

    // Build account with a map AND an asset so the vault root is non-trivial
    let mut map = StorageMap::new();
    for i in 1..=3u64 {
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
        .with_assets(vec![
            FungibleAsset::new(AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET)?, 100)?
                .into(),
        ])
        .build_existing()?;

    let account_id = account.id();
    let commitment = account.commitment();
    store.insert_account(&account, Address::new(account_id)).await?;

    // Pre-undo: 1 latest header, 1 historical header, storage/map/asset entries exist
    let m = get_storage_metrics(&store).await;
    assert_eq!(m.latest_account_headers, 1);
    assert_eq!(m.historical_account_headers, 1);
    assert!(m.latest_storage_map_entries > 0);
    assert_eq!(m.latest_account_assets, 1);

    // Undo the only state
    let smt_forest = store.smt_forest.clone();
    store
        .interact_with_connection(move |conn| {
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
            SqliteStore::undo_account_state(&tx, &mut smt_forest, &[commitment])?;
            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    // After undo: all tables should be empty for this account
    let m = get_storage_metrics(&store).await;
    assert_eq!(m.latest_account_headers, 0);
    assert_eq!(m.historical_account_headers, 0);
    assert_eq!(m.latest_account_storage, 0);
    assert_eq!(m.latest_storage_map_entries, 0);
    assert_eq!(m.historical_account_storage, 0);
    assert_eq!(m.historical_storage_map_entries, 0);
    assert_eq!(m.latest_account_assets, 0);
    assert_eq!(m.historical_account_assets, 0);

    // get_account should return None
    let result = store
        .interact_with_connection(move |conn| SqliteStore::get_account_header(conn, account_id))
        .await?;
    assert!(result.is_none());

    Ok(())
}

/// Verifies that `lock_account_on_unexpected_commitment` sets `locked = true` in both the
/// latest and historical tables so that the lock survives undo/rebuild.
#[tokio::test]
async fn lock_account_affects_latest_and_historical() -> anyhow::Result<()> {
    let store = create_test_store().await;
    let map_slot_name = StorageSlotName::new("test::lock::map").expect("valid slot name");

    // Insert account (nonce 0)
    let mut account = setup_account_with_map(&store, 3, &map_slot_name).await?;
    let account_id = account.id();

    // Apply a delta (nonce 1) with vault change
    let mut storage_delta = AccountStorageDelta::new();
    storage_delta.set_map_item(
        map_slot_name.clone(),
        [Felt::new(1), ZERO, ZERO, ZERO].into(),
        [Felt::new(2000), ZERO, ZERO, ZERO].into(),
    )?;
    let vault_delta = AccountVaultDelta::from_iters(
        vec![
            FungibleAsset::new(AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET)?, 100)?
                .into(),
        ],
        [],
    );
    let delta = AccountDelta::new(account.id(), storage_delta, vault_delta, ONE)?;
    let prev_header: AccountHeader = (&account).into();
    account.apply_delta(&delta)?;
    let final_header: AccountHeader = (&account).into();

    let smt_forest = store.smt_forest.clone();
    let delta_clone = delta.clone();
    store
        .interact_with_connection(move |conn| {
            let old_map_roots =
                SqliteStore::get_storage_map_roots_for_delta(conn, account_id, &delta_clone)?;
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
            SqliteStore::apply_account_delta(
                &tx,
                &mut smt_forest,
                &prev_header,
                &final_header,
                BTreeMap::default(),
                &old_map_roots,
                &delta,
            )?;
            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    // Pre-lock: 2 historical headers (nonce 0 + nonce 1), both unlocked
    let m = get_storage_metrics(&store).await;
    assert_eq!(m.historical_account_headers, 2);

    // Lock the account with a fake mismatched digest (not matching any historical commitment)
    let fake_digest = [Felt::new(999), Felt::new(888), Felt::new(777), Felt::new(666)].into();
    store
        .interact_with_connection(move |conn| {
            let tx = conn.transaction().into_store_error()?;
            SqliteStore::lock_account_on_unexpected_commitment(&tx, &account_id, &fake_digest)?;
            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    // Latest should be locked
    let (_header, status) = store
        .interact_with_connection(move |conn| SqliteStore::get_account_header(conn, account_id))
        .await?
        .expect("account should exist");
    assert!(status.is_locked(), "Latest header should be locked");

    // Historical entries should also be locked (so rebuild preserves the lock)
    let historical_locked: Vec<bool> = store
        .interact_with_connection(move |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT locked FROM historical_account_headers WHERE id = ? ORDER BY nonce",
                )
                .into_store_error()?;
            let rows = stmt
                .query_map(params![account_id.to_hex()], |row| row.get(0))
                .into_store_error()?
                .collect::<Result<Vec<bool>, _>>()
                .into_store_error()?;
            Ok(rows)
        })
        .await?;
    assert_eq!(historical_locked.len(), 2, "Should have 2 historical entries");
    assert!(historical_locked[0], "Historical nonce-0 should be locked");
    assert!(historical_locked[1], "Historical nonce-1 should be locked");

    Ok(())
}

/// Verifies that undoing a delta after `update_account_state` does not resurrect entries that
/// were removed by the update. This exercises the tombstone-writing logic in
/// `update_account_state`.
///
/// Flow:
/// 1. Insert account with map entries {A, B, C} and an asset X at nonce 0
/// 2. Apply delta at nonce 1: add asset Y (changes vault root)
/// 3. `update_account_state` with in-memory state at nonce 2: {A, B} and {X} (C and Y removed)
/// 4. Apply delta at nonce 3: change entry A, add asset Z
/// 5. Undo nonce 3
/// 6. Assert C and Y are not in latest tables
#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn undo_after_update_account_state_does_not_resurrect_removed_entries() -> anyhow::Result<()>
{
    let store = create_test_store().await;
    let map_slot_name =
        StorageSlotName::new("miden::testing::sqlite_store::map").expect("valid slot name");

    let faucet_id = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET)?;
    let nf_faucet_id = AccountId::try_from(ACCOUNT_ID_PUBLIC_NON_FUNGIBLE_FAUCET)?;

    // Build initial map with 3 entries: A (key=1), B (key=2), C (key=3)
    let key_a = [Felt::new(1), ZERO, ZERO, ZERO].into();
    let key_c = [Felt::new(3), ZERO, ZERO, ZERO].into();

    let mut initial_map = StorageMap::new();
    initial_map.insert(key_a, [Felt::new(100), ZERO, ZERO, ZERO].into())?;
    initial_map.insert(
        [Felt::new(2), ZERO, ZERO, ZERO].into(),
        [Felt::new(200), ZERO, ZERO, ZERO].into(),
    )?;
    initial_map.insert(key_c, [Felt::new(300), ZERO, ZERO, ZERO].into())?;

    let component = AccountComponent::new(
        basic_wallet_library(),
        vec![StorageSlot::with_map(map_slot_name.clone(), initial_map)],
    )?
    .with_supports_all_types();

    // Build with build() at nonce 0 — no initial assets
    let account = AccountBuilder::new([0; 32])
        .account_type(AccountType::RegularAccountImmutableCode)
        .with_auth_component(AuthFalcon512Rpo::new(PublicKeyCommitment::from(EMPTY_WORD)))
        .with_component(component)
        .build()?;

    let account_id = account.id();
    store.insert_account(&account, Address::new(account_id)).await?;

    // Step 1+2: Apply delta at nonce 1 adding assets X and Y
    let asset_x = FungibleAsset::new(faucet_id, 100)?;
    let asset_y = NonFungibleAsset::new(&NonFungibleAssetDetails::new(
        nf_faucet_id.prefix(),
        NON_FUNGIBLE_ASSET_DATA.into(),
    )?)?;

    let vault_delta_1 = AccountVaultDelta::from_iters(vec![asset_x.into(), asset_y.into()], []);
    let delta_1 = AccountDelta::new(account_id, AccountStorageDelta::new(), vault_delta_1, ONE)?;

    let prev_header_0: AccountHeader = (&account).into();
    let mut account_nonce1 = account.clone();
    account_nonce1.apply_delta(&delta_1)?;
    let final_header_1: AccountHeader = (&account_nonce1).into();

    let smt_forest = store.smt_forest.clone();
    store
        .interact_with_connection(move |conn| {
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
            SqliteStore::apply_account_delta(
                &tx,
                &mut smt_forest,
                &prev_header_0,
                &final_header_1,
                BTreeMap::default(),
                &BTreeMap::new(),
                &delta_1,
            )?;
            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    // Now: map entries {A, B, C} and assets {X, Y} at nonce 1
    let m = get_storage_metrics(&store).await;
    assert_eq!(m.latest_storage_map_entries, 3, "Should have 3 map entries");
    assert_eq!(m.latest_account_assets, 2, "Should have 2 assets (X + Y)");

    // Step 3: Build in-memory state with only {A, B} and {X} (C and Y removed)
    let mut storage_delta_remove = AccountStorageDelta::new();
    storage_delta_remove.set_map_item(map_slot_name.clone(), key_c, EMPTY_WORD)?;
    let vault_delta_remove = AccountVaultDelta::from_iters([], vec![asset_y.into()]);
    let delta_remove =
        AccountDelta::new(account_id, storage_delta_remove, vault_delta_remove, ONE)?;

    let mut account_updated = account_nonce1.clone();
    account_updated.apply_delta(&delta_remove)?;
    let updated_nonce = account_updated.nonce().as_int();

    // Call update_account_state with the updated state
    let smt_forest = store.smt_forest.clone();
    let account_updated_clone = account_updated.clone();
    store
        .interact_with_connection(move |conn| {
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
            SqliteStore::update_account_state(&tx, &mut smt_forest, &account_updated_clone)?;
            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    // After update: 2 map entries (A, B), 1 asset (X)
    let m = get_storage_metrics(&store).await;
    assert_eq!(m.latest_storage_map_entries, 2, "Should have 2 map entries after update");
    assert_eq!(m.latest_account_assets, 1, "Should have 1 asset after update");

    // Step 4: Apply a delta that changes entry A and adds asset Z
    let mut storage_delta_next = AccountStorageDelta::new();
    storage_delta_next.set_map_item(
        map_slot_name.clone(),
        key_a,
        [Felt::new(999), ZERO, ZERO, ZERO].into(),
    )?;

    let asset_z = NonFungibleAsset::new(&NonFungibleAssetDetails::new(
        nf_faucet_id.prefix(),
        vec![5, 6, 7, 8],
    )?)?;
    let vault_delta_next = AccountVaultDelta::from_iters(vec![asset_z.into()], []);

    let delta_next = AccountDelta::new(account_id, storage_delta_next, vault_delta_next, ONE)?;

    let prev_header: AccountHeader = (&account_updated).into();
    let mut account_next = account_updated.clone();
    account_next.apply_delta(&delta_next)?;
    let final_header: AccountHeader = (&account_next).into();
    let commitment_next = account_next.commitment();

    let smt_forest = store.smt_forest.clone();
    let delta_next_clone = delta_next.clone();
    store
        .interact_with_connection(move |conn| {
            let old_map_roots =
                SqliteStore::get_storage_map_roots_for_delta(conn, account_id, &delta_next_clone)?;
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
            SqliteStore::apply_account_delta(
                &tx,
                &mut smt_forest,
                &prev_header,
                &final_header,
                BTreeMap::default(),
                &old_map_roots,
                &delta_next,
            )?;
            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    // After delta: 2 map entries (A modified, B unchanged), 2 assets (X + Z)
    let m = get_storage_metrics(&store).await;
    assert_eq!(m.latest_storage_map_entries, 2, "Should have 2 map entries after delta");
    assert_eq!(m.latest_account_assets, 2, "Should have 2 assets after delta (X + Z)");

    // Step 5: Undo the last delta
    let smt_forest = store.smt_forest.clone();
    store
        .interact_with_connection(move |conn| {
            let tx = conn.transaction().into_store_error()?;
            let mut smt_forest = smt_forest.write().expect("smt_forest write lock not poisoned");
            SqliteStore::undo_account_state(&tx, &mut smt_forest, &[commitment_next])?;
            tx.commit().into_store_error()?;
            Ok(())
        })
        .await?;

    // Step 6: Verify C and Y are NOT resurrected
    let m = get_storage_metrics(&store).await;
    assert_eq!(
        m.latest_storage_map_entries, 2,
        "C should NOT be resurrected — only A and B should be in latest"
    );
    assert_eq!(
        m.latest_account_assets, 1,
        "Y should NOT be resurrected — only X should be in latest"
    );

    // Also verify the header reverted to the post-update nonce
    let (header, _) = store
        .interact_with_connection(move |conn| SqliteStore::get_account_header(conn, account_id))
        .await?
        .expect("account should exist");
    assert_eq!(header.nonce().as_int(), updated_nonce);

    Ok(())
}

/// Verifies that `get_account_header_by_commitment` retrieves historical states by commitment.
#[tokio::test]
async fn get_account_header_by_commitment_returns_historical() -> anyhow::Result<()> {
    let store = create_test_store().await;
    let map_slot_name = StorageSlotName::new("test::commitment::map").expect("valid slot name");

    // Insert account (nonce 0)
    let mut account = setup_account_with_map(&store, 3, &map_slot_name).await?;
    let initial_commitment = account.commitment();

    // Apply a delta (nonce 1)
    apply_single_entry_update(&store, &mut account, &map_slot_name, 1).await?;
    let post_delta_commitment = account.commitment();
    assert_ne!(initial_commitment, post_delta_commitment);

    // Look up the initial commitment — should find the nonce-0 state in historical
    let lookup = initial_commitment;
    let header = store
        .interact_with_connection(move |conn| {
            SqliteStore::get_account_header_by_commitment(conn, lookup)
        })
        .await?
        .expect("Initial commitment should exist in historical");
    assert_eq!(header.nonce().as_int(), 0);
    assert_eq!(header.commitment(), initial_commitment);

    // Look up the post-delta commitment — should find the nonce-1 state in historical
    let lookup = post_delta_commitment;
    let header = store
        .interact_with_connection(move |conn| {
            SqliteStore::get_account_header_by_commitment(conn, lookup)
        })
        .await?
        .expect("Post-delta commitment should exist in historical");
    assert_eq!(header.nonce().as_int(), 1);
    assert_eq!(header.commitment(), post_delta_commitment);

    Ok(())
}
