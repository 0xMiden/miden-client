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
use miden_client::{EMPTY_WORD, ONE, ZERO};
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
            let fungible_assets = SqliteStore::get_account_fungible_assets_for_delta(
                conn,
                &(&account).into(),
                &delta,
            )?;
            let storage_maps =
                SqliteStore::get_account_storage_maps_for_delta(conn, &(&account).into(), &delta)?;
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
